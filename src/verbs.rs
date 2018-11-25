use std::borrow::Cow;
use std::collections::HashMap;
use regex::{Captures, Regex};
use lazy_static::lazy_static;
use std::path::{PathBuf};

use app::AppStateCmdResult;
use external;

#[derive(Debug, Clone)]
pub struct Verb {
    pub name: String,
    pub exec_pattern: String,
}

pub struct VerbStore {
    file_verbs: HashMap<&'static str, Verb>,
}

impl Verb {
    pub fn execute(&self, path: &PathBuf) -> AppStateCmdResult {
        if self.exec_pattern==":quit" {
            return AppStateCmdResult::Quit;
        }
        lazy_static! {
            static ref regex: Regex = Regex::new(r"\{([\w.]+)\}").unwrap();
        }
        let exec = regex
            .replace_all(&*self.exec_pattern, |caps: &Captures| {
                match caps.get(1).unwrap().as_str() {
                    "file"  => path.to_string_lossy(),
                    _       => Cow::from("-hu?-"),
                }
            }).to_string();
        external::execute(&exec);
        AppStateCmdResult::Keep
    }
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
        self.add_file_verb("q", "quit", ":quit");
    }
    pub fn file_verb(&self, verb_key: &str) -> Option<&Verb> {
        self.file_verbs.get(verb_key)
    }
}
