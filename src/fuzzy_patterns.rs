//! a simple fuzzy pattern matcher for filename filtering / sorting.
//! It's not meant for file contents but for small strings (less than 1000 chars)
//!  such as file names.

use std::fmt::{self, Write};
use crate::patterns::{Match};

// weights used in match score computing
const BONUS_MATCH: i32 = 50_000;
const BONUS_EXACT: i32 = 1_000;
const BONUS_START: i32 = 0; // disabled
const BONUS_CANDIDATE_LENGTH: i32 = -1; // per char
const BONUS_LENGTH: i32 = -10; // per char of length of the match
const BONUS_NB_HOLES: i32 = -30; // there's also a max on that number

#[derive(Debug, Clone)]
pub struct FuzzyPattern {
    lc_chars: Box<[char]>, // lowercase characters
    max_nb_holes: usize,
}

impl fmt::Display for FuzzyPattern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for &c in self.lc_chars.iter() {
            f.write_char(c)?
        }
        Ok(())
    }
}

impl FuzzyPattern {
    pub fn from(pat: &str) -> FuzzyPattern {
        let lc_chars: Vec<char> = pat.chars().map(|c| c.to_ascii_lowercase()).collect();
        let lc_chars = lc_chars.into_boxed_slice();
        let max_nb_holes = match lc_chars.len() {
            1 => 0,
            2 => 1,
            3 => 2,
            4 => 3,
            5 => 3,
            6 => 4,
            7 => 4,
            8 => 4,
            _ => lc_chars.len() * 4 / 7,
        };
        FuzzyPattern {
            lc_chars,
            max_nb_holes,
        }
    }
    fn match_starting_at_index(
        &self,
        cand_chars: &[char],
        start_idx: usize, // start index in candidate
    ) -> Option<Match> {
        if cand_chars[start_idx] != self.lc_chars[0] {
            return None;
        }
        let mut pos: Vec<usize> = vec![]; // positions of matching chars in candidate
        pos.push(start_idx);
        let mut d = 1;
        let mut nb_holes = 0;
        for pat_idx in 1..self.lc_chars.len() {
            let hole_start = d;
            loop {
                let cand_idx = start_idx + d;
                if cand_idx == cand_chars.len() {
                    return None;
                }
                d += 1;
                if cand_chars[cand_idx] == self.lc_chars[pat_idx] {
                    pos.push(cand_idx);
                    break;
                }
            }
            if hole_start+1 != d {
                // note that there's no absolute guarantee we found the minimal
                // number of holes. The algorithm isn't perfect
                if nb_holes >= self.max_nb_holes {
                    return None;
                }
                nb_holes += 1;
            }
        }
        let mut score = BONUS_MATCH;
        score += BONUS_CANDIDATE_LENGTH * (cand_chars.len() as i32);
        score += BONUS_NB_HOLES * (nb_holes as i32);
        let match_len = (d as i32) - 1;
        score += match_len * BONUS_LENGTH;
        if start_idx == 0 {
            score += BONUS_START;
            if cand_chars.len() == self.lc_chars.len() {
                score += BONUS_EXACT;
            }
        }
        Some(Match { score, pos })
    }
    // return a match if the pattern can be found in the candidate string.
    // The algorithm tries to return the best one. For example if you search
    // "abc" in "ababaca-abc", the returned match would be at the end.
    pub fn find(&self, candidate: &str) -> Option<Match> {
        let cand_chars: Vec<char> = candidate.chars().map(|c| c.to_ascii_lowercase()).collect();
        if cand_chars.len() < self.lc_chars.len() {
            return None;
        }
        let mut best_score = 0;
        let mut best_match: Option<Match> = None;
        for start_idx in 0..=cand_chars.len() - self.lc_chars.len() {
            let sm = self.match_starting_at_index(&cand_chars, start_idx);
            if let Some(m) = sm {
                if m.score > best_score {
                    best_score = m.score;
                    best_match = Some(m);
                }
            }
        }
        best_match
    }
}

