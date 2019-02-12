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
    pub show_hidden: bool,      // whether files whose name starts with a dot should be shown
    pub only_folders: bool,     // whether to hide normal files and links
    pub show_sizes: bool,       // whether to compute and show sizes of files and dirs
    pub trim_root: bool,        // whether to cut out direct children of root
    pub show_permissions: bool, // show classic rwx unix permissions
    pub respect_git_ignore: OptionBool, // hide files as requested by .gitignore ?
    pub pattern: Pattern,       // an optional filtering/scoring pattern
}

impl TreeOptions {
    pub fn new() -> TreeOptions {
        TreeOptions {
            show_hidden: false,
            only_folders: false,
            show_sizes: false,
            trim_root: true,
            show_permissions: false,
            respect_git_ignore: OptionBool::Auto,
            pattern: Pattern::None,
        }
    }
    pub fn without_pattern(&self) -> TreeOptions {
        TreeOptions {
            show_hidden: self.show_hidden,
            only_folders: self.only_folders,
            show_sizes: self.show_sizes,
            trim_root: self.trim_root,
            show_permissions: self.show_permissions,
            respect_git_ignore: self.respect_git_ignore,
            pattern: Pattern::None,
        }
    }
}
