//! a simple and fast fuzzy pattern matcher for filename filtering / sorting
//! It's not meant for file contents but for small strings (less than 1000 chars)
//!  such as file names.
//! Speed is prefered over score precision.

use crate::patterns::{Match};

// weights used in match score computing
const BONUS_MATCH: i32 = 10_000;
const BONUS_EXACT: i32 = 1_000;
const BONUS_START: i32 = 0; // disabled
const BONUX_CANDIDATE_LENGTH: i32 = -1; // per char
const BONUS_LENGTH: i32 = -10; // per char of length
const MAX_LENGTH_BASE: usize = 2;
const MAX_LENGTH_PER_CHAR: usize = 2;

#[derive(Debug, Clone)]
pub struct FuzzyPattern {
    lc_chars: Box<[char]>, // lowercase characters
}

impl FuzzyPattern {
    pub fn from(pat: &str) -> FuzzyPattern {
        let lc_chars: Vec<char> = pat.chars().map(|c| c.to_ascii_lowercase()).collect();
        let lc_chars = lc_chars.into_boxed_slice();
        FuzzyPattern { lc_chars }
    }
    fn match_starting_at_index(
        &self,
        cand_chars: &[char],
        start_idx: usize, // start index in candidate
        max_match_len: usize,
    ) -> Option<Match> {
        if cand_chars[start_idx] != self.lc_chars[0] {
            return None;
        }
        let mut pos: Vec<usize> = vec![]; // positions of matching chars in candidate
        pos.push(start_idx);
        let mut d = 1;
        for pat_idx in 1..self.lc_chars.len() {
            loop {
                let cand_idx = start_idx + d;
                if cand_idx == cand_chars.len() || d > max_match_len {
                    return None;
                }
                d += 1;
                if cand_chars[cand_idx] == self.lc_chars[pat_idx] {
                    pos.push(cand_idx);
                    break;
                }
            }
        }
        Some(Match { score: 0, pos })
    }
    // return a match if the pattern can be found in the candidate string
    pub fn test(&self, candidate: &str) -> Option<Match> {
        let cand_chars: Vec<char> = candidate.chars().map(|c| c.to_ascii_lowercase()).collect();
        if cand_chars.len() < self.lc_chars.len() {
            return None;
        }
        let mut best_score = 0;
        let max_match_len = MAX_LENGTH_BASE + MAX_LENGTH_PER_CHAR * self.lc_chars.len();
        let mut best_match: Option<Match> = None;
        for start_idx in 0..=cand_chars.len() - self.lc_chars.len() {
            let sm = self.match_starting_at_index(&cand_chars, start_idx, max_match_len);
            if let Some(mut m) = sm {
                let match_len = m.pos[m.pos.len() - 1] - m.pos[0];
                let mut score = BONUS_MATCH;
                score += BONUX_CANDIDATE_LENGTH * (cand_chars.len() as i32);
                if m.pos[0] == 0 {
                    score += BONUS_START;
                    if cand_chars.len() == self.lc_chars.len() {
                        score += BONUS_EXACT;
                    }
                }
                score += (match_len as i32) * BONUS_LENGTH;
                if score > best_score {
                    best_score = score;
                    m.score = score;
                    best_match = Some(m);
                }
            }
        }
        best_match
    }
}

