
use {
    std::path::Path,
};

/// the id of a line, starting at 1
/// (0 if not specified)
pub type LineNumber = usize;


/// light information about the currently selected
/// file and maybe line number
#[derive(Debug, Clone, Copy)]
pub struct Selection<'s> {
    pub path: &'s Path,
    pub line: LineNumber, // the line number in the file (0 if none selected)
    pub stype: SelectionType,
}

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
