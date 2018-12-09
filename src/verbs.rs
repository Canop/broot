use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::borrow::Cow;
use std::collections::HashMap;
use std::io;

use app::AppStateCmdResult;
use browser_states::BrowserState;
use conf::Conf;
use external::Launchable;
use task_sync::TaskLifetime;

#[derive(Debug, Clone)]
pub struct Verb {
    pub name: String,
    pub exec_pattern: String,
}

pub struct VerbStore {
    pub verbs: HashMap<String, Verb>,
}

impl Verb {
    pub fn description(&self) -> String {
        match self.exec_pattern.as_ref() {
            ":back" => "reverts to the previous state (mapped to `<esc>`)".to_string(),
            ":print_path" => "prints path to stdout".to_string(),
            ":focus" => "displays a directory (mapped to `<enter>`)".to_string(),
            ":open" => "opens a file according to OS settings (mapped to `<enter>`)".to_string(),
            ":parent" => "moves to the parent directory".to_string(),
            ":quit" => "quits the application".to_string(),
            ":toggle_hidden" => "toggles showing hidden files".to_string(),
            ":toggle_files" => "toggles showing files (or just folders)".to_string(),
            _ => format!("`{}`", self.exec_pattern),
        }
    }
    pub fn execute(&self, state: &BrowserState) -> io::Result<AppStateCmdResult> {
        let line = match &state.filtered_tree {
            Some(tree) => tree.selected_line(),
            None => state.tree.selected_line(),
        };
        let path = &line.path;
        Ok(match self.exec_pattern.as_ref() {
            ":back" => AppStateCmdResult::PopState,
            ":focus" => {
                let path = state.tree.selected_line().path.clone();
                let options = state.options.clone();
                AppStateCmdResult::from_optional_state(BrowserState::new(
                    path,
                    options,
                    TaskLifetime::unlimited(),
                ))
            }
            ":toggle_hidden" => {
                let mut options = state.options.clone();
                options.show_hidden = !options.show_hidden;
                AppStateCmdResult::from_optional_state(BrowserState::new(
                    state.tree.root().clone(),
                    options,
                    TaskLifetime::unlimited(),
                ))
            }
            ":toggle_files" => {
                let mut options = state.options.clone();
                options.only_folders = !options.only_folders;
                AppStateCmdResult::from_optional_state(BrowserState::new(
                    state.tree.root().clone(),
                    options,
                    TaskLifetime::unlimited(),
                ))
            }
            ":print_path" => {
                let mut launchable = Launchable::from(&path.to_string_lossy())?;
                launchable.just_print = true;
                AppStateCmdResult::Launch(launchable)
            }
            ":open" => AppStateCmdResult::Launch(Launchable::opener(path)?),
            ":parent" => match &state.tree.selected_line().path.parent() {
                Some(path) => {
                    let path = path.to_path_buf();
                    let options = state.options.clone();
                    AppStateCmdResult::from_optional_state(BrowserState::new(
                        path,
                        options,
                        TaskLifetime::unlimited(),
                    ))
                }
                None => AppStateCmdResult::DisplayError("no parent found".to_string()),
            },
            ":quit" => AppStateCmdResult::Quit,
            _ => {
                lazy_static! {
                    static ref regex: Regex = Regex::new(r"\{([\w.]+)\}").unwrap();
                }
                // TODO replace token by token and pass an array of string arguments
                let exec = regex
                    .replace_all(&*self.exec_pattern, |caps: &Captures| {
                        match caps.get(1).unwrap().as_str() {
                            "file" => path.to_string_lossy(),
                            _ => Cow::from("-hu?-"),
                        }
                    }).to_string();
                AppStateCmdResult::Launch(Launchable::from(&exec)?)
            }
        })
    }
}

impl VerbStore {
    pub fn new() -> VerbStore {
        VerbStore {
            verbs: HashMap::new(),
        }
    }
    pub fn fill_from_conf(&mut self, conf: &Conf) {
        for verb_conf in &conf.verbs {
            self.verbs.insert(
                verb_conf.invocation.to_owned(),
                Verb {
                    name: verb_conf.name.to_owned(),
                    exec_pattern: verb_conf.execution.to_owned(),
                },
            );
        }
    }
    pub fn get(&self, verb_key: &str) -> Option<&Verb> {
        self.verbs.get(verb_key)
    }
}
