use crate::errors::ProgramError;
use crate::patterns::Pattern;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OptionBool {
    Auto,
    No,
    Yes,
}

impl FromStr for OptionBool {
    type Err = ProgramError;
    fn from_str(s: &str) -> Result<OptionBool, ProgramError> {
        match s {
            "auto" => Ok(OptionBool::Auto),
            "yes" => Ok(OptionBool::Yes),
            "no" => Ok(OptionBool::No),
            _ => Err(ProgramError::ArgParse {
                bad: s.to_string(),
                valid: "auto, yes, no".to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TreeOptions {
    pub show_hidden: bool,
    pub only_folders: bool,
    pub show_sizes: bool,
    pub show_permissions: bool,
    pub respect_git_ignore: OptionBool,
    pub pattern: Option<Pattern>,
}

impl TreeOptions {
    pub fn new() -> TreeOptions {
        TreeOptions {
            show_hidden: false,
            only_folders: false,
            show_sizes: false,
            show_permissions: false,
            respect_git_ignore: OptionBool::Auto,
            pattern: None,
        }
    }
    pub fn without_pattern(&self) -> TreeOptions {
        TreeOptions {
            show_hidden: self.show_hidden,
            only_folders: self.only_folders,
            show_sizes: self.show_sizes,
            show_permissions: self.show_permissions,
            respect_git_ignore: self.respect_git_ignore,
            pattern: None,
        }
    }
}
