
/// A NameMatch is a positive result of pattern matching inside
/// a filename or subpath
#[derive(Debug, Clone)]
pub struct NameMatch {
    pub score: i32, // score of the match, guaranteed strictly positive, bigger is better
    pub pos: Vec<usize>, // positions of the matching chars
}

