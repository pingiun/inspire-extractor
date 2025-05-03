type IntRef = u16;

pub struct StringInterner {
    strings: Vec<&'static str>,
    string_map: std::collections::HashMap<&'static str, IntRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StrRef(IntRef);

impl Default for StringInterner {
    fn default() -> Self {
        StringInterner::new()
    }
}

impl StringInterner {
    pub fn new() -> Self {
        StringInterner {
            strings: Vec::new(),
            string_map: std::collections::HashMap::new(),
        }
    }

    pub fn intern(&mut self, s: &str) -> StrRef {
        if let Some(&id) = self.string_map.get(s) {
            StrRef(id)
        } else {
            assert!(self.strings.len() < IntRef::MAX as usize);
            let id = self.strings.len() as IntRef;
            let str = Box::leak(Box::new(s.to_string()));
            self.strings.push(str);
            self.string_map.insert(str, id);
            StrRef(id)
        }
    }

    pub fn get(&self, id: StrRef) -> &str {
        self.strings.get(id.0 as usize).unwrap()
    }
}
