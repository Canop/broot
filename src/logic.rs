
/// a general purpose Trilean
#[derive(Debug, Clone, Copy)]
pub enum Trilean {
    Unknown,
    True,
    False,
}
impl Trilean {
    pub fn is_true(self) -> bool {
        match self {
            Trilean::True => true,
            _ => false,
        }
    }
    pub fn is_false(self) -> bool {
        match self {
            Trilean::False => true,
            _ => false,
        }
    }
}
