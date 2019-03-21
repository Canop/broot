use crate::conf::Conf;
use crate::verbs::Verb;

/// Provide access to the verbs:
/// - the built-in ones
/// - the user defined ones
/// When the user types some keys, we select a verb
/// - if the input exactly matches a shortcut or the key
/// - if only one verb key starts with the input
pub struct VerbStore {
    pub verbs: Vec<Verb>,
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
            verbs: Vec::new(),
        }
    }
    fn add_builtin(
        &mut self,
        key: &str,
        shortcut: Option<String>,
        description: &str,
    ) {
        self.verbs.push(Verb::create_builtin(
            key,
            shortcut,
            description,
        ));
    }
    pub fn init(&mut self, conf: &Conf) {
        // we first add the built-in verbs
        self.add_builtin(
            "back",
            None,
            "revert to the previous state (mapped to `<esc>`)",
        );
        self.verbs.push(Verb::create_external(
            "cd",
            None, // no real need for a shortcut as it's mapped to alt-enter
            "cd {directory}".to_string(),
            Some("change directory and quit (mapped to `<alt><enter>`)".to_string()),
            true, // needs to be launched from the parent shell
            true, // leaves broot
            false,
        ).unwrap());
        self.verbs.push(Verb::create_external(
            "cp {newpath}",
            None,
            "/bin/cp -r {file} {parent}/{newpath}".to_string(),
            None,
            false,
            false,
            false,
        ).unwrap());
        self.add_builtin(
            "focus",
            Some("goto".to_string()),
            "display the directory (mapped to `<enter>` in tree)",
        );
        self.add_builtin(
            "help",
            Some("?".to_string()),
            "display broot's help",
        );
        self.verbs.push(Verb::create_external(
            "mkdir {subpath}",
            Some("md".to_string()),
            "/bin/mkdir -p {directory}/{subpath}".to_string(),
            None,
            false,
            false, // doesn't leave broot
            false,
        ).unwrap());
        self.verbs.push(Verb::create_external(
            "mv {newpath}",
            None,
            "/bin/mv {file} {parent}/{newpath}".to_string(),
            None,
            false,
            false, // doesn't leave broot
            false,
        ).unwrap());
        self.add_builtin(
            "open",
            None,
            "open file according to OS settings (mapped to `<enter>`)",
        );
        self.add_builtin(
            "parent",
            None,
            "move to the parent directory",
        );
        self.add_builtin(
            "print_path",
            Some("pp".to_string()),
            "print path and leaves broot",
        );
        self.add_builtin(
            "print_tree",
            Some("pt".to_string()),
            "print tree and leaves broot",
        );
        self.add_builtin(
            "quit",
            Some("q".to_string()),
            "quit the application",
        );
        self.verbs.push(Verb::create_external(
            "rm",
            None,
            "/bin/rm -rf {file}".to_string(),
            None,
            false,
            false, // doesn't leave broot
            false,
        ).unwrap());
        self.add_builtin(
            "toggle_files",
            Some("files".to_string()),
            "toggle showing files (or just folders)",
        );
        self.add_builtin(
            "toggle_git_ignore",
            Some("gi".to_string()),
            "toggle use of .gitignore",
        );
        self.add_builtin(
            "toggle_hidden",
            Some("h".to_string()),
            "toggle showing hidden files",
        );
        self.add_builtin(
            "toggle_perm",
            Some("perm".to_string()),
            "toggle showing file permissions",
        );
        self.add_builtin(
            "toggle_sizes",
            Some("sizes".to_string()),
            "toggle showing sizes",
        );
        self.add_builtin(
            "toggle_trim_root",
            Some("t".to_string()),
            "toggle removing nodes at first level too (default)",
        );
        for verb_conf in &conf.verbs {
            match Verb::create_external(
                &verb_conf.invocation,
                verb_conf.shortcut.clone(),
                verb_conf.execution.clone(),
                verb_conf.description.clone(),
                verb_conf.from_shell.unwrap_or(false),
                verb_conf.leave_broot.unwrap_or(true),
                verb_conf.confirm.unwrap_or(false),
            ) {
                Ok(v) => {
                    self.verbs.push(v);
                }
                Err(e) => {
                    eprintln!("Verb error: {:?}", e);
                }
            }
        }
    }
    pub fn search(&self, prefix: &str) -> PrefixSearchResult<&Verb> {
        let mut found_index = 0;
        let mut nb_found = 0;
        for (index, verb) in self.verbs.iter().enumerate() {
            if let Some(shortcut) = &verb.shortcut {
                if shortcut.starts_with(prefix) {
                    if shortcut == prefix {
                        return PrefixSearchResult::Match(&verb);
                    }
                    found_index = index;
                    nb_found += 1;
                    continue;
                }
            }
            if verb.invocation.key.starts_with(prefix) {
                if verb.invocation.key == prefix {
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
    // It looks for verbs by key, starting from the builtins, to
    // ensure it hasn't been overriden.
    pub fn index_of(&self, key: &str) -> usize {
        for i in 0..self.verbs.len() {
            if self.verbs[i].invocation.key == key {
                return i;
            }
        }
        panic!("invalid verb search");
    }
}

