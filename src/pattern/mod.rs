use {
    std::{
        fs::FileType,
        path::Path,
    },
};

mod content_pattern;
mod fuzzy_patterns;
mod matched_string;
mod pattern;
mod pattern_object;
mod regex_patterns;
mod search_mode;

pub use {
    content_pattern::ContentPattern,
    fuzzy_patterns::FuzzyPattern,
    matched_string::MatchedString,
    pattern::{Match, Pattern},
    pattern_object::PatternObject,
    regex_patterns::RegexPattern,
    search_mode::{SearchMode, SearchModeMap, SearchModeMapEntry},
};

/// something in which we can search
#[derive(Debug, Clone, Copy)]
pub struct Candidate<'c> {

    /// path to the file to open if the pattern searches into files
    pub path: &'c Path,

    /// may be the filename or a subpath
    pub name: &'c str,

    /// the type of file
    pub file_type: FileType,
}
