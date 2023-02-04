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
        keys::KEY_FORMAT,
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
    Match(&'v str, T),
    Matches(Vec<&'v str>),
}

impl VerbStore {
    pub fn new(conf: &mut Conf) -> Result<Self, ConfError> {
        let mut verbs = Vec::new();
        for vc in &conf.verbs {
            let verb = vc.make_verb(&verbs)?;
            verbs.push(verb);
        }
        verbs.append(&mut builtin_verbs()); // at the end so that we can override them
        Ok(Self { verbs })
    }

    pub fn search_sel_info<'v>(
        &'v self,
        prefix: &str,
        sel_info: SelInfo<'_>,
    ) -> PrefixSearchResult<'v, &Verb> {
        let stype = sel_info.common_stype();
        let count = sel_info.count_paths();
        self.search(prefix, stype, Some(count), sel_info.extension())
    }

    pub fn search_prefix<'v>(
        &'v self,
        prefix: &str,
    ) -> PrefixSearchResult<'v, &Verb> {
        self.search(prefix, None, None, None)
    }

    /// Return either the only match, or None if there's not
    /// exactly one match
    pub fn search_sel_info_unique <'v>(
        &'v self,
        prefix: &str,
        sel_info: SelInfo<'_>,
    ) -> Option<&'v Verb> {
        match self.search_sel_info(prefix, sel_info) {
            PrefixSearchResult::Match(_, verb) => Some(verb),
            _ => None,
        }
    }

    pub fn search<'v>(
        &'v self,
        prefix: &str,
        stype: Option<SelectionType>,
        sel_count: Option<usize>,
        extension: Option<&str>,
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
            if !verb.file_extensions.is_empty() && !extension.map_or(false, |ext| verb.file_extensions.iter().any(|ve| ve == ext)) {
                continue;
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

    pub fn key_desc_of_internal_stype(
        &self,
        internal: Internal,
        stype: SelectionType,
    ) -> Option<String> {
        for verb in &self.verbs {
            if verb.get_internal() == Some(internal) && stype.respects(verb.selection_condition) {
                return verb.keys.get(0).map(|&k| KEY_FORMAT.to_string(k));
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
                return verb.keys.get(0).map(|&k| KEY_FORMAT.to_string(k));
            }
        }
        None
    }

}
