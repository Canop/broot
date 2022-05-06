//! a simple exact pattern matcher for filename filtering / sorting.
//! It's not meant for file contents but for small strings (less than 1000 chars)
//!  such as file names.

use {
    super::NameMatch,
    smallvec::SmallVec,
    std::{
        fmt,
        fs::File,
        io::{self, BufRead, BufReader},
        path::Path,
    },
};

// weights used in match score computing
// (but we always take the leftist match)
const BONUS_MATCH: i32 = 50_000;
const BONUS_EXACT: i32 = 1_000;
const BONUS_START: i32 = 10;
const BONUS_START_WORD: i32 = 5;
const BONUS_CANDIDATE_LENGTH: i32 = -1; // per byte
const BONUS_DISTANCE_FROM_START: i32 = -1; // per byte

/// A pattern for exact matching
#[derive(Debug, Clone)]
pub struct ExactPattern {
    pattern: String,
    chars_count: usize,
}

impl fmt::Display for ExactPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.pattern.fmt(f)
    }
}

fn is_word_separator(c: u8) -> bool {
    matches!(c, b'_' | b' ' | b'-' | b'/')
}

impl ExactPattern {
    /// build a pattern which will later be usable for fuzzy search.
    /// A pattern should be reused
    pub fn from(pattern: &str) -> Self {
        Self {
            pattern: pattern.to_string(),
            chars_count: pattern.chars().count(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.chars_count == 0
    }

    fn score(&self, start: usize, candidate: &str) -> i32 {
        // start is the byte index
        let mut score = BONUS_MATCH + BONUS_CANDIDATE_LENGTH * candidate.len() as i32;
        if start == 0 {
            score += BONUS_START;
            if candidate.len() == self.pattern.len() {
                score += BONUS_EXACT;
            }
        } else {
            if is_word_separator(candidate.as_bytes()[start - 1]) {
                score += BONUS_START_WORD;
            }
            score += BONUS_DISTANCE_FROM_START * start as i32;
        }
        score
    }

    /// return a match if the pattern can be found in the candidate string.
    pub fn find(&self, candidate: &str) -> Option<NameMatch> {
        candidate.find(&self.pattern)
            .map(|start| {
                let score = self.score(start, candidate);
                // we must find the start in chars, not bytes
                for (char_idx, (byte_idx, _)) in candidate.char_indices().enumerate() {
                    if byte_idx == start {
                        let mut pos = SmallVec::with_capacity(self.chars_count);
                        for i in 0..self.chars_count {
                            pos.push(i + char_idx);
                        }
                        return NameMatch {
                            score,
                            pos,
                        };
                    }
                }
                unreachable!(); // if there was a match, pos should have been reached
            })
    }

    /// get the line of the first match, if any
    /// (not used today, we use content_pattern to search in files)
    pub fn try_get_match_line_count(
        &self,
        path: &Path,
    ) -> io::Result<Option<usize>> {
        let mut line_count = 1; // first line in text editors is 1
        for line in BufReader::new(File::open(path)?).lines() {
            let line = line?;
            if line.contains(&self.pattern) {
                return Ok(Some(line_count));
            }
            line_count = 1;
        }
        Ok(None)
    }

    /// get the line of the first match, if any
    /// (not used today, we use content_pattern to search in files)
    pub fn get_match_line_count(
        &self,
        path: &Path,
    ) -> Option<usize> {
        self.try_get_match_line_count(path)
            .unwrap_or(None)
    }

    /// compute the score of the best match
    pub fn score_of(&self, candidate: &str) -> Option<i32> {
        candidate
            .find(&self.pattern)
            .map(|start| self.score(start, candidate))
    }
}

