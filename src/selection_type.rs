#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SelectionType {
    File,
    Directory,
    Any,
}

impl SelectionType {
    pub fn respects(self, constraint: Self) -> bool {
        constraint == Self::Any || self == constraint
    }
}
