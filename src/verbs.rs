use std::collections::HashMap;

#[derive(Debug)]
pub struct Verb {
    name: String,
    exec_pattern: String,
}

pub struct VerbStore {
    file_verbs: HashMap<&'static str, Verb>,
}

impl VerbStore {
    pub fn new() -> VerbStore {
        VerbStore {
            file_verbs: HashMap::new(),
        }
    }
    fn add_file_verb(&mut self, verb_key: &'static str, name: &str, exec_pattern: &str) {
        self.file_verbs.insert(verb_key, Verb {
            name: name.to_owned(),
            exec_pattern: exec_pattern.to_owned(),
        });

    }
    pub fn set_defaults(&mut self) {
        self.add_file_verb("e", "edit", "nvim {file}");
    }
    pub fn file_verb(&self, verb_key: &str) -> Option<&Verb> {
        self.file_verbs.get(verb_key)
    }
}
