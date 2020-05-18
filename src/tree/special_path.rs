
use {
    crate::{
        errors::ProgramError,
    },
    glob,
    std::path::Path,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpecialHandling {
    None,
    Enter,
    NoEnter,
    Hide,
}

#[derive(Debug, Clone)]
pub struct SpecialPath {
    pub pattern: glob::Pattern,
    pub handling: SpecialHandling,
}
impl SpecialPath {
    /// parse a "glob"="handling" representation as could be found in conf
    pub fn parse(name: &str, value: &str) -> Result<Self, ProgramError> {
        let pattern = match glob::Pattern::new(name) {
            Ok(pattern) => pattern,
            Err(_) => {
                return Err(ProgramError::InvalidGlobError { pattern: name.to_string() })
            }
        };
        let value = value.to_lowercase();
        let value = regex!(r"\W+").replace_all(&value, "");
        let handling = match value.as_ref() {
            "none" => SpecialHandling::None,
            "enter" => SpecialHandling::Enter,
            "noenter" => SpecialHandling::NoEnter,
            "hide" => SpecialHandling::Hide,
            _ => {
                return Err(ProgramError::Unrecognized { token: value.to_string() })
            }
        };
        Ok(Self { pattern, handling })
    }
}

pub trait SpecialPathList {
    fn find(
        self,
        path: &Path,
    ) -> SpecialHandling;
}

impl SpecialPathList for &[SpecialPath] {
    fn find(
        self,
        path: &Path,
    ) -> SpecialHandling {
        for sp in self {
            if sp.pattern.matches_path(path) {
                return sp.handling;
            }
        }
        SpecialHandling::None
    }
}
