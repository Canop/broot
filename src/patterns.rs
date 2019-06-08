//! a pattern for filtering and sorting filenames.
//! It's backed either by a fuzzy pattern matcher or
//!  by a regular expression (in which case there's no real
//!  score)

use core::result;
use std::{fmt, mem};

use crossterm::ObjectStyle;

use crate::commands::Command;
use crate::errors::RegexError;
use crate::fuzzy_patterns::FuzzyPattern;
use crate::regex_patterns::RegexPattern;

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
    pub fn regex(pat: &str, flags: &str) -> result::Result<Pattern, RegexError> {
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
    pub fn to_command(&self) -> Command {
        Command::from(match self {
            Pattern::Fuzzy(fp) => fp.to_string(),
            Pattern::Regex(rp) => rp.to_string(),
            Pattern::None => String::new(),
        })
    }
}

/// A Match is a positive result of pattern matching
#[derive(Debug)]
pub struct Match {
    pub score: i32, // score of the match, guaranteed strictly positive, bigger is better
    pub pos: Vec<usize>, // positions of the matching chars
}

pub struct MatchedString<'a> {
    pub pattern: &'a Pattern,
    pub string: &'a str,
    pub base_style: &'a ObjectStyle,
    pub match_style: &'a ObjectStyle,
}

impl Pattern {
    pub fn style<'a>(
        &'a self,
        string: &'a str,
        base_style: &'a ObjectStyle,
        match_style: &'a ObjectStyle,
    ) -> MatchedString<'a> {
        MatchedString {
            pattern: self,
            string,
            base_style,
            match_style,
        }
    }
}

impl fmt::Display for MatchedString<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.pattern.is_some() {
            if let Some(m) = self.pattern.find(self.string) {
                let mut pos_idx: usize = 0;
                for (cand_idx, cand_char) in self.string.chars().enumerate() {
                    if pos_idx < m.pos.len() && m.pos[pos_idx] == cand_idx {
                        write!(
                            f,
                            "{}",
                            self.base_style
                                .apply_to(self.match_style.apply_to(cand_char))
                        )?;
                        pos_idx += 1;
                    } else {
                        write!(f, "{}", self.base_style.apply_to(cand_char))?;
                    }
                }
                return Ok(());
            }
        }
        write!(f, "{}", self.base_style.apply_to(self.string))
    }
}
