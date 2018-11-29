//! a trivial fuzzy pattern matcher for filename filtering / sorting

// weights used in match score computing
// TODO use ML to set those weights
//  (just kidding, nobody cares)
const BONUS_MATCH: i32 = 5000;
const BONUS_SAME_CASE: i32 = 10;
const BONUS_START: i32 = 25;
const BONUS_LENGTH: i32 = -1; // per char of length

#[derive(Debug)]
pub struct Pattern {
    chars: Box<[char]>,
}

#[derive(Debug)]
pub struct Match {
    pub score: i32,  // score of the match, guaranteed strictly positive, bigger is better
    pos: Vec<usize>, // positions of the matching chars
}

impl Pattern {
    pub fn from(pat: &str) -> Pattern {
        let chars: Vec<char> = pat.chars().collect();
        let chars = chars.into_boxed_slice();
        Pattern { chars }
    }
    // this very matching function only looks for the first possible match
    //  which is usually the best one, but not always
    pub fn test(&self, candidate: &str) -> Option<Match> {
        if candidate.is_empty() || self.chars.is_empty() {
            return None;
        }
        let mut score: i32 = BONUS_MATCH;
        let mut cand_iter = candidate.chars().enumerate();
        let mut pos: Vec<usize> = vec![]; // positions of matching chars in candidate
        for &pat_char in self.chars.iter() {
            loop {
                if let Some((cand_idx, cand_char)) = cand_iter.next() {
                    // TODO give bonus for uppercases
                    // TODO bonus for adjacency
                    if pat_char == cand_char {
                        score += BONUS_SAME_CASE;
                        pos.push(cand_idx);
                        break;
                    } else if pat_char.to_ascii_lowercase() == cand_char.to_ascii_lowercase() {
                        pos.push(cand_idx);
                        break;
                    }
                } else {
                    return None;
                }
            }
        }
        if pos[0] == 0 {
            score += BONUS_START;
        }
        score += ((pos[pos.len() - 1] - pos[0]) as i32) * BONUS_LENGTH;
        Some(Match { score, pos })
    }
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
