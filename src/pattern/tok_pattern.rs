use {
    super::NameMatch,
    secular,
    smallvec::{smallvec, SmallVec},
    std::{
        cmp::Reverse,
        ops::Range,
    },
};

type CandChars = SmallVec<[char; 32]>;

static SEPARATORS: &[char] = &[',', ';'];

// weights used in match score computing
const BONUS_MATCH: i32 = 50_000;
const BONUS_CANDIDATE_LENGTH: i32 = -1; // per char

pub fn norm_chars(s: &str) -> Box<[char]> {
    secular::normalized_lower_lay_string(s)
        .chars()
        .collect::<Vec<char>>()
        .into_boxed_slice()
}

/// a list of tokens we want to find, non overlapping
/// and in any order, in strings
#[derive(Debug, Clone, PartialEq)]
pub struct TokPattern {
    toks: Vec<Box<[char]>>,
    sum_len: usize,
}

// optimizations to test:
// - score_of doesn't need to compute pos, just the ranges
// - try the first tok before creating the matching_ranges vec
//   (the first one is also impler to test)
// - if there's no room for any of the other toks before the
//   first matching tok, start other loops after the first
//   matching range
// But until I add some scoring and multiple positions tests
//  there should not be any perf concerns
// scoring basis ?
// - number of parts of the candidats (separated by / for example)
//   that are touched by a tok ?
// - malus for adjacent ranges
// - bonus for ranges starting just after a separator
// - bonus for order ?
impl TokPattern {

    pub fn new(pattern: &str) -> Self {
        // we accept several separators. The first one
        // we encounter among the possible ones is the
        // separator of the whole. This allows using the
        // other char: In ";ab,er", the comma isn't seen
        // as a separator but as part of a tok
        let sep = pattern.chars()
            .filter(|c| SEPARATORS.contains(c))
            .next();
        let mut toks: Vec<Box<[char]>> = if let Some(sep) = sep {
            pattern.split(sep)
                .filter(|s| s.len() > 0)
                .map(norm_chars)
                .collect()
        } else {
            if pattern.is_empty() {
                Vec::new()
            } else {
                vec![norm_chars(pattern)]
            }
        };
        // we sort the tokens from biggest to smallest
        // because the current algorithm stops at the
        // first match for any tok. Thus it would fail
        // to find "abc,b" in "abcdb" if it looked first
        // at the "b" token
        toks.sort_by_key(|t| Reverse(t.len()));
        let sum_len = toks.iter().map(|s| s.len()).sum();
        Self {
            toks,
            sum_len,
        }
    }

    /// return either None (no match) or a vec whose size is the number
    /// of tokens
    pub fn find_ranges(&self, candidate: &str) -> Option<Vec<Range<usize>>> {
        if candidate.len() < self.sum_len {
            return None;
        }
        let mut cand_chars: CandChars = SmallVec::with_capacity(candidate.len());
        cand_chars.extend(candidate.chars().map(secular::lower_lay_char));
        let mut matching_ranges: Vec<Range<usize>> = Vec::with_capacity(self.toks.len());
        for tok in &self.toks {
            let l = tok.len();
            let matching_range = (0..cand_chars.len()+1-l)
                .map(|idx| idx..idx+l)
                .filter(|r| {
                    &cand_chars[r.start..r.end] == tok.as_ref()
                })
                .filter(|r| {
                    // check we're not intersecting a previous range
                    for pr in &matching_ranges {
                        if pr.contains(&r.start) || pr.contains(&(r.end-1)) {
                            return false;
                        }
                    }
                    true
                })
                .next();
            if let Some(r) = matching_range {
                matching_ranges.push(r);
            } else {
                return None;
            }
        }
        Some(matching_ranges)
    }

    fn score_of_matching(&self, candidate: &str) -> i32 {
        BONUS_MATCH + BONUS_CANDIDATE_LENGTH * candidate.len() as i32
    }

    pub fn find(&self, candidate: &str) -> Option<NameMatch> {
        self.find_ranges(candidate)
            .map(|matching_ranges| {
                let mut pos = smallvec![0; self.sum_len];
                let mut i = 0;
                for r in matching_ranges {
                    for p in r {
                        pos[i] = p;
                        i += 1;
                    }
                }
                pos.sort();
                let score = self.score_of_matching(candidate);
                NameMatch { score, pos }
            })
    }

    /// compute the score of the best match
    pub fn score_of(&self, candidate: &str) -> Option<i32> {
        self.find_ranges(candidate)
            .map(|_| self.score_of_matching(candidate))
    }
}

#[cfg(test)]
mod tok_pattern_tests {

    use {
        super::*,
        crate::pattern::Pos,
    };

    /// check position of the match of the pattern in name
    fn check_pos(pattern: &str, name: &str, pos: &str) {
        let pat = TokPattern::new(pattern);
        let match_pos = pat.find(name).unwrap().pos;
        let target_pos: Pos = pos.chars()
            .enumerate()
            .filter(|(_, c)| *c=='^')
            .map(|(i, _)| i)
            .collect();
        assert_eq!(match_pos, target_pos);
    }

    #[test]
    fn check_match_pos() {
        check_pos(
            "m,",
            "miaou",
            "^   ",
        );
        check_pos(
            "bat",
            "cabat",
            "  ^^^",
        );
        check_pos(
            ";ba",
            "babababaaa",
            "^^        ",
        );
        check_pos(
            "ba,ca",
            "bababacaa",
            "^^    ^^ ",
        );
        check_pos(
            "sub,doc,2",
            "/home/user/path2/subpath/Documents/",
            "               ^ ^^^     ^^^",
        );
        check_pos(
            "ab,abc",
            "0123/abc/ab/cdg",
            "     ^^^ ^^    ",
        );
    }

    fn check_match(pattern: &str, name: &str, do_match: bool) {
        assert_eq!(
            TokPattern::new(pattern).find(name).is_some(),
            do_match,
        );
    }

    #[test]
    fn test_separators() {
        let a = TokPattern::new("ab;cd;ef");
        let b = TokPattern::new("ab,cd,ef");
        assert_eq!(a, b);
        let a = TokPattern::new(",ab;cd;ef");
        assert_eq!(a.toks.len(), 1);
        assert_eq!(a.toks[0].len(), 8);
        let a = TokPattern::new(";ab,cd,ef;");
        assert_eq!(a.toks.len(), 1);
        assert_eq!(a.toks[0].len(), 8);
    }

    #[test]
    fn test_match() {
        check_match("mia", "android/phonegap", false);
        check_match("mi", "a", false);
        check_match("mi", "π", false);
        check_match("mi", "miaou/a", true);
    }

    #[test]
    fn test_tok_repetitions() {
        check_match("sub", "rasub", true);
        check_match("sub,sub", "rasub", false);
        check_match("sub,sub", "rasubandsub", true);
        check_match("sub,sub,sub", "rasubandsub", false);
        check_match("ghi,abc,def,ccc", "abccc/Defghi", false);
        check_match("ghi,abc,def,ccc", "abcccc/Defghi", true);
    }

}
