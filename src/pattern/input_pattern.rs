use {
    super::*,
    crate::{
        app::AppContext,
        errors::PatternError,
        pattern::{Pattern, PatternParts},
    },
    bet::BeTree,
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
        let pattern = Pattern::new(parts_expr, con)?;
        Ok(Self { raw, pattern })
    }
    pub fn is_none(&self) -> bool {
        self.raw.is_empty()
    }
    pub fn is_some(&self) -> bool {
        self.pattern.is_some()
    }
    /// empties the pattern and return it
    /// Similar to Option::take
    pub fn take(&mut self) -> Self {
        std::mem::replace(self, Self::none())
    }
    /// from a pattern used to filter a tree, build a pattern
    /// which would make sense to filter a previewed file
    pub fn tree_to_preview(&self) -> Self {
        self.pattern.get_content_pattern()
            .and_then(|cp| RegexPattern::from(&cp.to_string(), "").ok())
            .map(|rp| InputPattern {
                raw: rp.to_string(),
                pattern: Pattern::NameRegex(rp),
            })
            .unwrap_or_else(InputPattern::none)
    }
}
