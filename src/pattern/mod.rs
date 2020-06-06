use {
    std::path::Path,
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

#[derive(Debug, Clone, Copy)]
pub struct Candidate<'c> {
    pub path: &'c Path,
    pub name: &'c str,
}
