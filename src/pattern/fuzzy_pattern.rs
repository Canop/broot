//! a simple fuzzy pattern matcher for filename filtering / sorting.
//! It's not meant for file contents but for small strings (less than 1000 chars)
//!  such as file names.

use {
    super::NameMatch,
    secular,
    smallvec::smallvec,
    std::fmt::{self, Write},
};

// weights used in match score computing
const BONUS_MATCH: i32 = 50_000;
const BONUS_EXACT: i32 = 1_000;
const BONUS_START: i32 = 10;
const BONUS_START_WORD: i32 = 5;
const BONUS_CANDIDATE_LENGTH: i32 = -1; // per char
const BONUS_MATCH_LENGTH: i32 = -10; // per char of length of the match
const BONUS_NB_HOLES: i32 = -30; // there's also a max on that number
const BONUS_SINGLED_CHAR: i32 = -15; // when there's a char, neither first not last, isolated

/// A pattern for fuzzy matching
#[derive(Debug, Clone)]
pub struct FuzzyPattern {
    chars: Box<[char]>, // secularized characters
    max_nb_holes: usize,
}

impl fmt::Display for FuzzyPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &c in self.chars.iter() {
            f.write_char(c)?
        }
        Ok(())
    }
}

enum MatchSearchResult {
    Perfect(NameMatch), // no need to test other positions
    Some(NameMatch),
    None,
}

fn is_word_separator(c: char) -> bool {
    matches!(c, '_' | ' ' | '-')
}

impl FuzzyPattern {
    /// build a pattern which will later be usable for fuzzy search.
    /// A pattern should be reused
    pub fn from(pat: &str) -> Self {
        let chars = pat
            .chars()
            .map(secular::lower_lay_char)
            .collect::<Vec<char>>()
            .into_boxed_slice();
        let max_nb_holes = match chars.len() {
            1 => 0,
            2 => 1,
            3 => 2,
            4 => 2,
            5 => 3,
            6 => 3,
            7 => 3,
            8 => 4,
            _ => chars.len() * 4 / 7,
        };
        FuzzyPattern {
            chars,
            max_nb_holes,
        }
    }

    /// look for a match starting at a given character
    fn match_starting_at_index(
        &self,
        cand_chars: &[char],
        start_idx: usize, // start index in candidate, in chars
    ) -> MatchSearchResult {
        if cand_chars[start_idx] != self.chars[0] {
            return MatchSearchResult::None;
        }
        let mut pos = smallvec![0; self.chars.len()]; // positions of matching chars in candidate
        let mut nb_holes = 0;
        let mut nb_singled_chars = 0;
        let mut cand_idx = 1 + start_idx;
        if self.chars.len() > 1 {
            let mut group_len = 1;
            let mut pat_idx = 1; // index both in self.chars and pos
            let mut in_hole = false;
            loop {
                if cand_chars[cand_idx] == self.chars[pat_idx] {
                    if in_hole {
                        // we're no more in a hole
                        in_hole = false;
                        if nb_holes > 0 {
                            // it's not the first hole. We see whether it's possible
                            // to join the previous group to the new one
                            let mut can_be_joined = true;
                            for ri in 0..group_len {
                                if cand_chars[cand_idx-ri-1] != self.chars[pat_idx-ri-1] {
                                    can_be_joined = false;
                                    break;
                                }
                            }
                            if can_be_joined {
                                for ri in 0..group_len {
                                    pos[pat_idx-ri-1] = cand_idx-ri-1;
                                }
                            } else {
                                if group_len == 1 {
                                    nb_singled_chars += 1;
                                }
                                nb_holes += 1;
                                group_len = 0;
                            }
                        } else {
                            // first hole
                            nb_holes += 1;
                            group_len = 0;
                        }
                    }
                    pos[pat_idx] = cand_idx;
                    pat_idx += 1;
                    if pat_idx == self.chars.len() {
                        break; // match, finished
                    }
                    cand_idx += 1;
                    group_len += 1;
                } else {
                    // there's a hole
                    if cand_chars.len() - cand_idx <= self.chars.len() - pat_idx {
                        return MatchSearchResult::None;
                    }
                    cand_idx += 1;
                    in_hole = true;
                }
            }
        }
        pos[0] = start_idx;
        let match_len = 1 + cand_idx - start_idx;
        let mut score = BONUS_MATCH;
        score += BONUS_CANDIDATE_LENGTH * (cand_chars.len() as i32);
        score += BONUS_SINGLED_CHAR * (nb_singled_chars as i32);
        score += BONUS_NB_HOLES * (nb_holes as i32);
        score += match_len as i32 * BONUS_MATCH_LENGTH;
        if start_idx == 0 {
            score += BONUS_START + BONUS_START_WORD;
            if cand_chars.len() == self.chars.len() {
                score += BONUS_EXACT;
                return MatchSearchResult::Perfect(NameMatch { score, pos });
            }
        } else {
            let previous = cand_chars[start_idx - 1];
            if is_word_separator(previous) {
                score += BONUS_START_WORD;
                if cand_chars.len() - start_idx == self.chars.len() {
                    return MatchSearchResult::Perfect(NameMatch { score, pos });
                }
            }
        }
        MatchSearchResult::Some(NameMatch { score, pos })
    }

    /// return a match if the pattern can be found in the candidate string.
    /// The algorithm tries to return the best one. For example if you search
    /// "abc" in "ababca-abc", the returned match would be at the end.
    pub fn find(&self, candidate: &str) -> Option<NameMatch> {
        if candidate.len() < self.chars.len() {
            return None;
        }
        let mut cand_chars: Vec<char> = Vec::with_capacity(candidate.len());
        cand_chars.extend(candidate.chars().map(secular::lower_lay_char));
        if cand_chars.len() < self.chars.len() {
            return None;
        }
        let mut best_score = 0;
        let mut best_match: Option<NameMatch> = None;
        let n = cand_chars.len() - self.chars.len();
        for start_idx in 0..=n {
            match self.match_starting_at_index(&cand_chars, start_idx) {
                MatchSearchResult::Perfect(m) => {
                    return Some(m);
                }
                MatchSearchResult::Some(m) => {
                    if m.score > best_score {
                        best_score = m.score;
                        best_match = Some(m);
                    }
                }
                _ => {}
            }
        }
        best_match
    }

    /// compute the score of the best match, in a way mostly similar to `find` but
    /// faster by not storing match positions
    pub fn score_of(&self, candidate: &str) -> Option<i32> {
        self.find(candidate)
            .map(|nm| nm.score)
    }
}

#[cfg(test)]
mod fuzzy_pattern_tests {

    use super::*;

    /// check that the scores of all names are strictly decreasing
    /// (pattern is first tested against itself).
    /// We verify this property with both computation functions.
    fn check_ordering_for(pattern: &str, names: &[&str]) {
        let fp = FuzzyPattern::from(pattern);
        let mut last_score = fp.find(pattern).map(|m| m.score);
        let mut last_name = pattern;
        for name in names {
            let score = fp.find(name).map(|m| m.score);
            assert!(
                score < last_score,
                "score({:?}) should be lower than score({:?}) (using find)",
                name,
                last_name
            );
            last_name = name;
            last_score = score;
        }
    }

    #[test]
    fn check_orderings() {
        check_ordering_for(
            "broot",
            &[
                "a broot",
                "abbroot",
                "abcbroot",
                " abdbroot",
                "1234broot1",
                "12345brrrroooottt",
                "12345brrr roooottt",
                "brot",
            ],
        );
        check_ordering_for(
            "Abc",
            &[
                "abCd",
                "aBdc",
                " abdc",
                " abdbccccc",
                " a b c",
                "nothing",
            ],
        );
        check_ordering_for(
            "réveil",
            &[
                "Réveillon",
                "Réveillons",
                " réveils",
                "πréveil",
                "déréveil",
                " rêves",
            ],
        );
    }
}
