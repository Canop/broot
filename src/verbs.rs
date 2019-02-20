use regex::Regex;
/// Verbs are the engines of broot commands, and apply
/// - to the selected file (if user-defined, then must contain {file} or {directory})
/// - to the current app state
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::app::AppStateCmdResult;
use crate::app_context::AppContext;
use crate::conf::Conf;
use crate::external;
use crate::screens::Screen;

#[derive(Debug, Clone)]
pub struct Verb {
    pub name: String,              // a public name, eg "cd"
    pub short_key: Option<String>, // a shortcut, eg "c"
    pub long_key: String,          // a long typable key, eg "cd"
    pub exec_pattern: String,      // a pattern usable for execution, eg ":cd" or "less {file}"
    pub description: String,       // a description for the user
    pub from_shell: bool, // whether it must be launched from the parent shell (eg because it's a shell function)
}

impl Verb {
    fn create(
        name: String,
        invocation: Option<String>,
        exec_pattern: String,
        description: String,
        from_shell: bool,
    ) -> Verb {
        // we build the long key such as
        // ":goto" -> "goto"
        lazy_static! {
            static ref RE: Regex = Regex::new(r"\w+").unwrap();
        }
        Verb {
            name,
            short_key: invocation,
            long_key: RE
                .find(&exec_pattern)
                .map_or("", |m| m.as_str())
                .to_string(),
            exec_pattern,
            description,
            from_shell,
        }
    }
    // built-ins are verbs offering a logic other than the execution
    //  based on exec_pattern. They mostly modify the appstate
    fn create_built_in(name: &str, short_key: Option<String>, description: &str) -> Verb {
        Verb {
            name: name.to_string(),
            short_key,
            long_key: name.to_string(),
            exec_pattern: (format!(":{}", name)).to_string(),
            description: description.to_string(),
            from_shell: false,
        }
    }
    pub fn description_for(&self, mut path: PathBuf) -> String {
        //let mut path = path;
        if self.exec_pattern == ":cd" {
            if !path.is_dir() {
                path = path.parent().unwrap().to_path_buf();
            }
            format!("cd {}", path.to_string_lossy())
        } else if self.exec_pattern.starts_with(':') {
            self.description.to_string()
        } else {
            self.exec_token(&path).join(" ")
        }
    }
    pub fn exec_token(&self, path: &Path) -> Vec<String> {
        self.exec_pattern
            .split_whitespace()
            .map(|t| {
                if t == "{file}" {
                    external::escape_for_shell(path)
                } else if t == "{directory}" {
                    let mut path = path;
                    if !path.is_dir() {
                        path = path.parent().unwrap();
                    }
                    external::escape_for_shell(path)
                } else {
                    t.to_string()
                }
            })
            .collect()
    }
    // build the cmd result for a verb defined with an exec pattern.
    // Calling this function on a built-in doesn't make sense
    pub fn to_cmd_result(&self, path: &Path, con: &AppContext) -> io::Result<AppStateCmdResult> {
        Ok(if self.from_shell {
            if let Some(ref export_path) = con.launch_args.cmd_export_path {
                // new version of the br function: the whole command is exported
                // in the passed file
                let f = OpenOptions::new().append(true).open(export_path)?;
                writeln!(&f, "{}", self.exec_token(path).join(" "))?;
                AppStateCmdResult::Quit
            } else if let Some(ref export_path) = con.launch_args.file_export_path {
                // old version of the br function: only the file is exported
                // in the passed file
                let f = OpenOptions::new().append(true).open(export_path)?;
                writeln!(&f, "{}", path.to_string_lossy())?;
                AppStateCmdResult::Quit
            } else {
                AppStateCmdResult::DisplayError(
                    "this verb needs broot to be launched as `br`. Try `broot --install` if necessary.".to_string()
                )
            }
        } else {
            AppStateCmdResult::Launch(external::Launchable::from(self.exec_token(path))?)
        })
    }
}

/// Provide access to the verbs:
/// - the built-in ones
/// - the user defined ones
/// When the user types some keys, we select a verb
/// - if the input exactly matches a shortcut or the name
/// - if only one verb starts with the input
pub struct VerbStore {
    pub verbs: Vec<Verb>,
}

pub trait VerbExecutor {
    fn execute_verb(
        &self,
        verb: &Verb,
        screen: &Screen,
        con: &AppContext,
    ) -> io::Result<AppStateCmdResult>;
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
            "back",
            None,
            "revert to the previous state (mapped to `<esc>`)",
        ));
        self.verbs.push(Verb::create(
            "cd".to_string(),
            None, // no real need for a shortcut as it's mapped to alt-enter
            "cd {directory}".to_string(),
            "change directory and quit (mapped to `<alt><enter>` in tree)".to_string(),
            true, // needs to be launched from the parent shell
        ));
        self.verbs.push(Verb::create_built_in(
            "focus",
            Some("goto".to_string()),
            "display the directory (mapped to `<enter>` in tree)",
        ));
        self.verbs.push(Verb::create_built_in(
            "help",
            Some("?".to_string()),
            "display broot's help",
        ));
        self.verbs.push(Verb::create_built_in(
            "open",
            None,
            "open a file according to OS settings (mapped to `<enter>` in tree)",
        ));
        self.verbs.push(Verb::create_built_in(
            "parent",
            None,
            "move to the parent directory",
        ));
        self.verbs.push(Verb::create_built_in(
            "print_path",
            Some("pp".to_string()),
            "print path and leaves broot",
        ));
        self.verbs.push(Verb::create_built_in(
            "quit",
            Some("q".to_string()),
            "quit the application",
        ));
        self.verbs.push(Verb::create_built_in(
            "toggle_files",
            Some("files".to_string()),
            "toggle showing files (or just folders)",
        ));
        self.verbs.push(Verb::create_built_in(
            "toggle_git_ignore",
            Some("gi".to_string()),
            "toggle use of .gitignore",
        ));
        self.verbs.push(Verb::create_built_in(
            "toggle_hidden",
            Some("h".to_string()),
            "toggle showing hidden files",
        ));
        self.verbs.push(Verb::create_built_in(
            "toggle_perm",
            Some("perm".to_string()),
            "toggle showing file permissions",
        ));
        self.verbs.push(Verb::create_built_in(
            "toggle_sizes",
            Some("sizes".to_string()),
            "toggle showing sizes",
        ));
        self.verbs.push(Verb::create_built_in(
            "toggle_trim_root",
            Some("t".to_string()),
            "toggle removing nodes at first level too (default)",
        ));
        // then we add the verbs from conf
        // which may in fact be just changing the shortcut of
        // already present verbs
        for verb_conf in &conf.verbs {
            if let Some(mut v) = self
                .verbs
                .iter_mut()
                .find(|v| v.exec_pattern == verb_conf.execution)
            {
                v.short_key = Some(verb_conf.invocation.to_string());
            } else {
                self.verbs.push(Verb::create(
                    verb_conf.name.to_owned(),
                    Some(verb_conf.invocation.to_string()),
                    verb_conf.execution.to_owned(),
                    verb_conf.execution.to_owned(),
                    verb_conf.from_shell,
                ));
            }
        }
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
    // return the index of the verb having the long key. This function is meant
    // for internal access when it's sure it can't failed (i.e. for a builtin)
    // It looks for verbs by name, starting from the builtins, to
    // ensure it hasn't been overriden.
    pub fn index_of(&self, name: &str) -> usize {
        for i in 0..self.verbs.len() {
            if self.verbs[i].name == name {
                return i;
            }
        }
        panic!("invalid verb search");
    }
}
