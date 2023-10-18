use {
    crate::{
        verb::*,
    },
    serde::Deserialize,
    std::{
        path::Path,
        fmt,
    },
};

/// A pattern which can be expanded into an executable
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ExecPattern {
    String(String),
    Array(Vec<String>),
}

impl ExecPattern {
    pub fn is_empty(&self) -> bool {
        match self {
            Self::String(s) => s.is_empty(),
            Self::Array(v) => v.is_empty(),
        }
    }
    pub fn has_selection_group(&self) -> bool {
        match self {
            Self::String(s) => str_has_selection_group(s),
            Self::Array(v) => v.iter().any(|s| str_has_selection_group(s)),
        }
    }
    pub fn has_other_panel_group(&self) -> bool {
        match self {
            Self::String(s) => str_has_other_panel_group(s),
            Self::Array(v) => v.iter().any(|s| str_has_other_panel_group(s)),
        }
    }
    pub fn as_internal_pattern(&self) -> Option<&str> {
        match self {
            Self::String(s) => {
                if s.starts_with(':') || s.starts_with(' ') {
                    Some(&s[1..])
                } else {
                    None
                }
            }
            Self::Array(_) => None,
        }
    }
    pub fn into_array(self) -> Vec<String> {
        match self {
            Self::String(s) => {
                splitty::split_unquoted_whitespace(&s)
                    .unwrap_quotes(true)
                    .map(|s| s.to_string())
                    .collect()
            }
            Self::Array(v) => v,
        }
    }
    pub fn from_string<T: Into<String>>(t: T) -> Self {
        Self::String(t.into())
    }
    pub fn from_array(v: Vec<String>) -> Self {
        Self::Array(v)
    }
    pub fn tokenize(self) -> Self {
        Self::Array(self.into_array())
    }
    pub fn apply(&self, f: &dyn Fn(&str) -> String) -> Self {
        Self::Array(
            match self {
                Self::String(s) => {
                    splitty::split_unquoted_whitespace(s)
                        .unwrap_quotes(true)
                        .map(f)
                        .collect()
                }
                Self::Array(v) => {
                    v.iter()
                        .map(|s| f(s))
                        .collect()
                }
            }
        )
    }
    pub fn fix_paths(self) -> Self {
        match self {
            Self::String(s) => Self::Array(
                splitty::split_unquoted_whitespace(&s)
                    .unwrap_quotes(true)
                    .map(fix_token_path)
                    .collect()
            ),
            Self::Array(v) => Self::Array(
                v.iter()
                    .map(fix_token_path)
                    .collect()
            ),
        }
    }
}

fn fix_token_path<T: Into<String> + AsRef<str>>(token: T) -> String {
    let path = Path::new(token.as_ref());
    if path.exists() {
        if let Some(path) = path.to_str() {
            return path.to_string();
        }
    }
    token.into()
}

// this implementation builds a string usable for exect
impl fmt::Display for ExecPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(s) => s.fmt(f),
            Self::Array(v) => {
                for (idx, s) in v.iter().enumerate() {
                    if idx > 0 {
                        write!(f, " ")?;
                    }
                    if s.contains(' ') {
                        write!(f, "\"{s}\"")?;
                    } else {
                        write!(f, "{s}")?;
                    }
                }
                Ok(())
            }
        }
    }
}
