use {
    super::{
        builtin::builtin_verbs,
        internal::Internal,
        Verb,
    },
    crate::{
        app::SelectionType,
        conf::Conf,
        keys,
    },
    crossterm::event::KeyEvent,
    std::convert::TryFrom,
};

/// Provide access to the verbs:
/// - the built-in ones
/// - the user defined ones
/// A user defined verb can replace a built-in.
/// When the user types some keys, we select a verb
/// - if the input exactly matches a shortcut or the name
/// - if only one verb name starts with the input
#[derive(Default)]
pub struct VerbStore {
    pub verbs: Vec<Verb>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrefixSearchResult<'v, T> {
    NoMatch,
    Match(&'v str, T),
    Matches(Vec<&'v str>),
}

impl VerbStore {
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
            for name in &verb.names {
                if name.starts_with(prefix) {
                    if name == prefix {
                        return PrefixSearchResult::Match(name, &verb);
                    }
                    found_index = index;
                    nb_found += 1;
                    completions.push(name);
                    continue;
                }
            }
        }
        match nb_found {
            0 => PrefixSearchResult::NoMatch,
            1 => PrefixSearchResult::Match(completions[0], &self.verbs[found_index]),
            _ => PrefixSearchResult::Matches(completions),
        }
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

    pub fn key_desc_of_internal_stype(
        &self,
        internal: Internal,
        stype: SelectionType,
    ) -> Option<String> {
        for verb in &self.verbs {
            if verb.get_internal() == Some(internal) && stype.respects(verb.selection_condition) {
                return verb.keys.get(0).map(|&k| keys::key_event_desc(k));
            }
        }
        None
    }

    pub fn key_desc_of_internal(
        &self,
        internal: Internal,
    ) -> Option<String> {
        for verb in &self.verbs {
            if verb.get_internal() == Some(internal) {
                return verb.keys.get(0).map(|&k| keys::key_event_desc(k));
            }
        }
        None
    }

}
