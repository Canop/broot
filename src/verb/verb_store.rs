use {
    super::{
        builtin::builtin_verbs,
        Internal,
        Verb,
    },
    crate::{
        app::*,
        conf::Conf,
        errors::ConfError,
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
    pub fn init(&mut self, conf: &mut Conf) -> Result<(), ConfError> {
        // We first add the verbs coming from configuration, as we'll search in order.
        // This way, a user can overload a standard verb.
        for vc in &conf.verbs {
            self.verbs.push(Verb::try_from(vc)?);
        }
        self.verbs.extend(builtin_verbs());
        Ok(())
    }

    pub fn search_sel_info<'v>(
        &'v self,
        prefix: &str,
        sel_info: &SelInfo<'_>,
    ) -> PrefixSearchResult<'v, &Verb> {
        let stype = sel_info.common_stype();
        let count = sel_info.count_paths();
        self.search(prefix, stype, Some(count))
    }

    pub fn search_prefix<'v>(
        &'v self,
        prefix: &str,
    ) -> PrefixSearchResult<'v, &Verb> {
        self.search(prefix, None, None)
    }

    pub fn search<'v>(
        &'v self,
        prefix: &str,
        stype: Option<SelectionType>,
        sel_count: Option<usize>,
    ) -> PrefixSearchResult<'v, &Verb> {
        let mut found_index = 0;
        let mut nb_found = 0;
        let mut completions: Vec<&str> = Vec::new();
        for (index, verb) in self.verbs.iter().enumerate() {
            if let Some(stype) = stype {
                if !stype.respects(verb.selection_condition) {
                    continue;
                }
            }
            if let Some(count) = sel_count {
                if count > 1 && verb.is_sequence() {
                    continue;
                }
                if count == 0 && verb.needs_selection {
                    continue;
                }
            }
            for name in &verb.names {
                if name.starts_with(prefix) {
                    if name == prefix {
                        return PrefixSearchResult::Match(name, verb);
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
