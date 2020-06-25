
/// A non None sort mode implies only one level of the tree
/// is displayed.
/// When in None mode, paths are alpha sorted
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sort {
    None,
    Count,
    Date,
    Size,
}

impl Sort {
    pub fn is_some(self) -> bool {
        match self {
            Sort::None => false,
            _ => true,
        }
    }
}
