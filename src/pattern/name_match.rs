use {
    super::Pos,
};

/// A NameMatch is a positive result of pattern matching inside
/// a filename or subpath
#[derive(Debug, Clone)]
pub struct NameMatch {
    pub score: i32, // score of the match, guaranteed strictly positive, bigger is better
    pub pos: Pos, // positions of the matching chars
}

impl NameMatch {
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
}

