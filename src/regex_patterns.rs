
use core::result;
use regex;
use crate::patterns;


#[derive(Debug, Clone)]
pub struct RegexPattern {
    rex: regex::Regex,
}

impl RegexPattern {
    pub fn from(s: &str) -> result::Result<RegexPattern, regex::Error> {
        Ok(RegexPattern {
            rex: regex::Regex::new(s)?,
        })
    }
    // return a match if the pattern can be found in the candidate string
    pub fn test(&self, candidate: &str) -> Option<patterns::Match> {
        match self.rex.find(candidate) {
            Some(rm) => {
                let mut pos = Vec::with_capacity(rm.end()-rm.start());
                for i in rm.start()..rm.end() {
                    pos.push(i);
                }
                Some(patterns::Match {
                    score: 1,
                    pos,
                })
            }
            None => None,
        }
    }
}

