//! a simple fuzzy pattern matcher for filename filtering / sorting.
//! It's not meant for file contents but for small strings (less than 1000 chars)
//!  such as file names.

use {
    super::NameMatch,
    secular,
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
enum ScoreSearchResult {
    Perfect(i32), // no need to test other positions
    Some(i32),
    None,
    NoneToEnd,
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
            4 => 3,
            5 => 3,
            6 => 4,
            7 => 4,
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
        let mut pos: Vec<usize> = vec![]; // positions of matching chars in candidate
        pos.push(start_idx);
        let mut d = 1;
        let mut nb_holes = 0;
        for pat_idx in 1..self.chars.len() {
            let hole_start = d;
            loop {
                let cand_idx = start_idx + d;
                if cand_idx == cand_chars.len() {
                    return MatchSearchResult::None;
                }
                d += 1;
                if cand_chars[cand_idx] == self.chars[pat_idx] {
                    pos.push(cand_idx);
                    break;
                }
            }
            if hole_start + 1 != d {
                // note that there's no absolute guarantee we found the minimal
                // number of holes. The algorithm isn't perfect
                if nb_holes >= self.max_nb_holes {
                    return MatchSearchResult::None;
                }
                nb_holes += 1;
            }
        }
        let mut score = BONUS_MATCH;
        score += BONUS_CANDIDATE_LENGTH * (cand_chars.len() as i32);
        score += BONUS_NB_HOLES * (nb_holes as i32);
        score += d as i32 * BONUS_MATCH_LENGTH;
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

    /// compute the score in the specific case the pattern is of length 1
    fn score_1_char(&self, candidate: &str, pat_chr: char) -> Option<i32> {
        let mut cand_chars = candidate.chars().map(secular::lower_lay_char);
        match cand_chars.next() {
            None => None, // empty candidate: this looks pathological but might be valid
            Some(chr) if pat_chr == chr => {
                let cand_len = cand_chars.count() as i32 + 1;
                let score = BONUS_MATCH
                    + BONUS_START
                    + BONUS_START_WORD
                    + cand_len * BONUS_CANDIDATE_LENGTH
                    + BONUS_MATCH_LENGTH;
                Some(if cand_len == 1 {
                    score + BONUS_EXACT
                } else {
                    score
                })
            }
            Some(chr) => {
                let mut starts_word = is_word_separator(chr);
                while let Some(cand_chr) = cand_chars.next() {
                    if cand_chr == pat_chr {
                        let cand_len = candidate.chars().count() as i32;
                        let score =
                            BONUS_MATCH + cand_len * BONUS_CANDIDATE_LENGTH + BONUS_MATCH_LENGTH;
                        if starts_word {
                            return Some(score + BONUS_START_WORD);
                        } else {
                            // looking for another match after a space
                            for cand_chr in cand_chars {
                                if cand_chr == pat_chr && starts_word {
                                    return Some(score + BONUS_START_WORD);
                                } else {
                                    starts_word = is_word_separator(cand_chr);
                                }
                            }
                            return Some(score);
                        }
                    } else {
                        starts_word = is_word_separator(cand_chr);
                    }
                }
                None
            }
        }
    }

    /// if a match starts at the given char index, return its score.
    /// Return it as "Perfect" if no better match can be found further
    /// in the string.
    /// The pattern is assumed to be of length 2 or more
    fn score_starting_at(
        &self,
        cand_chars: &[char],
        start_idx: usize, // start index in candidate
    ) -> ScoreSearchResult {
        if cand_chars[start_idx] != self.chars[0] {
            return ScoreSearchResult::None;
        }
        let mut d = 1;
        let mut nb_holes = 0;
        for pat_idx in 1..self.chars.len() {
            let hole_start = d;
            loop {
                let cand_idx = start_idx + d;
                if cand_idx == cand_chars.len() {
                    return ScoreSearchResult::NoneToEnd;
                }
                d += 1;
                if cand_chars[cand_idx] == self.chars[pat_idx] {
                    break;
                }
            }
            if hole_start + 1 != d {
                // note that there's no absolute guarantee we found the minimal
                // number of holes. The algorithm isn't perfect
                if nb_holes >= self.max_nb_holes {
                    return ScoreSearchResult::None;
                }
                nb_holes += 1;
            }
        }
        let mut score = BONUS_MATCH
            + BONUS_CANDIDATE_LENGTH * (cand_chars.len() as i32)
            + BONUS_NB_HOLES * (nb_holes as i32)
            + d as i32 * BONUS_MATCH_LENGTH;
        if start_idx == 0 {
            score += BONUS_START + BONUS_START_WORD;
            if cand_chars.len() == self.chars.len() {
                score += BONUS_EXACT;
                return ScoreSearchResult::Perfect(score);
            }
        } else {
            let previous = cand_chars[start_idx - 1];
            if is_word_separator(previous) {
                score += BONUS_START_WORD;
                if cand_chars.len() - start_idx == self.chars.len() {
                    return ScoreSearchResult::Perfect(score);
                }
            }
        }
        ScoreSearchResult::Some(score)
    }

    fn score_n_chars(&self, candidate: &str) -> Option<i32> {
        if candidate.len() < self.chars.len() {
            return None;
        }
        let mut cand_chars: Vec<char> = Vec::with_capacity(candidate.len());
        cand_chars.extend(candidate.chars().map(secular::lower_lay_char));
        if cand_chars.len() < self.chars.len() {
            return None;
        }
        let mut best_score = 0;
        let n = cand_chars.len() - self.chars.len();
        for start_idx in 0..=n {
            match self.score_starting_at(&cand_chars, start_idx) {
                ScoreSearchResult::Perfect(s) => {
                    return Some(s);
                }
                ScoreSearchResult::Some(score) => {
                    if score > best_score {
                        best_score = score;
                    }
                }
                ScoreSearchResult::NoneToEnd => {
                    break;
                }
                _ => {}
            }
        }
        if best_score > 0 {
            Some(best_score)
        } else {
            None
        }
    }

    /// compute the score of the best match, in a way mostly similar to `find` but
    /// faster by not storing match positions
    pub fn score_of(&self, candidate: &str) -> Option<i32> {
        match self.chars.len() {
            1 => self.score_1_char(candidate, self.chars[0]),
            _ => self.score_n_chars(candidate),
        }
    }
}

#[cfg(test)]
mod fuzzy_pattern_tests {

    use super::*;

    #[test]
    fn check_equal_scores() {
        static PATTERNS: &[&str] = &["reveil", "dystroy", "broot", "AB", "z", "é", "év", "a"];
        static NAMES: &[&str] = &[
            " brr ooT",
            "Reveillon",
            "una a",
            "dys",
            "test",
            " a reveil",
            "a rbrroot",
            "Ab",
            "Eve",
            "zeévr",
            "alnékjhz vaoi",
            "jfhf br mleh & é kn rr o hrzpqôùù",
            "ÅΩ",
        ];
        for pattern in PATTERNS {
            let fp = FuzzyPattern::from(pattern);
            for name in NAMES {
                println!("checking pattern {:?} on name {:?}", pattern, name);
                assert_eq!(fp.score_of(name), fp.find(name).map(|m| m.score));
            }
        }
    }

    /// check that the scores of all names are strictly decreasing
    /// (pattern is first tested against itself).
    /// We verify this property with both computation functions.
    fn check_ordering_for(pattern: &str, names: &[&str]) {
        let fp = FuzzyPattern::from(pattern);
        // checking using score_of
        let mut last_score = fp.score_of(pattern);
        let mut last_name = pattern;
        for name in names {
            let score = fp.score_of(name);
            assert!(
                score < last_score,
                "score({:?}) should be lower than score({:?}) (using score_of)",
                name,
                last_name
            );
            last_name = name;
            last_score = score;
        }
        // checking using find
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
            &["abCd", "aBdc", " abdc", " abdbccccc", " a b c", "nothing"],
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
