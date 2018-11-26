use std::borrow::Cow;
use std::collections::HashMap;
use regex::{Captures, Regex};
use lazy_static::lazy_static;
use std::path::{PathBuf};
use std::io;

use app::AppStateCmdResult;
use external::Launchable;

#[derive(Debug, Clone)]
pub struct Verb {
    pub name: String,
    pub exec_pattern: String,
}

pub struct VerbStore {
    verbs: HashMap<&'static str, Verb>,
}

impl Verb {
    pub fn execute(&self, path: &PathBuf) -> io::Result<AppStateCmdResult> {
        Ok(match self.exec_pattern.as_ref() {
            ":quit"     => {
                AppStateCmdResult::Quit
            },
            ":open"     => {
                AppStateCmdResult::Launch(Launchable::opener(path)?)
            },
            _           => {
                lazy_static! {
                    static ref regex: Regex = Regex::new(r"\{([\w.]+)\}").unwrap();
                }
                // TODO replace token by token and pass an array of string arguments
                let exec = regex
                    .replace_all(&*self.exec_pattern, |caps: &Captures| {
                        match caps.get(1).unwrap().as_str() {
                            "file"  => path.to_string_lossy(),
                            _       => Cow::from("-hu?-"),
                        }
                    }).to_string();
                AppStateCmdResult::Launch(Launchable::from(&exec)?)
            },
        })
    }
}

impl VerbStore {
    pub fn new() -> VerbStore {
        VerbStore {
            verbs: HashMap::new(),
        }
    }
    fn add(&mut self, verb_key: &'static str, name: &str, exec_pattern: &str) {
        self.verbs.insert(verb_key, Verb {
            name: name.to_owned(),
            exec_pattern: exec_pattern.to_owned(),
        });
    }
    pub fn set_defaults(&mut self) {
        self.add("c", "cd", "cd {file}");
        self.add("e", "edit", "nvim {file}");
        self.add("o", "quit", ":open");
        self.add("q", "quit", ":quit");
    }
    pub fn get(&self, verb_key: &str) -> Option<&Verb> {
        self.verbs.get(verb_key)
    }
}
