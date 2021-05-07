use {
    super::Pos,
    smallvec::SmallVec,
};

/// A NameMatch is a positive result of pattern matching inside
/// a filename or subpath
#[derive(Debug, Clone)]
pub struct NameMatch {
    pub score: i32, // score of the match, guaranteed strictly positive, bigger is better
    pub pos: Pos, // positions of the matching chars
}

impl NameMatch {
    /// wraps any group of matching characters with match_start and match_end
    pub fn wrap(&self, name: &str, match_start: &str, match_end: &str) -> String {
        let mut result = String::new();
        let mut index_in_pos = 0;
        let mut wrapped = false;
        for (idx, c) in name.chars().enumerate() {
            if index_in_pos < self.pos.len() && self.pos[index_in_pos] == idx {
                index_in_pos += 1;
                if !wrapped {
                    result.push_str(match_start);
                    wrapped = true;
                }
            } else if wrapped {
                result.push_str(match_end);
                wrapped = false;
            }
            result.push(c);
        }
        if wrapped {
            result.push_str(match_end);
        }
        result
    }
    // cut the name match in two parts by recomputing the pos
    // arrays
    pub fn cut_after(&mut self, chars_count: usize) -> Self {
        let mut tail = Self {
            score: self.score,
            pos: SmallVec::new(),
        };
        let idx = self.pos.iter().position(|&p| p >= chars_count);
        if let Some(idx) = idx {
            for p in self.pos.drain(idx..) {
                tail.pos.push(p - chars_count);
            }
        }
        tail
    }
}

