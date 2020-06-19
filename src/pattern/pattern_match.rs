
/// A NameMatch is a positive result of pattern matching inside
/// a filename or subpath
#[derive(Debug, Clone)]
pub struct NameMatch {
    pub score: i32, // score of the match, guaranteed strictly positive, bigger is better
    pub pos: Vec<usize>, // positions of the matching chars
}

impl Default for NameMatch {
    /// default implementation is for example useful to negate an
    /// absence of match
    fn default() -> Self {
        Self {
            score: 1,
            pos: Vec::new(),
        }
    }
}

