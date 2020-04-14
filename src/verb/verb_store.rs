
use {
    crate::{
        conf::Conf,
    },
    crossterm::event::{
        KeyEvent,
    },
    std::{
        convert::TryFrom,
    },
    super::{
        builtin::builtin_verbs,
        Verb,
    },
};

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
pub enum PrefixSearchResult<'v, T> {
    NoMatch,
    Match(T),
    TooManyMatches(Vec<&'v str>),
}

impl VerbStore {
    pub fn new() -> VerbStore {
        VerbStore { verbs: Vec::new() }
    }
    pub fn init(&mut self, conf: &Conf) {
        // we first add the verbs coming from configuration, as
        // we'll search in order. This way, a user can overload a
        // standard verb.
        for verb_conf in &conf.verbs {
            match Verb::try_from(verb_conf) {
                Ok(v) => {
                    self.verbs.push(v);
                }
                Err(e) => {
                    eprintln!("Verb error: {:?}", e);
                }
            }
        }
        self.verbs.extend(builtin_verbs());
    }
    pub fn search<'v>(&'v self, prefix: &str) -> PrefixSearchResult<'v, &Verb> {
        let mut found_index = 0;
        let mut nb_found = 0;
        let mut completions: Vec<&str> = Vec::new();
        for (index, verb) in self.verbs.iter().enumerate() {
            if let Some(shortcut) = &verb.shortcut {
                if shortcut.starts_with(prefix) {
                    if shortcut == prefix {
                        return PrefixSearchResult::Match(&verb);
                    }
                    found_index = index;
                    nb_found += 1;
                    completions.push(&shortcut);
                    continue;
                }
            }
            if verb.invocation.name.starts_with(prefix) {
                if verb.invocation.name == prefix {
                    return PrefixSearchResult::Match(&verb);
                }
                found_index = index;
                nb_found += 1;
                completions.push(&verb.invocation.name);
            }
        }
        match nb_found {
            0 => PrefixSearchResult::NoMatch,
            1 => PrefixSearchResult::Match(&self.verbs[found_index]),
            _ => PrefixSearchResult::TooManyMatches(completions),
        }
    }
    /// return the index of the verb having the long name. This function is meant
    /// for internal access when it's sure it can't fail (i.e. for a builtin)
    /// It looks for verbs by name, starting from the builtins, to
    /// ensure it hasn't been overriden.
    pub fn index_of(&self, name: &str) -> usize {
        for i in 0..self.verbs.len() {
            if self.verbs[i].invocation.name == name {
                return i;
            }
        }
        panic!("invalid verb search");
    }
    /// return the index of the verb which is triggered by the given keyboard key, if any
    pub fn index_of_key(&self, key: KeyEvent) -> Option<usize> {
        for i in 0..self.verbs.len() {
            for verb_key in &self.verbs[i].keys {
                if *verb_key == key {
                    return Some(i);
                }
            }
        }
        None
    }
}

