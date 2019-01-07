#![warn(clippy::all)]

use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;

use crate::app::AppStateCmdResult;
use crate::app_context::AppContext;
use crate::browser_states::BrowserState;
use crate::conf::Conf;
use crate::external::Launchable;
use crate::help_states::HelpState;
use crate::task_sync::TaskLifetime;
use crate::tree_options::OptionBool;

#[derive(Debug, Clone)]
pub struct Verb {
    pub name: String,
    pub exec_pattern: String,
}

pub struct VerbStore {
    pub verbs: HashMap<String, Verb>,
}

pub trait VerbExecutor {
    fn execute_verb(&self, verb: &Verb, con: &AppContext) -> io::Result<AppStateCmdResult>;
}

impl VerbExecutor for HelpState {
    fn execute_verb(&self, verb: &Verb, _con: &AppContext) -> io::Result<AppStateCmdResult> {
        Ok(match verb.exec_pattern.as_ref() {
            ":quit" => AppStateCmdResult::Quit,
            _ => AppStateCmdResult::Keep,
        })
    }
}

impl VerbExecutor for BrowserState {
    fn execute_verb(&self, verb: &Verb, con: &AppContext) -> io::Result<AppStateCmdResult> {
        let tree = match &self.filtered_tree {
            Some(tree) => &tree,
            None => &self.tree,
        };
        let path = &tree.selected_line().path;
        Ok(match verb.exec_pattern.as_ref() {
            ":back" => AppStateCmdResult::PopState,
            ":focus" => {
                let path = self.tree.selected_line().path.clone();
                let options = self.tree.options.clone();
                AppStateCmdResult::from_optional_state(BrowserState::new(
                    path,
                    options,
                    &TaskLifetime::unlimited(),
                ))
            }
            ":toggle_hidden" => {
                let mut options = self.tree.options.clone();
                options.show_hidden = !options.show_hidden;
                AppStateCmdResult::from_optional_state(BrowserState::new(
                    self.tree.root().clone(),
                    options,
                    &TaskLifetime::unlimited(),
                ))
            }
            ":toggle_git_ignore" => {
                let mut options = self.tree.options.clone();
                options.respect_git_ignore = match options.respect_git_ignore {
                    OptionBool::Auto => {
                        if tree.nb_gitignored > 0 {
                            OptionBool::No
                        } else {
                            OptionBool::Yes
                        }
                    }
                    OptionBool::Yes => OptionBool::No,
                    OptionBool::No => OptionBool::Yes,
                };
                debug!("respect_git_ignore = {:?}", options.respect_git_ignore);
                AppStateCmdResult::from_optional_state(BrowserState::new(
                    self.tree.root().clone(),
                    options,
                    &TaskLifetime::unlimited(),
                ))
            }
            ":toggle_files" => {
                let mut options = self.tree.options.clone();
                options.only_folders = !options.only_folders;
                AppStateCmdResult::from_optional_state(BrowserState::new(
                    self.tree.root().clone(),
                    options,
                    &TaskLifetime::unlimited(),
                ))
            }
            ":toggle_perm" => {
                let mut options = self.tree.options.clone();
                options.show_permissions = !options.show_permissions;
                AppStateCmdResult::from_optional_state(BrowserState::new(
                    self.tree.root().clone(),
                    options,
                    &TaskLifetime::unlimited(),
                ))
            }
            ":toggle_sizes" => {
                let mut options = self.tree.options.clone();
                options.show_sizes = !options.show_sizes;
                AppStateCmdResult::from_optional_state(BrowserState::new(
                    self.tree.root().clone(),
                    options,
                    &TaskLifetime::unlimited(),
                ))
            }
            ":print_path" | ":cd" => {
                if let Some(ref output_path) = con.output_path {
                    // an output path was provided, we write to it
                    let f = OpenOptions::new().append(true).open(output_path)?;
                    writeln!(&f, "{}", path.to_string_lossy())?;
                    AppStateCmdResult::Quit
                } else {
                    // no output path provided. We write on stdout, but we must
                    // do it after app closing to have the normal terminal
                    let mut launchable = Launchable::from(&path.to_string_lossy())?;
                    launchable.just_print = true;
                    AppStateCmdResult::Launch(launchable)
                }
            }
            ":open" => AppStateCmdResult::Launch(Launchable::opener(path)?),
            ":parent" => match &self.tree.selected_line().path.parent() {
                Some(path) => {
                    let path = path.to_path_buf();
                    let options = self.tree.options.clone();
                    AppStateCmdResult::from_optional_state(BrowserState::new(
                        path,
                        options,
                        &TaskLifetime::unlimited(),
                    ))
                }
                None => AppStateCmdResult::DisplayError("no parent found".to_string()),
            },
            ":quit" => AppStateCmdResult::Quit,
            _ => AppStateCmdResult::Launch(Launchable::from(&verb.exec_string(path))?),
        })
    }
}

impl Verb {
    fn exec_string(&self, path: &Path) -> String {
        lazy_static! {
            static ref regex: Regex = Regex::new(r"\{([\w.]+)\}").unwrap();
        }
        regex
            .replace_all(&*self.exec_pattern, |caps: &Captures<'_>| {
                match caps.get(1).unwrap().as_str() {
                    "file" => path.to_string_lossy(),
                    _ => Cow::from("-hu?-"),
                }
            })
            .to_string()
    }
    pub fn description_for(&self, state: &BrowserState) -> String {
        let line = match &state.filtered_tree {
            Some(tree) => tree.selected_line(),
            None => state.tree.selected_line(),
        };

        let path = &line.path;

        if self.exec_pattern == ":cd" {
            return format!("cd {}", path.to_string_lossy());
        }

        if self.exec_pattern.starts_with(':') {
            self.description()
        } else {
            self.exec_string(path)
        }
    }
    pub fn description(&self) -> String {
        match self.exec_pattern.as_ref() {
            ":back" => "reverts to the previous state (mapped to `<esc>`)".to_string(),
            ":cd" => "changes directory - see https://github.com/Canop/broot".to_string(),
            ":print_path" => "prints path (e.g. to change directory)".to_string(),
            ":focus" => "displays a directory (mapped to `<enter>`)".to_string(),
            ":open" => "opens a file according to OS settings (mapped to `<enter>`)".to_string(),
            ":parent" => "moves to the parent directory".to_string(),
            ":quit" => "quits the application".to_string(),
            ":toggle_hidden" => "toggles showing hidden files".to_string(),
            ":toggle_git_ignore" => "toggles use of .gitignore".to_string(),
            ":toggle_files" => "toggles showing files (or just folders)".to_string(),
            ":toggle_sizes" => "toggles showing sizes".to_string(),
            _ => format!("`{}`", self.exec_pattern),
        }
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
