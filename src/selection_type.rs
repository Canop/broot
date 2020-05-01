#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SelectionType {
    File,
    Directory,
    Any,
}

impl SelectionType {
    pub fn respects(self, constraint: Self) -> bool {
        constraint == Self::Any || self == constraint
        //use SelectionType::*;
        //match (self, constraint) {
        //    (_, Undefined) => true, // no constraint
        //    (File, File) => true,
        //    (Directory, Directory) => true
        //}
    }
}
