
/// A sort key.
/// A non None sort mode implies only one level of the tree
/// is displayed.
/// When in None mode, paths are alpha sorted
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sort {
    None,
    Count,
    Date,
    Size,
    TypeDirsFirst,
    TypeDirsLast,
}

impl Sort {
    pub fn prevent_deep_display(self) -> bool {
        match self {
            Self::None => false,
            Self::Count => true,
            Self::Date => true,
            Self::Size => true,
            Self::TypeDirsFirst => false,
            Self::TypeDirsLast => false,
        }
    }
}
