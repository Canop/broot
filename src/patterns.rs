//! a pattern for filtering and sorting filenames.
//! It's backed either by a fuzzy pattern matcher or
//!  by a regular expression (in which case there's no real
//!  score)

use std::{fmt, mem};

use crate::{errors::RegexError, fuzzy_patterns::FuzzyPattern, regex_patterns::RegexPattern};

#[derive(Debug, Clone)]
pub enum Pattern {
    None,
    Fuzzy(FuzzyPattern),
    Regex(RegexPattern),
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Pattern::Fuzzy(fp) => write!(f, "Fuzzy({})", fp),
            Pattern::Regex(rp) => write!(f, "Regex({})", rp),
            Pattern::None => write!(f, "None"),
        }
    }
}

impl Pattern {
    /// create a new fuzzy pattern
    pub fn fuzzy(pat: &str) -> Pattern {
        Pattern::Fuzzy(FuzzyPattern::from(pat))
    }
    /// try to create a regex pattern
    pub fn regex(pat: &str, flags: &str) -> Result<Pattern, RegexError> {
        Ok(Pattern::Regex(RegexPattern::from(pat, flags)?))
    }
    pub fn find(&self, candidate: &str) -> Option<Match> {
        match self {
            Pattern::Fuzzy(fp) => fp.find(candidate),
            Pattern::Regex(rp) => rp.find(candidate),
            Pattern::None => Some(Match {
                // this isn't really supposed to be used
                score: 1,
                pos: Vec::with_capacity(0),
            }),
        }
    }
    pub fn score_of(&self, candidate: &str) -> Option<i32> {
        match self {
            Pattern::Fuzzy(fp) => fp.score_of(candidate),
            Pattern::Regex(rp) => rp.find(candidate).map(|m| m.score),
            Pattern::None => None,
        }
    }
    pub fn is_some(&self) -> bool {
        match self {
            Pattern::None => false,
            _ => true,
        }
    }
    /// empties the pattern and return it
    /// Similar to Option::take
    pub fn take(&mut self) -> Pattern {
        mem::replace(self, Pattern::None)
    }
    // return the number of results we should find before starting to
    //  sort them (unless time is runing out).
    pub fn optimal_result_number(&self, targeted_size: usize) -> usize {
        match self {
            Pattern::Fuzzy(fp) => fp.optimal_result_number(targeted_size),
            Pattern::Regex(rp) => rp.optimal_result_number(targeted_size),
            Pattern::None => targeted_size,
        }
    }
}

/// A Match is a positive result of pattern matching
#[derive(Debug, Clone)]
pub struct Match {
    pub score: i32, // score of the match, guaranteed strictly positive, bigger is better
    pub pos: Vec<usize>, // positions of the matching chars
}
