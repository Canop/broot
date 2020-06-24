
/// A non None sort mode implies only one level of the tree
/// is displayed.
/// When in None mode, paths are alpha sorted
#[derive(Debug, Clone, Copy)]
pub enum Sort {
    None,
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
    pub fn is_date(self) -> bool {
        match self {
            Sort::Date => true,
            _ => false,
        }
    }
    pub fn is_size(self) -> bool {
        match self {
            Sort::Size => true,
            _ => false,
        }
    }
}
