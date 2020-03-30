//! a pattern for filtering and sorting filenames.
//! It's backed either by a fuzzy pattern matcher or
//!  by a regular expression (in which case there's no real
//!  score)

mod fuzzy_patterns;
mod matched_string;
mod pattern;
mod regex_patterns;

pub use {
    fuzzy_patterns::FuzzyPattern,
    matched_string::MatchedString,
    pattern::{
        Match,
        Pattern,
    },
    regex_patterns::RegexPattern,
};

