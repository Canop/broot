//! a pattern for filtering and sorting filenames.
//! It's backed either by a fuzzy pattern matcher or
//!  by a regular expression (in which case there's no real
//!  score)

use core::result;
use std::{fmt, mem};

use crate::errors::RegexError;
use crate::fuzzy_patterns::{FuzzyPattern};
use crate::regex_patterns::{RegexPattern};

#[derive(Debug, Clone)]
pub enum Pattern {
    None,
    Fuzzy(FuzzyPattern),
    Regex(RegexPattern),
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
    pub fn regex(pat: &str, flags: &str) -> result::Result<Pattern, RegexError> {
        Ok(Pattern::Regex(RegexPattern::from(pat, flags)?))
    }
    pub fn find(&self, candidate: &str) -> Option<Match> {
        match self {
            Pattern::Fuzzy(fp) => fp.find(candidate),
            Pattern::Regex(rp) => rp.find(candidate),
            Pattern::None => Some(Match { // this isn't really supposed to be used
                score: 1,
                pos: Vec::with_capacity(0),
            }),
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
}

/// A Match is a positive result of pattern matching
#[derive(Debug)]
pub struct Match {
    pub score: i32,  // score of the match, guaranteed strictly positive, bigger is better
    pub pos: Vec<usize>, // positions of the matching chars
}

impl Match {
    // returns a new string made from candidate (which should be at the origin of the match)
    //  where the characters at positions pos (matching chars) are wrapped between
    //  prefix and postfix
    pub fn wrap_matching_chars(&self, candidate: &str, prefix: &str, postfix: &str) -> String {
        let mut pos_idx: usize = 0;
        let mut decorated = String::new();
        for (cand_idx, cand_char) in candidate.chars().enumerate() {
            if pos_idx < self.pos.len() && self.pos[pos_idx] == cand_idx {
                decorated.push_str(prefix);
                decorated.push(cand_char);
                decorated.push_str(postfix);
                pos_idx += 1;
            } else {
                decorated.push(cand_char);
            }
        }
        decorated
    }
}
