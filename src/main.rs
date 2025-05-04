use core::{panic, str};
use std::{path::PathBuf, sync::OnceLock};

use gmlparser::{
    FeatureMember, StrRef, StringInterner,
    emitter::{
        ChooseEmitter, FeatureMemberEmitter, multifile::MultiFileEmitter, null::NullEmitter,
    },
};
use quick_xml::events::Event;

fn main() {
    // Open the file supplied as the first argument
    let file_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/Users/jelle/Downloads/addresses.gml".to_string());
    let format = std::env::args().nth(2).unwrap_or_else(|| "tsv".to_string());
    let file = std::fs::File::open(&file_path).expect("Failed to open file");
    // Create a buffered reader
    let reader = std::io::BufReader::new(file);
    let mut xml_reader = quick_xml::reader::Reader::from_reader(reader);

    let mut buf = Vec::new();

    let emitter: ChooseEmitter = if format == "tsv" {
        // Create a TSV file emitter
        let tsv_file_path = PathBuf::from(file_path);
        MultiFileEmitter::new(tsv_file_path.parent().unwrap()).into()
    } else {
        NullEmitter::new().into()
    };

    let mut collector = AddressCollector::new(emitter);

    loop {
        match xml_reader
            .read_event_into(&mut buf)
            .expect("Failed to read event")
        {
            Event::Start(e) => {
                collector.visit_start(e);
            }
            Event::Text(e) => {
                collector.visit_text(e);
            }
            Event::Empty(e) => {
                collector.visit_empty(e);
            }
            Event::End(e) => {
                collector.visit_end(e);
            }
            Event::Eof => break,
            _ => {}
        }
    }
}

type XmlPath = Vec<StrRef>;

fn path_starts_with(path: &XmlPath, prefix: &XmlPath) -> bool {
    if path.len() < prefix.len() {
        return false;
    }
    prefix.iter().zip(path.iter()).all(|(a, b)| a == b)
}

fn path_ends_with(path: &XmlPath, suffix: &XmlPath) -> bool {
    if path.len() < suffix.len() {
        return false;
    }
    suffix
        .iter()
        .rev()
        .zip(path.iter().rev())
        .all(|(a, b)| a == b)
}

enum FeatureMemberBuilder {
    Address(AddressBuilder),
    AdminUnitName(AdminUnitNameBuilder),
    AddressAreaName(AddressAreaNameBuilder),
    ThoroughfareName(ThoroughfareNameBuilder),
}

struct AddressBuilder {
    number: Option<String>,
    number_extension: Option<String>,
    number_2nd_extension: Option<String>,
    postal_delivery_identifier: Option<String>,
    admin_unit_ref: Option<String>,
    address_area_ref: Option<String>,
    thoroughfare_ref: Option<String>,
    locator_designator_builder: Option<LocatorDesignatorBuilder>,
}
impl AddressBuilder {
    fn set_designator(&mut self, designator: LocatorDesignator) {
        match designator.type_ {
            LocatorDesignatorType::Number => self.number = Some(designator.designator),
            LocatorDesignatorType::NumberExtension => {
                self.number_extension = Some(designator.designator)
            }
            LocatorDesignatorType::Number2ndExtension => {
                self.number_2nd_extension = Some(designator.designator)
            }
            LocatorDesignatorType::PostalDeliveryIdentifier => {
                self.postal_delivery_identifier = Some(designator.designator)
            }
        }
    }
}

struct LocatorDesignatorBuilder {
    type_: Option<LocatorDesignatorType>,
    designator: Option<String>,
}

struct LocatorDesignator {
    type_: LocatorDesignatorType,
    designator: String,
}

impl LocatorDesignatorBuilder {
    fn new_with_type(type_: LocatorDesignatorType) -> LocatorDesignatorBuilder {
        LocatorDesignatorBuilder {
            type_: Some(type_),
            designator: None,
        }
    }
    fn new_with_designator(designator: String) -> LocatorDesignatorBuilder {
        LocatorDesignatorBuilder {
            type_: None,
            designator: Some(designator),
        }
    }

    fn set_type(&mut self, type_: LocatorDesignatorType) -> Option<LocatorDesignator> {
        if let Some(designator) = self.designator.take() {
            return Some(LocatorDesignator { type_, designator });
        }
        self.type_ = Some(type_);
        None
    }

    fn set_designator(&mut self, designator: String) -> Option<LocatorDesignator> {
        if let Some(type_) = self.type_.take() {
            return Some(LocatorDesignator { type_, designator });
        }
        self.designator = Some(designator);
        None
    }
}

enum LocatorDesignatorType {
    Number,
    NumberExtension,
    Number2ndExtension,
    PostalDeliveryIdentifier,
}

struct AdminUnitNameBuilder {
    name: Option<String>,
}

struct AddressAreaNameBuilder {
    name: Option<String>,
    situated_in_ref: Option<String>,
}

struct ThoroughfareNameBuilder {
    name: Option<String>,
    situated_in_ref: Option<String>,
}

impl FeatureMemberBuilder {
    fn new_from_tag(tag: &str) -> Self {
        match tag {
            "ad:Address" => FeatureMemberBuilder::Address(AddressBuilder {
                number: None,
                number_extension: None,
                number_2nd_extension: None,
                postal_delivery_identifier: None,
                admin_unit_ref: None,
                address_area_ref: None,
                thoroughfare_ref: None,
                locator_designator_builder: None,
            }),
            "ad:AdminUnitName" => {
                FeatureMemberBuilder::AdminUnitName(AdminUnitNameBuilder { name: None })
            }
            "ad:AddressAreaName" => FeatureMemberBuilder::AddressAreaName(AddressAreaNameBuilder {
                name: None,
                situated_in_ref: None,
            }),
            "ad:ThoroughfareName" => {
                FeatureMemberBuilder::ThoroughfareName(ThoroughfareNameBuilder {
                    name: None,
                    situated_in_ref: None,
                })
            }
            _ => panic!("Unknown feature member tag: {}", tag),
        }
    }
}

struct CurrentMemberBuilder {
    local_id: Option<String>,
    feature_member: FeatureMemberBuilder,
}

impl CurrentMemberBuilder {
    fn visit_start(
        &mut self,
        string_interner: &mut StringInterner,
        current_path: &[StrRef],
        e: quick_xml::events::BytesStart<'_>,
    ) {
        match &mut self.feature_member {
            FeatureMemberBuilder::AdminUnitName(_) => {
                // Don't need to check anything here
            }
            FeatureMemberBuilder::Address(builder) => {
                // Check for xlink:href attributes in reference elements
                // Extract admin_unit_ref, address_area_ref, thoroughfare_ref

                static COMPONENT_PATH: OnceLock<XmlPath> = OnceLock::new();
                let component_path = COMPONENT_PATH.get_or_init(|| {
                    vec![
                        string_interner.intern("gml:FeatureCollection"),
                        string_interner.intern("gml:featureMember"),
                        string_interner.intern("ad:Address"),
                        string_interner.intern("ad:component"),
                    ]
                });

                if current_path
                    == component_path
                {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"xlink:href" {
                            let value = String::from_utf8_lossy(&attr.value);
                            if value.starts_with("#nl-imbag-ad-adminunitname.") {
                                builder.admin_unit_ref = Some(
                                    value
                                        .strip_prefix("#nl-imbag-ad-adminunitname.")
                                        .unwrap()
                                        .to_string(),
                                );
                            } else if value.starts_with("#nl-imbag-ad-addressareaname.") {
                                builder.address_area_ref = Some(
                                    value
                                        .strip_prefix("#nl-imbag-ad-addressareaname.")
                                        .unwrap()
                                        .to_string(),
                                );
                            } else if value.starts_with("#nl-imbag-ad-thoroughfarename.") {
                                builder.thoroughfare_ref = Some(
                                    value
                                        .strip_prefix("#nl-imbag-ad-thoroughfarename.")
                                        .unwrap()
                                        .to_string(),
                                );
                            }
                        }
                    }
                }

                static DESIGNATOR_PATH: OnceLock<XmlPath> = OnceLock::new();
                let designator_path = DESIGNATOR_PATH.get_or_init(|| {
                    vec![
                        string_interner.intern("gml:FeatureCollection"),
                        string_interner.intern("gml:featureMember"),
                        string_interner.intern("ad:Address"),
                        string_interner.intern("ad:locator"),
                        string_interner.intern("ad:AddressLocator"),
                        string_interner.intern("ad:designator"),
                        string_interner.intern("ad:LocatorDesignator"),
                        string_interner.intern("ad:type"),
                    ]
                });

                if current_path == designator_path {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"xlink:href" {
                            let value = String::from_utf8_lossy(&attr.value);
                            let type_ = if value
                                == "http://inspire.ec.europa.eu/codelist/LocatorDesignatorTypeValue/addressNumber"
                            {
                                Some(LocatorDesignatorType::Number)
                            } else if value
                                == "http://inspire.ec.europa.eu/codelist/LocatorDesignatorTypeValue/addressNumberExtension"
                            {
                                Some(LocatorDesignatorType::NumberExtension)
                            } else if value
                                == "http://inspire.ec.europa.eu/codelist/LocatorDesignatorTypeValue/addressNumber2ndExtension"
                            {
                                Some(LocatorDesignatorType::Number2ndExtension)
                            } else if value
                                == "http://inspire.ec.europa.eu/codelist/LocatorDesignatorTypeValue/postalDeliveryIdentifier"
                            {
                                Some(LocatorDesignatorType::PostalDeliveryIdentifier)
                            } else {
                                None
                            };
                            if let Some(type_) = type_ {
                                if let Some(mut locator_designator) =
                                    builder.locator_designator_builder.take()
                                {
                                    if let Some(designator) = locator_designator.set_type(type_) {
                                        builder.set_designator(designator);
                                    } else {
                                        builder.locator_designator_builder =
                                            Some(locator_designator);
                                    }
                                } else {
                                    builder.locator_designator_builder =
                                        Some(LocatorDesignatorBuilder::new_with_type(type_));
                                }
                            }
                        }
                    }
                }
            }
            FeatureMemberBuilder::AddressAreaName(builder) => {
                static SITUATED_IN_PATH: OnceLock<XmlPath> = OnceLock::new();
                let situated_in_path = SITUATED_IN_PATH.get_or_init(|| {
                    vec![
                        string_interner.intern("gml:FeatureCollection"),
                        string_interner.intern("gml:featureMember"),
                        string_interner.intern("ad:AddressAreaName"),
                        string_interner.intern("ad:situatedWithin"),
                    ]
                });
                if current_path == situated_in_path {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"xlink:href" {
                            let value = String::from_utf8_lossy(&attr.value);
                            builder.situated_in_ref = Some(
                                value
                                    .strip_prefix("#nl-imbag-ad-adminunitname.")
                                    .unwrap()
                                    .to_string(),
                            );
                        }
                    }
                }
            }
            FeatureMemberBuilder::ThoroughfareName(builder) => {
                static SITUATED_IN_PATH: OnceLock<XmlPath> = OnceLock::new();
                let situated_in_path = SITUATED_IN_PATH.get_or_init(|| {
                    vec![
                        string_interner.intern("gml:FeatureCollection"),
                        string_interner.intern("gml:featureMember"),
                        string_interner.intern("ad:ThoroughfareName"),
                        string_interner.intern("ad:situatedWithin"),
                    ]
                });
                if current_path == situated_in_path {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"xlink:href" {
                            let value = String::from_utf8_lossy(&attr.value);
                            builder.situated_in_ref = Some(
                                value
                                    .strip_prefix("#nl-imbag-ad-addressareaname-")
                                    .unwrap()
                                    .to_string(),
                            );
                        }
                    }
                }
            }
        }
    }

    fn visit_end(
        &mut self,
        _: &mut StringInterner,
        _: &[StrRef],
        _: quick_xml::events::BytesEnd<'_>,
    ) {
        // Nothing to do for end tags in this implementation
    }

    fn visit_empty(
        &mut self,
        string_interner: &mut StringInterner,
        current_path: &[StrRef],
        e: quick_xml::events::BytesStart<'_>,
    ) {
        // Handle empty tags - typically reference elements
        self.visit_start(string_interner, current_path, e);
    }

    fn visit_text(
        &mut self,
        string_interner: &mut StringInterner,
        current_path: &[StrRef],
        e: quick_xml::events::BytesText<'_>,
    ) {
        // This is where you'll add specific checks for different XML paths
        match &mut self.feature_member {
            FeatureMemberBuilder::Address(builder) => {
                static DESIGNATOR_PATH: OnceLock<XmlPath> = OnceLock::new();
                let designator_path = DESIGNATOR_PATH.get_or_init(|| {
                    vec![
                        string_interner.intern("gml:FeatureCollection"),
                        string_interner.intern("gml:featureMember"),
                        string_interner.intern("ad:Address"),
                        string_interner.intern("ad:locator"),
                        string_interner.intern("ad:AddressLocator"),
                        string_interner.intern("ad:designator"),
                        string_interner.intern("ad:LocatorDesignator"),
                        string_interner.intern("ad:designator"),
                    ]
                });
                if current_path == designator_path {
                    let text = String::from_utf8(e.into_inner().into_owned()).unwrap();
                    if let Some(mut locator_designator) = builder.locator_designator_builder.take()
                    {
                        if let Some(designator) = locator_designator.set_designator(text) {
                            builder.set_designator(designator);
                        } else {
                            builder.locator_designator_builder = Some(locator_designator);
                        }
                    } else {
                        builder.locator_designator_builder =
                            Some(LocatorDesignatorBuilder::new_with_designator(text));
                    }
                }
            }
            FeatureMemberBuilder::AdminUnitName(builder) => {
                static NAME_PATH: OnceLock<XmlPath> = OnceLock::new();
                let name_path = NAME_PATH.get_or_init(|| {
                    vec![
                        string_interner.intern("gml:FeatureCollection"),
                        string_interner.intern("gml:featureMember"),
                        string_interner.intern("ad:AdminUnitName"),
                        string_interner.intern("ad:name"),
                        string_interner.intern("gn:GeographicalName"),
                        string_interner.intern("gn:spelling"),
                        string_interner.intern("gn:SpellingOfName"),
                        string_interner.intern("gn:text"),
                    ]
                });
                if current_path == name_path && builder.name.is_none() {
                    let text = String::from_utf8(e.into_inner().into_owned()).unwrap();
                    builder.name = Some(text);
                }
            }
            FeatureMemberBuilder::AddressAreaName(builder) => {
                static NAME_PATH: OnceLock<XmlPath> = OnceLock::new();
                let name_path = NAME_PATH.get_or_init(|| {
                    vec![
                        string_interner.intern("gml:FeatureCollection"),
                        string_interner.intern("gml:featureMember"),
                        string_interner.intern("ad:AddressAreaName"),
                        string_interner.intern("ad:name"),
                        string_interner.intern("gn:GeographicalName"),
                        string_interner.intern("gn:spelling"),
                        string_interner.intern("gn:SpellingOfName"),
                        string_interner.intern("gn:text"),
                    ]
                });
                if current_path == name_path && builder.name.is_none() {
                    let text = String::from_utf8(e.into_inner().into_owned()).unwrap();
                    builder.name = Some(text);
                }
            }
            FeatureMemberBuilder::ThoroughfareName(builder) => {
                static NAME_PATH: OnceLock<XmlPath> = OnceLock::new();
                let name_path = NAME_PATH.get_or_init(|| {
                    vec![
                        string_interner.intern("gml:FeatureCollection"),
                        string_interner.intern("gml:featureMember"),
                        string_interner.intern("ad:ThoroughfareName"),
                        string_interner.intern("ad:name"),
                        string_interner.intern("ad:ThoroughfareNameValue"),
                        string_interner.intern("ad:name"),
                        string_interner.intern("gn:GeographicalName"),
                        string_interner.intern("gn:spelling"),
                        string_interner.intern("gn:SpellingOfName"),
                        string_interner.intern("gn:text"),
                    ]
                });
                if current_path == name_path && builder.name.is_none() {
                    let text = String::from_utf8(e.into_inner().into_owned()).unwrap();
                    builder.name = Some(text);
                }
            }
        }
    }

    fn finish(self) -> FeatureMember {
        match self.feature_member {
            FeatureMemberBuilder::Address(builder) => FeatureMember::Address {
                local_id: self.local_id.expect("Local ID not set"),
                number: builder.number,
                number_extension: builder.number_extension,
                number_2nd_extension: builder.number_2nd_extension,
                postal_delivery_identifier: builder.postal_delivery_identifier,
                admin_unit_ref: builder.admin_unit_ref,
                address_area_ref: builder.address_area_ref,
                thoroughfare_ref: builder.thoroughfare_ref,
            },
            FeatureMemberBuilder::AdminUnitName(builder) => FeatureMember::AdminUnitName {
                local_id: self.local_id.expect("Local ID not set"),
                name: builder.name,
            },
            FeatureMemberBuilder::AddressAreaName(builder) => FeatureMember::AddressAreaName {
                local_id: self.local_id.expect("Local ID not set"),
                name: builder.name,
                situated_in_ref: builder.situated_in_ref,
            },
            FeatureMemberBuilder::ThoroughfareName(builder) => FeatureMember::ThoroughfareName {
                local_id: self.local_id.expect("Local ID not set"),
                name: builder.name,
                situated_in_ref: builder.situated_in_ref,
            },
        }
    }
}

struct AddressCollector<T> {
    string_interner: gmlparser::StringInterner,
    current_path: XmlPath,
    current_member: Option<CurrentMemberBuilder>,
    emitter: T,
}

static FEATURE_MEMBER_PREFIX: OnceLock<XmlPath> = OnceLock::new();

fn init_feature_member_prefix(interner: &mut StringInterner) -> impl FnOnce() -> XmlPath {
    move || {
        vec![
            interner.intern("gml:FeatureCollection"),
            interner.intern("gml:featureMember"),
        ]
    }
}

impl<T> AddressCollector<T>
where
    T: FeatureMemberEmitter,
{
    fn new(emitter: T) -> Self {
        AddressCollector {
            string_interner: gmlparser::StringInterner::default(),
            current_path: Vec::new(),
            current_member: None,
            emitter,
        }
    }

    fn visit_start(&mut self, e: quick_xml::events::BytesStart) {
        let feature_member_tag =
            FEATURE_MEMBER_PREFIX.get_or_init(init_feature_member_prefix(&mut self.string_interner));

        let name_ref = self
            .string_interner
            .intern(str::from_utf8(e.name().as_ref()).unwrap());
        self.current_path.push(name_ref);

        if self.current_path == *feature_member_tag || self.current_path.len() < 2 {
            assert!(self.current_member.is_none());
        } else if path_starts_with(&self.current_path, feature_member_tag)
            && self.current_path.len() == feature_member_tag.len() + 1
        {
            // A new member is starting
            let tag = self.string_interner.get(name_ref);
            let feature_member = FeatureMemberBuilder::new_from_tag(tag);
            self.current_member = Some(CurrentMemberBuilder {
                local_id: None,
                feature_member,
            });
        } else {
            assert!(self.current_member.is_some());
            self.current_member.as_mut().unwrap().visit_start(
                &mut self.string_interner,
                &self.current_path,
                e,
            );
        }
    }

    fn visit_end(&mut self, e: quick_xml::events::BytesEnd) {
        let feature_member_tag =
            FEATURE_MEMBER_PREFIX.get_or_init(init_feature_member_prefix(&mut self.string_interner));

        let name_ref = self
            .string_interner
            .intern(str::from_utf8(e.name().as_ref()).unwrap());
        if self.current_path.last() == Some(&name_ref) {
            self.current_path.pop();
        } else {
            panic!("Mismatched end tag: {:?}", name_ref);
        }

        if self.current_member.is_none() {
            assert!(self.current_path.len() <= 1);
            self.emitter.flush();
            return;
        }

        if self.current_path == *feature_member_tag {
            let current_member = self.current_member.take().unwrap();
            let finished_member = current_member.finish();
            self.emitter.emit(finished_member);
        } else {
            self.current_member.as_mut().unwrap().visit_end(
                &mut self.string_interner,
                &self.current_path,
                e,
            );
        }
    }

    fn visit_text(&mut self, e: quick_xml::events::BytesText) {
        static LOCAL_ID_SUFFIX: OnceLock<XmlPath> = OnceLock::new();
        let local_id_suffix = LOCAL_ID_SUFFIX.get_or_init(|| {
            vec![
                self.string_interner.intern("ad:inspireId"),
                self.string_interner.intern("base:Identifier"),
                self.string_interner.intern("base:localId"),
            ]
        });

        assert!(self.current_member.is_some());

        let current_member = self.current_member.as_mut().unwrap();
        if path_ends_with(&self.current_path, local_id_suffix) {
            current_member.local_id = Some(String::from_utf8(e.into_inner().into_owned()).unwrap());
            return;
        }

        current_member.visit_text(&mut self.string_interner, &self.current_path, e);
    }

    fn visit_empty(&mut self, e: quick_xml::events::BytesStart<'_>) {
        let name_ref = self
            .string_interner
            .intern(str::from_utf8(e.name().as_ref()).unwrap());
        self.current_path.push(name_ref);

        assert!(self.current_member.is_some());

        self.current_member.as_mut().unwrap().visit_empty(
            &mut self.string_interner,
            &self.current_path,
            e,
        );

        self.current_path.pop();
    }
}
