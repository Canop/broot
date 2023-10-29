use {
    super::*,
    crate::{
        app::AppContext,
        errors::PatternError,
        pattern::{
            Pattern,
            PatternParts,
        },
    },
    bet::BeTree,
    lazy_regex::*,
};

/// wraps both
/// - the "pattern" (which may be used to filter and rank file entries)
/// - the source raw string which was used to build it and which may
/// be put back in the input.
#[derive(Debug, Clone)]
pub struct InputPattern {
    pub raw: String,
    pub pattern: Pattern,
}

impl PartialEq for InputPattern {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl InputPattern {
    pub fn none() -> Self {
        Self {
            raw: String::new(),
            pattern: Pattern::None,
        }
    }
    pub fn new(
        raw: String,
        parts_expr: &BeTree<PatternOperator, PatternParts>,
        con: &AppContext,
    ) -> Result<Self, PatternError> {
        let pattern = Pattern::new(parts_expr, &con.search_modes, con.content_search_max_file_size)?;
        Ok(Self { raw, pattern })
    }
    pub fn is_none(&self) -> bool {
        self.pattern.is_empty()
    }
    pub fn is_some(&self) -> bool {
        self.pattern.is_some()
    }
    /// empties the pattern and return it
    /// Similar to Option::take
    pub fn take(&mut self) -> Self {
        std::mem::replace(self, Self::none())
    }
    pub fn as_option(self) -> Option<Self> {
        if self.is_some() {
            Some(self)
        } else {
            None
        }
    }
    /// from a pattern used to filter a tree, build a pattern
    /// which would make sense to filter a previewed file
    pub fn tree_to_preview(&self) -> Self {
        let regex_parts: Option<(String, String)> = match &self.pattern {
            Pattern::ContentExact(cp) => Some(cp.to_regex_parts()),
            Pattern::ContentRegex(rp) => Some(rp.to_regex_parts()),
            Pattern::Composite(cp) => cp.expr
                .iter_atoms()
                .find_map(|p| match p {
                    Pattern::ContentExact(ce) => Some(ce.to_regex_parts()),
                    Pattern::ContentRegex(cp) => Some(cp.to_regex_parts()),
                    _ => None
                }),
            _ => None,
        };
        regex_parts
            .map(|(core, modifiers)|
                // The regex part is missing the escaping which prevents it from
                // ending the pattern in the input. We need to restore it
                // See https://github.com/Canop/broot/issues/778
                (regex_replace_all!("[ :]", &core, "\\$0").to_string(), modifiers)
            )
            .and_then(|(core, modifiers)| RegexPattern::from(&core, &modifiers).ok())
            .map(|rp| InputPattern {
                raw: rp.to_string(), // this adds the initial /
                pattern: Pattern::NameRegex(rp),
            })
            .unwrap_or_else(InputPattern::none)
    }
}
