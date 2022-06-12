use {
    crate::{
        app::AppContext,
        pattern::*,
    },
};

/// what should be shown for a search_mode in the help screen, after
/// filtering
pub struct SearchModeHelp {
    pub prefix: String,
    pub description: String,
    pub example: String,
}

/// return the rows of the "Search Modes" table in help.
pub fn search_mode_help(mode: SearchMode, con: &AppContext) -> SearchModeHelp {
    let prefix = mode.prefix(con);
    let description = format!(
        "{} search on {}",
        match mode.kind() {
            SearchKind::Exact => "exact string",
            SearchKind::Fuzzy => "fuzzy",
            SearchKind::Regex => "regex",
            SearchKind::Tokens => "tokens",
        },
        match mode.object() {
            SearchObject::Name => "file name",
            SearchObject::Path => "sub path",
            SearchObject::Content => "file content",
        },
    );
    let example = match mode {
        SearchMode::NameExact => format!("`{prefix}feat` matches *help_features.rs*"),
        SearchMode::NameFuzzy => format!("`{prefix}conh` matches *DefaultConf.hjson*"),
        SearchMode::NameRegex => format!("`{prefix}rs$` matches *build.rs*"),
        SearchMode::NameTokens => format!("`{prefix}fea,he` matches *HelpFeature.java*"),
        SearchMode::PathExact => format!("`{prefix}te\\/do` matches *website/docs*"),
        SearchMode::PathFuzzy => format!("`{prefix}flam` matches *src/flag/mod.rs*"),
        SearchMode::PathRegex => format!(r#"`{prefix}\d{{3}}.*txt` matches *dir/a123/b.txt*"#),
        SearchMode::PathTokens => format!("`{prefix}help,doc` matches *website/docs/help.md*"),
        SearchMode::ContentExact => format!("`{prefix}find(` matches a file containing *a.find(b);*"),
        SearchMode::ContentRegex => format!("`{prefix}find/i` matches a file containing *A::Find(b)*"),
    };
    SearchModeHelp {
        prefix,
        description,
        example,
    }
}

