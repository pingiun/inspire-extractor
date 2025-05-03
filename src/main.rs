use core::{panic, str};
use std::io::Read;

use gmlparser::StrRef;
use quick_xml::{events::Event, name::QName};

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

    let mut string_interner = gmlparser::StringInterner::default();
    let mut collector = AddressCollector::new();

    let mut current_path = Vec::new();

    loop {
        match xml_reader
            .read_event_into(&mut buf)
            .expect("Failed to read event")
        {
            Event::Start(e) => {
                let name = str::from_utf8(e.name().into_inner()).expect("Failed to decode name");
                let name = string_interner.intern(name);
                current_path.push(name);
                collector.visit_start(e);
            }
            Event::Text(e) => {
                collector.visit_text(e);
            }
            Event::Empty(e) => {
                let name = str::from_utf8(e.name().into_inner()).expect("Failed to decode name");
                let name = string_interner.intern(name);
                current_path.push(name);
                collector.visit_empty(e);
                current_path.pop();
            }
            Event::End(e) => {
                debug_assert!(
                    current_path.last() == Some(&string_interner.intern(str::from_utf8(e.name().into_inner()).expect("Failed to decode name")))
                );
                collector.visit_end(e);
                current_path.pop();
            }
            Event::Eof => break,
            _ => {}
        }
    }
}

type XmlPath = Vec<StrRef>;

fn path_ends_with(path: &XmlPath, suffix: &XmlPath) -> bool {
    if path.len() < suffix.len() {
        return false;
    }
    suffix.iter().rev().zip(path.iter().rev()).all(|(a, b)| a == b)
}

enum FeatureMember {
    Address(Address),
    AdminUnitName(AdminUnitName),
    AddressAreaName(AddressAreaName),
    ThoroughfareName(ThoroughfareName),
}

struct Address {}
struct AdminUnitName {}
struct AddressAreaName {}
struct ThoroughfareName {}

struct CurrentMember {
    local_id: Option<String>,
}

struct AddressCollector {
    state: State,
}

#[derive(Default)]
enum State {
    #[default]
    Starting,
    Member,
    UnknownType,
    ParsingAddress,
    SkippingAddressElement(Vec<u8>),
    SawInspireIdNode,
    CaptureLocalId,
}

impl AddressCollector {
    fn new() -> Self {
        AddressCollector {
            state: State::default(),
        }
    }

    fn visit_start(&mut self, e: quick_xml::events::BytesStart) {
        match self.state {
            State::Starting => {
                match e.name().as_ref() {
                    b"gml:FeatureCollection" => {
                        // Ignore
                    }
                    b"gml:featureMember" => {
                        self.state = State::Member;
                    }
                    _ => {
                        panic!("Unexpected list element: {:?}", e.name());
                    }
                }
            }
            State::Member => match e.name().as_ref() {
                b"ad:Address" => {
                    self.state = State::ParsingAddress;
                }
                b"ad:AdminUnitName" => {
                    // Todo
                    self.state = State::UnknownType;
                }
                b"ad:AddressAreaName" => {
                    // Todo
                    self.state = State::UnknownType;
                }
                b"ad:ThoroughfareName" => {
                    // Todo
                    self.state = State::UnknownType;
                }
                _ => {
                    panic!("Unexpected start element: {:?}", e.name());
                }
            },
            State::UnknownType => {
                // Ignore
            }
            State::ParsingAddress => {
                match e.name().as_ref() {
                    b"ad:inspireId" => {
                        self.state = State::SawInspireIdNode;
                    }
                    _ => {
                        self.state = State::SkippingAddressElement(e.name().into_inner().to_owned());
                    }
                }
            }
            State::SkippingAddressElement(_) => {
                // Ignore
            }
            State::SawInspireIdNode => {
                match e.name().as_ref() {
                    b"base:Identifier" => {
                        // Ignore
                    }
                    b"base:namespace" => {
                        // Ignore
                    }
                    b"base:localId" => {
                        self.state = State::CaptureLocalId;
                    }
                    _ => {
                        panic!("Unexpected element after seeing inspire ID: {:?}", e.name());
                    }
                }
            }
            State::CaptureLocalId => {
                panic!("Unexpected element while capturing localId: {:?}", e.name());
            }
        }
    }

    fn visit_end(&mut self, e: quick_xml::events::BytesEnd) {
        match self.state {
            State::Member => {
                match e.name().as_ref() {
                    b"gml:featureMember" => {
                        self.state = State::Starting;
                    }
                    _ => {
                        // Ignore
                    }
                }
            }
            State::UnknownType => {
                match e.name().as_ref() {
                    b"gml:featureMember" => {
                        self.state = State::Starting;
                    }
                    _ => {
                        // Ignore
                    }
                }
            }
            State::ParsingAddress => {
                match e.name().as_ref() {
                    b"ad:Address" => {
                        self.state = State::Member;
                    }
                    _ => {
                        // Ignore
                    }
                }
            }
            State::SkippingAddressElement(ref name) => {
                if e.name().as_ref() == name {
                    self.state = State::ParsingAddress;
                }
                // Ignore other end elements
            }
            State::SawInspireIdNode => {
                match e.name().as_ref() {
                    b"ad:inspireId" => {
                        self.state = State::ParsingAddress;
                    }
                    _ => {
                        // Ignore
                    }
                }
            }
            State::CaptureLocalId => {
                match e.name().as_ref() {
                    b"base:localId" => {
                        self.state = State::SawInspireIdNode;
                    }
                    _ => {
                        panic!("Unexpected end element while capturing localId: {:?}", e.name());
                    }
                }
            }
            _ => {}
        }
    }

    fn visit_text(&mut self, e: quick_xml::events::BytesText) {
        match self.state {
            State::CaptureLocalId => {
                let local_id = e.into_inner().into_owned();
                let local_id = String::from_utf8(local_id).expect("Failed to decode localId");
                println!("Local ID: {}", local_id);
            }
            _ => {
                // Ignore text in other states
            }
        }
    }

    fn visit_empty(&self, e: quick_xml::events::BytesStart<'_>) {
        // Ignore
    }
}
