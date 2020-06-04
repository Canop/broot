
use {
    crate::{
        errors::{ConfError, PatternError},
    },
    std::str::FromStr,
};

#[derive(Debug, Clone, Copy)]
pub enum SearchMode {
    NameFuzzy,
    PathFuzzy,
    NameRegex,
    PathRegex,
}

/// define a mapping from a search mode which can be typed in
/// the input to a SearchMode value
#[derive(Debug, Clone)]
pub struct SearchModeMapEntry {
    key: Option<String>,
    mode: SearchMode,
}

/// manage how to find the search mode to apply to a
/// pattern taking the config in account.
#[derive(Debug, Clone)]
pub struct SearchModeMap {
    entries: Vec<SearchModeMapEntry>,
}

impl SearchModeMapEntry {
    pub fn parse(conf_key: &str, conf_mode: &str) -> Result<Self, ConfError> {
        let s = conf_mode.to_lowercase();
        let name = s.contains("name");
        let path = s.contains("path");
        let fuzzy = s.contains("fuzzy");
        let regex = s.contains("regex");
        let mode = match (name, path, fuzzy, regex) {
            (false, false, _, _ ) => {
                return Err(ConfError::InvalidSearchMode {
                    details: "you need either \"name\" or \"path\"".to_string()
                });
            }
            (true, true, _, _ ) => {
                return Err(ConfError::InvalidSearchMode {
                    details: "you can't simultaneously have \"name\" and \"path\"".to_string()
                });
            }
            (_, _, false, false) => {
                return Err(ConfError::InvalidSearchMode {
                    details: "you need either \"fuzzy\" or \"regex\"".to_string()
                });
            }
            (_, _, true, true) => {
                return Err(ConfError::InvalidSearchMode {
                    details: "you can't simultaneously have \"fuzzy\" and \"regex\"".to_string()
                });
            }
            (true, false, true, false) => SearchMode::NameFuzzy,
            (true, false, false, true) => SearchMode::NameRegex,
            (false, true, true, false) => SearchMode::PathFuzzy,
            (false, true, false, true) => SearchMode::PathRegex,
        };
        let key = if conf_key.is_empty() || conf_key == "<empty>" {
            // serde toml parser don't handle correctly empty keys so we accept as
            // alternative the `"<empty>" = "fuzzy name"` solution.
            None
        } else if regex!(r"^\w*/$").is_match(conf_key) {
            Some(conf_key[0..conf_key.len()-1].to_string())
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
        smm.set(SearchModeMapEntry { key: None, mode: SearchMode::NameFuzzy });
        smm.setm(&["n", "nf", "fn"], SearchMode::NameFuzzy);
        smm.setm(&["", "r", "nr", "rn"], SearchMode::NameRegex);
        smm.setm(&["pf", "fp", "p"], SearchMode::PathFuzzy);
        smm.setm(&["pr", "rp"], SearchMode::PathRegex);
        smm
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
    pub fn search_mode(&self, key: &Option<String>) -> Result<SearchMode, PatternError> {
        debug!("searching mode with key {:?}", key);
        for entry in self.entries.iter().rev() {
            if entry.key == *key {
                debug!("found mode {:?}", entry.mode);
                return Ok(entry.mode);
            }
        }
        debug!("map: {:?}", &self);
        Err(PatternError::InvalidMode {
            mode: if let Some(key) = key {
                format!("{}/", key)
            } else {
                "".to_string()
            },
        })
    }
}

impl FromStr for SearchMode {
    type Err = ConfError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_ref() {
            "n" | "f" | "nf" | "fn" => Ok(Self::NameFuzzy),
            "r" | "nr" | "rn" => Ok(Self::NameRegex),
            "p" | "fp" | "pf" => Ok(Self::PathFuzzy),
            _ => {
                let name = s.contains("name");
                let path = s.contains("path");
                let fuzzy = s.contains("fuzzy");
                let regex = s.contains("regex");
                match (name, path, fuzzy, regex) {
                    (false, false, _, _ ) => Err(ConfError::InvalidSearchMode{
                        details: "you need either \"name\" or \"path\"".to_string()
                    }),
                    (true, true, _, _ ) => Err(ConfError::InvalidSearchMode{
                        details: "you can't simultaneously have \"name\" and \"path\"".to_string()
                    }),
                    (_, _, false, false) => Err(ConfError::InvalidSearchMode{
                        details: "you need either \"fuzzy\" or \"regex\"".to_string()
                    }),
                    (_, _, true, true) => Err(ConfError::InvalidSearchMode{
                        details: "you can't simultaneously have \"fuzzy\" and \"regex\"".to_string()
                    }),
                    (true, false, true, false) => Ok(Self::NameFuzzy),
                    (true, false, false, true) => Ok(Self::NameRegex),
                    (false, true, true, false) => Ok(Self::PathFuzzy),
                    (false, true, false, true) => Ok(Self::PathRegex),
                }
            }
        }
    }
}
