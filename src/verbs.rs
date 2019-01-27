use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;
use regex::Regex;

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
    pub short_key: Option<String>,
    pub long_key: String,
    pub exec_pattern: String,
    pub description: String,
}

impl Verb {
    fn create(
        name: String,
        invocation: Option<String>,
        exec_pattern: String,
        description: String,
    ) -> Verb {
        // we build the long key such as
        // ":goto" -> "goto"
        lazy_static! {
            static ref RE: Regex = Regex::new(r"\w+").unwrap();
        }
        Verb {
            name,
            short_key: invocation,
            long_key: RE.find(&exec_pattern).map_or("", |m| m.as_str()).to_string(),
            exec_pattern,
            description,
        }
    }
    fn create_built_in(
        name: &str,
        short_key: Option<String>,
        description: &str,
    ) -> Verb {
        Verb {
            name: name.to_string(),
            short_key: short_key,
            long_key: name.to_string(),
            exec_pattern: (format!(":{}", name)).to_string(),
            description: description.to_string(),
        }
    }
    #[allow(dead_code)]
    fn matches(&self, prefix: &str) -> bool {
        if let Some(s) = &self.short_key {
            if s.starts_with(prefix) {
                return true;
            }
        }
        self.long_key.starts_with(prefix)
    }
}

pub struct VerbStore {
    //pub map: BTreeMap<String, Verb>,
    pub verbs: Vec<Verb>,
}

pub trait VerbExecutor {
    fn execute_verb(&self, verb: &Verb, con: &AppContext) -> io::Result<AppStateCmdResult>;
}

impl VerbExecutor for HelpState {
    fn execute_verb(&self, verb: &Verb, _con: &AppContext) -> io::Result<AppStateCmdResult> {
        Ok(match verb.exec_pattern.as_ref() {
            ":open" => AppStateCmdResult::Launch(Launchable::opener(&Conf::default_location())?),
            ":quit" => AppStateCmdResult::Quit,
            _ => {
                if verb.exec_pattern.starts_with(':') {
                    AppStateCmdResult::Keep
                } else {
                    AppStateCmdResult::Launch(Launchable::from(verb.exec_token(&Conf::default_location()))?)
                }
            }
        })
    }
}

impl VerbExecutor for BrowserState {
    fn execute_verb(&self, verb: &Verb, con: &AppContext) -> io::Result<AppStateCmdResult> {
        let tree = match &self.filtered_tree {
            Some(tree) => &tree,
            None => &self.tree,
        };
        let line = &tree.selected_line();
        Ok(match verb.exec_pattern.as_ref() {
            ":back" => AppStateCmdResult::PopState,
            ":focus" => {
                let path = tree.selected_line().path.clone();
                let options = tree.options.clone();
                AppStateCmdResult::from_optional_state(BrowserState::new(
                    path,
                    options,
                    &TaskLifetime::unlimited(),
                ))
            }
            ":toggle_hidden" => {
                let mut options = tree.options.clone();
                options.show_hidden = !options.show_hidden;
                AppStateCmdResult::from_optional_state(BrowserState::new(
                    self.tree.root().clone(),
                    options,
                    &TaskLifetime::unlimited(),
                ))
            }
            ":toggle_git_ignore" => {
                let mut options = tree.options.clone();
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
                let mut options = tree.options.clone();
                options.only_folders = !options.only_folders;
                AppStateCmdResult::from_optional_state(BrowserState::new(
                    self.tree.root().clone(),
                    options,
                    &TaskLifetime::unlimited(),
                ))
            }
            ":toggle_perm" => {
                let mut options = tree.options.clone();
                options.show_permissions = !options.show_permissions;
                AppStateCmdResult::from_optional_state(BrowserState::new(
                    self.tree.root().clone(),
                    options,
                    &TaskLifetime::unlimited(),
                ))
            }
            ":toggle_sizes" => {
                let mut options = tree.options.clone();
                options.show_sizes = !options.show_sizes;
                AppStateCmdResult::from_optional_state(BrowserState::new(
                    self.tree.root().clone(),
                    options,
                    &TaskLifetime::unlimited(),
                ))
            }
            ":print_path" => {
                if let Some(ref output_path) = con.output_path {
                    // an output path was provided, we write to it
                    let f = OpenOptions::new().append(true).open(output_path)?;
                    writeln!(&f, "{}", line.target().to_string_lossy())?;
                    AppStateCmdResult::Quit
                } else {
                    // no output path provided. We write on stdout, but we must
                    // do it after app closing to have the normal terminal
                    let mut launchable = Launchable::from(vec![line.target().to_string_lossy().to_string()])?;
                    launchable.just_print = true;
                    AppStateCmdResult::Launch(launchable)
                }
            }
            ":cd" => {
                if let Some(ref output_path) = con.output_path {
                    // an output path was provided, we write to it
                    let f = OpenOptions::new().append(true).open(output_path)?;
                    let mut path = line.target();
                    if !line.is_dir() {
                        path = path.parent().unwrap().to_path_buf();
                    }
                    writeln!(&f, "{}", path.to_string_lossy())?;
                    AppStateCmdResult::Quit
                } else {
                    // This is a usage problem. :cd is meant to change directory
                    // and it currently can't be done without the shell companion function
                    AppStateCmdResult::DisplayError(
                        "broot not properly called. See https://github.com/Canop/broot#cd".to_string()
                    )
                }
            }
            ":open" => AppStateCmdResult::Launch(Launchable::opener(&line.target())?),
            ":parent" => match &line.target().parent() {
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
            _ => AppStateCmdResult::Launch(Launchable::from(verb.exec_token(&line.target()))?),
        })
    }
}

impl Verb {
    fn exec_token(&self, path: &Path) -> Vec<String> {
        self.exec_pattern
            .split_whitespace()
            .map(|t| if t=="{file}" { path.to_string_lossy().to_string() } else { t.to_string() })
            .collect()
    }
    pub fn description_for(&self, state: &BrowserState) -> String {
        if self.exec_pattern == ":cd" {
            let line = match &state.filtered_tree {
                Some(tree) => tree.selected_line(),
                None => state.tree.selected_line(),
            };
            let mut path = line.target();
            if !line.is_dir() {
                path = path.parent().unwrap().to_path_buf();
            }
            format!("cd {}", path.to_string_lossy())
        } else {
            self.description.to_string()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrefixSearchResult<T> {
    NoMatch,
    Match(T),
    TooManyMatches,
}

impl VerbStore {
    pub fn new() -> VerbStore {
        VerbStore {
            //map: BTreeMap::new(),
            verbs: Vec::new(),
        }
    }
    pub fn init(&mut self, conf: &Conf) {
        // we first add the built-in verbs
        self.verbs.push(Verb::create_built_in(
            "back", None, "reverts to the previous state (mapped to `<esc>`)"
        ));
        self.verbs.push(Verb::create_built_in(
            "cd", None, "changes directory - see https://github.com/Canop/broot",
        ));
        self.verbs.push(Verb::create_built_in(
            "focus", Some("goto".to_string()), "displays a directory (mapped to `<enter>`)",
        ));
        self.verbs.push(Verb::create_built_in(
            "open", None, "opens a file according to OS settings (mapped to `<enter>`)",
        ));
        self.verbs.push(Verb::create_built_in(
            "parent", None, "moves to the parent directory",
        ));
        self.verbs.push(Verb::create_built_in(
            "print_path", Some("pp".to_string()), "prints path and leaves broot",
        ));
        self.verbs.push(Verb::create_built_in(
            "quit", None, "quits the application",
        ));
        self.verbs.push(Verb::create_built_in(
            "toggle_files", Some("files".to_string()), "toggles showing files (or just folders)",
        ));
        self.verbs.push(Verb::create_built_in(
            "toggle_git_ignore", Some("gi".to_string()), "toggles use of .gitignore",
        ));
        self.verbs.push(Verb::create_built_in(
            "toggle_hidden", Some("hidden".to_string()), "toggles showing hidden files",
        ));
        self.verbs.push(Verb::create_built_in(
            "toggle_perm", Some("perm".to_string()), "toggles showing file permissions",
        ));
        self.verbs.push(Verb::create_built_in(
            "toggle_sizes", Some("sizes".to_string()), "toggles showing sizes",
        ));
        // then we add the verbs from conf
        // which may in fact be just changing the shortcut of
        // already present verbs
        for verb_conf in &conf.verbs {
            if let Some(mut v) = self.verbs.iter_mut().find(|v| v.exec_pattern==verb_conf.execution) {
                v.short_key = Some(verb_conf.invocation.to_string());
            } else {
                self.verbs.push(Verb::create(
                        verb_conf.name.to_owned(),
                        Some(verb_conf.invocation.to_string()),
                        verb_conf.execution.to_owned(),
                        verb_conf.execution.to_owned(),
                ));
            }
        }
    }
    #[allow(dead_code)]
    pub fn matching_verbs(&self, prefix: &str) -> Vec<&Verb> {
        self.verbs.iter().filter(|v| v.matches(prefix)).collect()
    }
    pub fn search(&self, prefix: &str) -> PrefixSearchResult<&Verb> {
        let mut found_index = 0;
        let mut nb_found = 0;
        for (index, verb) in self.verbs.iter().enumerate() {
            if let Some(short_key) = &verb.short_key {
                if short_key.starts_with(prefix) {
                    if short_key == prefix {
                        return PrefixSearchResult::Match(&verb);
                    }
                    found_index = index;
                    nb_found += 1;
                    continue;
                }
            }
            if verb.long_key.starts_with(prefix) {
                if verb.long_key == prefix {
                    return PrefixSearchResult::Match(&verb);
                }
                found_index = index;
                nb_found += 1;
            }
        }
        match nb_found {
            0 => PrefixSearchResult::NoMatch,
            1 => PrefixSearchResult::Match(&self.verbs[found_index]),
            _ => PrefixSearchResult::TooManyMatches,
        }
    }
}
