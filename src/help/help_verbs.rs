use crate::{
    app::AppContext,
    keys,
    pattern::*,
    verb::*,
};

/// what should be shown for a verb in the help screen, after
/// filtering
pub struct MatchingVerbRow<'v> {
    /// the name in markdown (with matching chars between stars if a pattern is active)
    name: Option<String>,
    /// the shortcut in markdown (with matching chars between stars if a pattern is active)
    shortcut: Option<String>,
    /// the description in markdown, with matching chars between stars if a pattern is active, and
    /// with backticks if the description is code
    pub description_md: String,
    /// the keys description in markdown (with matching chars between stars if a pattern is active)
    pub keys_desc: String,
    pub verb: &'v Verb,
}

impl MatchingVerbRow<'_> {
    /// the name in markdown (with matching chars in bold if
    /// some filtering occurred)
    pub fn name(&self) -> &str {
        // there should be a better way to write this
        self.name
            .as_deref()
            .unwrap_or_else(|| match self.verb.names.first() {
                Some(s) => s.as_str(),
                _ => " ",
            })
    }
    pub fn shortcut(&self) -> &str {
        // there should be a better way to write this
        self.shortcut
            .as_deref()
            .unwrap_or_else(|| match self.verb.names.get(1) {
                Some(s) => s.as_str(),
                _ => " ",
            })
    }
}

/// return the rows of the verbs table in help, taking the current filter
/// into account
pub fn matching_verb_rows<'v>(
    pat: &Pattern,
    con: &'v AppContext,
) -> Vec<MatchingVerbRow<'v>> {
    let mut rows = Vec::new();
    for verb in con.verb_store.verbs() {
        if !verb.show_in_doc {
            continue;
        }
        let mut name = None;
        let mut shortcut = None;
        let mut keys_desc = String::from(" ");
        let keys = verb
            .keys
            .iter()
            .filter(|&&k| con.modal || !keys::is_key_only_modal(k));
        for (i, key) in keys.enumerate() {
            if i > 0 {
                keys_desc.push_str(", ");
            }
            keys_desc.push_str(keys::KEY_FORMAT.to_string(*key).as_str());
        }
        let mut desc_match = None;
        let desc = &verb.description.content;
        if pat.is_some() {
            let mut ok = false;
            name = verb.names.first().and_then(|s| {
                pat.search_string(s).map(|nm| {
                    ok = true;
                    nm.wrap(s, "**", "**")
                })
            });
            shortcut = verb.names.get(1).and_then(|s| {
                pat.search_string(s).map(|nm| {
                    ok = true;
                    nm.wrap(s, "**", "**")
                })
            });
            if let Some(nm) = pat.search_string(keys_desc.as_str()) {
                keys_desc = nm.wrap(keys_desc.as_str(), "**", "**");
                ok = true;
            }
            desc_match = pat.search_string(desc);
            if desc_match.is_some() {
                ok = true;
            }
            if !ok {
                continue;
            }
        }
        let description_md = match (desc_match, verb.description.code) {
            (Some(m), false) => {
                format!("`{}`", m.wrap(desc, "`**", "**`"))
            }
            (Some(m), true) => m.wrap(desc, "**", "**"),
            (None, true) => {
                format!("`{}`", desc)
            }
            (None, false) => desc.to_string(),
        };
        rows.push(MatchingVerbRow {
            name,
            shortcut,
            keys_desc,
            description_md,
            verb,
        });
    }
    rows
}
