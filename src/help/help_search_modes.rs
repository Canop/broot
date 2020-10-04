use {
    crate::{
        app::AppContext,
        pattern::*,
    },
};

/// what should be shown for a search_mode in the help screen, after
/// filtering
pub struct MatchingSearchModeRow {
    pub prefix: String,
    pub description: String,
}

/// return the rows of the "Search Modes" table in help.
pub fn search_mode_rows(
    con: &AppContext,
) -> Vec<MatchingSearchModeRow> {
    SEARCH_MODES.iter().map(|mode| {
        let prefix = con.search_modes.key(*mode).map_or_else(
            || "".to_string(),
            |k| format!("{:>3}/", k),
        );
        let description = format!(
            "{} search on {}",
            match mode.kind() {
                SearchKind::Exact => "exact string",
                SearchKind::Fuzzy => "fuzzy",
                SearchKind::Regex => "regex",
                _ => "???", // should not happen
            },
            match mode.object() {
                SearchObject::Name => "file name",
                SearchObject::Path => "sub path",
                SearchObject::Content => "file content",
            },
        );
        MatchingSearchModeRow { prefix, description }
    }).collect()
}

