use crossterm::input::KeyEvent;

use crate::{conf::Conf, permissions, verbs::Verb};

/// Provide access to the verbs:
/// - the built-in ones
/// - the user defined ones
/// A user defined verb can replace a built-in.
/// When the user types some keys, we select a verb
/// - if the input exactly matches a shortcut or the name
/// - if only one verb name starts with the input
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
        VerbStore { verbs: Vec::new() }
    }
    fn add_builtin(
        &mut self,
        name: &str,
        key: Option<KeyEvent>,
        shortcut: Option<String>,
        description: &str,
    ) {
        self.verbs
            .push(Verb::create_builtin(name, key, shortcut, description));
    }
    pub fn init(&mut self, conf: &Conf) {
        // we first add the verbs coming from configuration, as
        // we'll search in order. This way, a user can overload a
        // standard verb.
        for verb_conf in &conf.verbs {
            match Verb::create_external(
                &verb_conf.invocation,
                // TODO remove the clone in the following line when crossterm's KeyEvent is Copy
                verb_conf.key.clone(),
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
        self.add_builtin(
            "back",
            None, // esc is mapped in commands.rs
            None,
            "revert to the previous state (mapped to `<esc>`)",
        );
        self.verbs.push(
            Verb::create_external(
                "cd",
                None,
                None, // no real need for a shortcut as it's mapped to alt-enter
                "cd {directory}".to_string(),
                Some("change directory and quit (mapped to `<alt><enter>`)".to_string()),
                true, // needs to be launched from the parent shell
                true, // leaves broot
                false,
            )
            .unwrap(),
        );
        self.verbs.push(
            Verb::create_external(
                "cp {newpath}",
                None,
                None,
                "/bin/cp -r {file} {newpath:path-from-parent}".to_string(),
                None,
                false,
                false,
                false,
            )
            .unwrap(),
        );
        self.add_builtin(
            "focus",
            None, // enter
            Some("goto".to_string()),
            "display the directory (mapped to `<enter>` in tree)",
        );
        self.add_builtin(
            "help",
            Some(KeyEvent::F(1)), // note: some terminals intercept the F1 key
            Some("?".to_string()),
            "display broot's help",
        );
        self.add_builtin(
            "line_down",
            Some(KeyEvent::Down),
            None,
            "move one line down",
        );
        self.add_builtin("line_up", Some(KeyEvent::Up), None, "move one line up");
        self.verbs.push(
            Verb::create_external(
                "mkdir {subpath}",
                None,
                Some("md".to_string()),
                "/bin/mkdir -p {subpath:path-from-directory}".to_string(),
                None,
                false,
                false, // doesn't leave broot
                false,
            )
            .unwrap(),
        );
        self.verbs.push(
            Verb::create_external(
                "mv {newpath}",
                None,
                None,
                "/bin/mv {file} {newpath:path-from-parent}".to_string(),
                None,
                false,
                false, // doesn't leave broot
                false,
            )
            .unwrap(),
        );
        self.add_builtin(
            "open_stay",
            None, // default mapping directly handled in commands#add_event
            None,
            "open file or directory according to OS settings (stays in broot)",
        );
        self.add_builtin(
            "open_leave",
            None, // default mapping directly handled in commands#add_event
            None,
            "open file or directory according to OS settings (quit broot)",
        );
        self.add_builtin(
            "page_down",
            Some(KeyEvent::PageDown),
            None,
            "scroll one page down",
        );
        self.add_builtin(
            "page_up",
            Some(KeyEvent::PageUp),
            None,
            "scroll one page up",
        );
        self.add_builtin(
            "parent",
            None,
            Some("p".to_string()),
            "move to the parent directory",
        );
        self.add_builtin(
            "print_path",
            None,
            Some("pp".to_string()),
            "print path and leaves broot",
        );
        self.add_builtin(
            "print_tree",
            None,
            Some("pt".to_string()),
            "print tree and leaves broot",
        );
        self.add_builtin(
            "quit",
            Some(KeyEvent::Ctrl('q')),
            Some("q".to_string()),
            "quit the application",
        );
        self.add_builtin(
            "refresh",
            Some(KeyEvent::F(5)),
            None,
            "refresh tree and clear size cache",
        );
        self.verbs.push(
            Verb::create_external(
                "rm",
                None, // the delete key is used in the input
                None,
                "/bin/rm -rf {file}".to_string(),
                None,
                false,
                false, // doesn't leave broot
                false,
            )
            .unwrap(),
        );
        self.add_builtin(
            "toggle_dates",
            None,
            Some("dates".to_string()),
            "toggle showing last modified dates",
        );
        self.add_builtin(
            "toggle_files",
            None,
            Some("files".to_string()),
            "toggle showing files (or just folders)",
        );
        self.add_builtin(
            "toggle_git_ignore",
            None,
            Some("gi".to_string()),
            "toggle use of .gitignore",
        );
        self.add_builtin(
            "toggle_hidden",
            None,
            Some("h".to_string()),
            "toggle showing hidden files",
        );
        if permissions::supported() {
            self.add_builtin(
                "toggle_perm",
                None,
                Some("perm".to_string()),
                "toggle showing file permissions",
            );
        }
        self.add_builtin(
            "toggle_sizes",
            None,
            Some("sizes".to_string()),
            "toggle showing sizes",
        );
        self.add_builtin(
            "toggle_trim_root",
            None,
            Some("t".to_string()),
            "toggle removing nodes at first level too (default)",
        );
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
    /// return the index of the verb having the long name. This function is meant
    /// for internal access when it's sure it can't failed (i.e. for a builtin)
    /// It looks for verbs by key, starting from the builtins, to
    /// ensure it hasn't been overriden.
    pub fn index_of(&self, name: &str) -> usize {
        for i in 0..self.verbs.len() {
            if self.verbs[i].invocation.key == name {
                return i;
            }
        }
        panic!("invalid verb search");
    }
    /// return the index of the verb which is triggered by the given key, if any
    pub fn index_of_key(&self, key: KeyEvent) -> Option<usize> {
        for i in 0..self.verbs.len() {
            // TODO remove the clone in the following line when crossterm's KeyEvent is Copy
            if let Some(verb_key) = self.verbs[i].key.clone() {
                if verb_key == key {
                    return Some(i);
                }
            }
        }
        None
    }
}
