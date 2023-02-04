
use {
    crate::{
        app::AppContext,
        errors::{ConfError, PatternError},
    },
    fnv::FnvHashMap,
    lazy_regex::regex_is_match,
    std::convert::TryFrom,
};

/// where to search
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SearchObject {
    Name,
    Path,
    Content,
}
/// how to search
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SearchKind {
    Exact,
    Fuzzy,
    Regex,
    Tokens,
}

/// a valid combination of SearchObject and SearchKind,
/// determine how a pattern will be used
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SearchMode {
    NameExact,
    NameFuzzy,
    NameRegex,
    NameTokens,
    PathExact,
    PathFuzzy,
    PathRegex,
    PathTokens,
    ContentExact,
    ContentRegex,
}

pub static SEARCH_MODES: &[SearchMode] = &[
    SearchMode::NameFuzzy,
    SearchMode::NameRegex,
    SearchMode::NameExact,
    SearchMode::NameTokens,
    SearchMode::PathExact,
    SearchMode::PathFuzzy,
    SearchMode::PathRegex,
    SearchMode::PathTokens,
    SearchMode::ContentExact,
    SearchMode::ContentRegex,
];

impl SearchMode {
    fn new(search_object: SearchObject, search_kind: SearchKind) -> Option<Self> {
        use {
            SearchObject::*,
            SearchKind::*,
        };
        match (search_object, search_kind) {
            (Name, Exact) => Some(Self::NameExact),
            (Name, Fuzzy) => Some(Self::NameFuzzy),
            (Name, Regex) => Some(Self::NameRegex),
            (Name, Tokens) => Some(Self::NameTokens),

            (Path, Exact) => Some(Self::PathExact),
            (Path, Fuzzy) => Some(Self::PathFuzzy),
            (Path, Regex) => Some(Self::PathRegex),
            (Path, Tokens) => Some(Self::PathTokens),

            (Content, Exact) => Some(Self::ContentExact),
            (Content, Fuzzy) => None, // unsupported for now - could be but why ?
            (Content, Regex) => Some(Self::ContentRegex),
            (Content, Tokens) => None, // unsupported for now - could be but need bench
        }
    }
    /// Return the prefix to type, eg "/" in standard for a name-regex,
    /// "" for a name-fuzzy, and "ep" for a path-exact
    pub fn prefix(self, con: &AppContext) -> String {
        con
            .search_modes
            .key(self)
            .map_or_else(|| "".to_string(), |k| format!("{k}/"))
    }
    pub fn object(self) -> SearchObject {
        match self {
            Self::NameExact | Self::NameFuzzy | Self::NameRegex | Self::NameTokens => SearchObject::Name,
            Self::PathExact | Self::PathFuzzy | Self::PathRegex | Self::PathTokens => SearchObject::Path,
            Self::ContentExact | Self::ContentRegex => SearchObject::Content,
        }
    }
    pub fn kind(self) -> SearchKind {
        match self {
            Self::NameExact => SearchKind::Exact,
            Self::NameFuzzy => SearchKind::Fuzzy,
            Self::NameRegex => SearchKind::Regex,
            Self::NameTokens => SearchKind::Tokens,
            Self::PathExact => SearchKind::Exact,
            Self::PathFuzzy => SearchKind::Fuzzy,
            Self::PathRegex => SearchKind::Regex,
            Self::PathTokens => SearchKind::Tokens,
            Self::ContentExact => SearchKind::Exact,
            Self::ContentRegex => SearchKind::Regex,
        }
    }
}

/// define a mapping from a search mode which can be typed in
/// the input to a SearchMode value
#[derive(Debug, Clone)]
pub struct SearchModeMapEntry {
    pub key: Option<String>,
    pub mode: SearchMode,
}

/// manage how to find the search mode to apply to a
/// pattern taking the config in account.
#[derive(Debug, Clone)]
pub struct SearchModeMap {
    pub entries: Vec<SearchModeMapEntry>,
}

impl SearchModeMapEntry {
    pub fn parse(conf_key: &str, conf_mode: &str) -> Result<Self, ConfError> {
        let mut search_kinds = Vec::new();
        let mut search_objects = Vec::new();

        let s = conf_mode.to_lowercase();
        for t in s.split_whitespace() {
            match t {
                "exact" => search_kinds.push(SearchKind::Exact),
                "fuzzy" => search_kinds.push(SearchKind::Fuzzy),
                "regex" => search_kinds.push(SearchKind::Regex),
                "tokens" => search_kinds.push(SearchKind::Tokens),
                "name" => search_objects.push(SearchObject::Name),
                "content" => search_objects.push(SearchObject::Content),
                "path" => search_objects.push(SearchObject::Path),
                _ => {
                    return Err(ConfError::InvalidSearchMode {
                        details: format!("{t:?} not understood in search mode definition"),
                    });
                }
            }
        }
        if search_kinds.is_empty() {
            return Err(ConfError::InvalidSearchMode {
                details: "missing search kind in search mode definition\
                    (the search kind must be one of 'exact', 'fuzzy', 'regex', 'tokens')".to_string()
            });
        }
        if search_kinds.len() > 1 {
            return Err(ConfError::InvalidSearchMode {
                details: "only one search kind can be specified in a search mode".to_string()
            });
        }
        if search_objects.is_empty() {
            return Err(ConfError::InvalidSearchMode {
                details: "missing search object in search mode definition\
                    (the search object must be one of 'name', 'path', 'content')".to_string()
            });
        }
        if search_objects.len() > 1 {
            return Err(ConfError::InvalidSearchMode {
                details: "only one search object can be specified in a search mode".to_string()
            });
        }

        let mode = match SearchMode::new(search_objects[0], search_kinds[0]) {
            Some(mode) => mode,
            None => {
                return Err(ConfError::InvalidSearchMode {
                    details: "Unsupported combination of search object and kind".to_string()
                });
            },
        };

        let key = if conf_key.is_empty() || conf_key == "<empty>" {
            // serde toml parser doesn't handle correctly empty keys so we accept as
            // alternative the `"<empty>" = "fuzzy name"` solution.
            // TODO look at issues and/or code in serde-toml
            None
        } else if regex_is_match!(r"^\w*/$", conf_key) {
            Some(conf_key[0..conf_key.len() - 1].to_string())
        } else {
            return Err(ConfError::InvalidKey {
                raw: conf_key.to_string(),
            });
        };
        Ok(SearchModeMapEntry { key, mode })
    }
}

impl Default for SearchModeMap {
    fn default() -> Self {
        let mut smm = SearchModeMap {
            entries: Vec::new(),
        };
        // the last keys are preferred
        smm.setm(&["ne", "en", "e"], SearchMode::NameExact);
        smm.setm(&["nf", "fn", "n", "f"], SearchMode::NameFuzzy);
        smm.setm(&["r", "nr", "rn", ""], SearchMode::NameRegex);
        smm.setm(&["pe", "ep"], SearchMode::PathExact);
        smm.setm(&["pf", "fp", "p"], SearchMode::PathFuzzy);
        smm.setm(&["pr", "rp"], SearchMode::PathRegex);
        smm.setm(&["ce", "ec", "c"], SearchMode::ContentExact);
        smm.setm(&["rx", "cr"], SearchMode::ContentRegex);
        smm.setm(&["pt", "tp", "t"], SearchMode::PathTokens);
        smm.setm(&["tn", "nt"], SearchMode::NameTokens);
        smm.set(SearchModeMapEntry { key: None, mode: SearchMode::PathFuzzy });
        smm
    }
}

impl TryFrom<&FnvHashMap<String, String>> for SearchModeMap {
    type Error = ConfError;
    fn try_from(map: &FnvHashMap<String, String>) -> Result<Self, Self::Error> {
        let mut smm = Self::default();
        for (k, v) in map {
            smm.entries.push(SearchModeMapEntry::parse(k, v)?);
        }
        Ok(smm)
    }
}

impl SearchModeMap {
    pub fn setm(&mut self, keys: &[&str], mode: SearchMode) {
        for key in keys {
            self.set(SearchModeMapEntry {
                key: Some(key.to_string()),
                mode,
            });
        }
    }
    /// we don't remove existing entries to ensure there's always a matching entry in
    /// mode->key (but search iterations will be done in reverse)
    pub fn set(&mut self, entry: SearchModeMapEntry) {
        self.entries.push(entry);
    }
    pub fn search_mode(&self, key: Option<&String>) -> Result<SearchMode, PatternError> {
        for entry in self.entries.iter().rev() {
            if entry.key.as_ref() == key {
                return Ok(entry.mode);
            }
        }
        Err(PatternError::InvalidMode {
            mode: if let Some(key) = key {
                format!("{key}/")
            } else {
                "".to_string()
            },
        })
    }
    pub fn key(&self, search_mode: SearchMode) -> Option<&String> {
        for entry in self.entries.iter().rev() {
            if entry.mode == search_mode {
                return entry.key.as_ref();
            }
        }
        warn!("search mode key not found for {:?}", search_mode); // should not happen
        None
    }
}

