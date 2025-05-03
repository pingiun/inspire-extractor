use core::{panic, str};
use std::{io::Write, path::{self, Path, PathBuf}, sync::OnceLock};

use gmlparser::{StrRef, StringInterner};
use quick_xml::events::Event;

fn main() {
    // Open the file supplied as the first argument
    let file_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/Users/jelle/Downloads/addresses.gml".to_string());
    let file = std::fs::File::open(&file_path).expect("Failed to open file");
    // Create a buffered reader
    let reader = std::io::BufReader::new(file);
    let mut xml_reader = quick_xml::reader::Reader::from_reader(reader);

    let mut buf = Vec::new();

    // Create a TSV file emitter
    let tsv_file_path = PathBuf::from(file_path);
    let tsv_emitter = MultiFileEmitter::new(tsv_file_path.parent().unwrap());

    let mut collector = AddressCollector::new(tsv_emitter);

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

trait FeatureMemberEmitter {
    fn emit(&mut self, feature_member: FeatureMember);
}

struct MultiFileEmitter {
    address_writer: std::io::BufWriter<std::fs::File>,
    admin_unit_writer: std::io::BufWriter<std::fs::File>,
    address_area_writer: std::io::BufWriter<std::fs::File>,
    thoroughfare_writer: std::io::BufWriter<std::fs::File>,
}

impl MultiFileEmitter {
    fn new(base_path: &Path) -> Self {
        // Create separate files for each feature member type
        let address_file = std::fs::File::create(base_path.join("addresses.tsv"))
            .expect("Failed to create address file");
        let admin_unit_file = std::fs::File::create(base_path.join("admin_units.tsv"))
            .expect("Failed to create admin unit file");
        let address_area_file = std::fs::File::create(base_path.join("address_areas.tsv"))
            .expect("Failed to create address area file");
        let thoroughfare_file = std::fs::File::create(base_path.join("thoroughfares.tsv"))
            .expect("Failed to create thoroughfare file");

        // Write headers to each file
        let mut address_writer = std::io::BufWriter::new(address_file);
        writeln!(
            address_writer,
            "local_id\tnumber\tnumber_extension\tnumber_2nd_extension\tpostal_delivery_identifier\tunit_level\tadmin_unit_ref\taddress_area_ref\tthoroughfare_ref"
        ).expect("Failed to write address header");

        let mut admin_unit_writer = std::io::BufWriter::new(admin_unit_file);
        writeln!(
            admin_unit_writer,
            "local_id\tname"
        ).expect("Failed to write admin unit header");

        let mut address_area_writer = std::io::BufWriter::new(address_area_file);
        writeln!(
            address_area_writer,
            "local_id\tname\tsituated_in_ref"
        ).expect("Failed to write address area header");

        let mut thoroughfare_writer = std::io::BufWriter::new(thoroughfare_file);
        writeln!(
            thoroughfare_writer,
            "local_id\tname\tsituated_in_ref"
        ).expect("Failed to write thoroughfare header");

        MultiFileEmitter {
            address_writer,
            admin_unit_writer,
            address_area_writer,
            thoroughfare_writer,
        }
    }
}

impl FeatureMemberEmitter for MultiFileEmitter {
    fn emit(&mut self, feature_member: FeatureMember) {
        match feature_member {
            FeatureMember::Address {
                local_id,
                number,
                number_extension,
                number_2nd_extension,
                postal_delivery_identifier,
                unit_level,
                admin_unit_ref,
                address_area_ref,
                thoroughfare_ref,
            } => {
                // Write the address data to the address file
                writeln!(
                    self.address_writer,
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    local_id,
                    number.unwrap_or_default(),
                    number_extension.unwrap_or_default(),
                    number_2nd_extension.unwrap_or_default(),
                    postal_delivery_identifier.unwrap_or_default(),
                    unit_level.unwrap_or_default(),
                    admin_unit_ref.unwrap_or_default(),
                    address_area_ref.unwrap_or_default(),
                    thoroughfare_ref.unwrap_or_default()
                )
                .expect("Failed to write to address file");
            }
            FeatureMember::AdminUnitName { local_id, name } => {
                // Write the admin unit name data to the admin unit file
                writeln!(
                    self.admin_unit_writer,
                    "{}\t{}",
                    local_id,
                    name.unwrap_or_default()
                )
                .expect("Failed to write to admin unit file");
            }
            FeatureMember::AddressAreaName {
                local_id,
                name,
                situated_in_ref,
            } => {
                // Write the address area name data to the address area file
                writeln!(
                    self.address_area_writer,
                    "{}\t{}\t{}",
                    local_id,
                    name.unwrap_or_default(),
                    situated_in_ref.unwrap_or_default()
                )
                .expect("Failed to write to address area file");
            }
            FeatureMember::ThoroughfareName {
                local_id,
                name,
                situated_in_ref,
            } => {
                // Write the thoroughfare name data to the thoroughfare file
                writeln!(
                    self.thoroughfare_writer,
                    "{}\t{}\t{}",
                    local_id,
                    name.unwrap_or_default(),
                    situated_in_ref.unwrap_or_default()
                )
                .expect("Failed to write to thoroughfare file");
            }
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

enum FeatureMember {
    Address {
        local_id: String,
        number: Option<String>,
        number_extension: Option<String>,
        number_2nd_extension: Option<String>,
        postal_delivery_identifier: Option<String>,
        unit_level: Option<String>,
        admin_unit_ref: Option<String>,
        address_area_ref: Option<String>,
        thoroughfare_ref: Option<String>,
    },
    // Country
    AdminUnitName {
        local_id: String,
        name: Option<String>,
    },
    // City
    AddressAreaName {
        local_id: String,
        name: Option<String>,
        situated_in_ref: Option<String>,
    },
    // Street
    ThoroughfareName {
        local_id: String,
        name: Option<String>,
        situated_in_ref: Option<String>,
    },
}

enum FeatureMemberBuilder {
    Address(AddressBuilder),
    AdminUnitName(AdminUnitNameBuilder),
    AddressAreaName(AddressAreaNameBuilder),
    ThoroughfareName(ThoroughfareNameBuilder),
}

struct AddressBuilder {}
struct AdminUnitNameBuilder {}
struct AddressAreaNameBuilder {}
struct ThoroughfareNameBuilder {}

impl FeatureMemberBuilder {
    fn new_from_tag(tag: &str) -> Self {
        match tag {
            "ad:Address" => FeatureMemberBuilder::Address(AddressBuilder {}),
            "ad:AdminUnitName" => FeatureMemberBuilder::AdminUnitName(AdminUnitNameBuilder {}),
            "ad:AddressAreaName" => {
                FeatureMemberBuilder::AddressAreaName(AddressAreaNameBuilder {})
            }
            "ad:ThoroughfareName" => {
                FeatureMemberBuilder::ThoroughfareName(ThoroughfareNameBuilder {})
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
    fn visit_start(&mut self, current_path: &[StrRef], e: quick_xml::events::BytesStart<'_>) {
        todo!()
    }

    fn visit_end(&mut self, current_path: &[StrRef], e: quick_xml::events::BytesEnd<'_>) {
        todo!()
    }

    fn visit_empty(&mut self, current_path: &[StrRef], e: quick_xml::events::BytesStart<'_>) {
        todo!()
    }

    fn visit_text(&mut self, current_path: &[StrRef], e: quick_xml::events::BytesText<'_>) {
        todo!()
    }

    fn finish(self) -> FeatureMember {
        todo!()
    }
}

struct AddressCollector<T> {
    string_interner: gmlparser::StringInterner,
    current_path: XmlPath,
    current_member: Option<CurrentMemberBuilder>,
    emitter: T,
}

static FEATURE_MEMBER_TAG: OnceLock<XmlPath> = OnceLock::new();

fn init_feature_member_tag(interner: &mut StringInterner) -> impl FnOnce() -> XmlPath {
    move || {
        vec![
            interner.intern("gml:featureCollection"),
            interner.intern("gml:featureMember"),
        ]
    }
}

impl<T> AddressCollector<T> where T: FeatureMemberEmitter {
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
            FEATURE_MEMBER_TAG.get_or_init(init_feature_member_tag(&mut self.string_interner));

        let name_ref = self
            .string_interner
            .intern(str::from_utf8(e.name().as_ref()).unwrap());
        self.current_path.push(name_ref);

        if self.current_path == *feature_member_tag {
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
            self.current_member
                .as_mut()
                .unwrap()
                .visit_start(&self.current_path, e);
        }
    }

    fn visit_end(&mut self, e: quick_xml::events::BytesEnd) {
        let feature_member_tag =
            FEATURE_MEMBER_TAG.get_or_init(init_feature_member_tag(&mut self.string_interner));

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
            return;
        }

        if self.current_path == *feature_member_tag {
            let current_member = self.current_member.take().unwrap();
            let finished_member = current_member.finish();
            self.emitter.emit(finished_member);
        } else {
            self.current_member
                .as_mut()
                .unwrap()
                .visit_end(&self.current_path, e);
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

        current_member.visit_text(&self.current_path, e);
    }

    fn visit_empty(&mut self, e: quick_xml::events::BytesStart<'_>) {
        let name_ref = self
            .string_interner
            .intern(str::from_utf8(e.name().as_ref()).unwrap());
        self.current_path.push(name_ref);

        assert!(self.current_member.is_some());

        self.current_member
            .as_mut()
            .unwrap()
            .visit_empty(&self.current_path, e);

        self.current_path.pop();
    }
}
