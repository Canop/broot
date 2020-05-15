
/// The type of a line which can be displayed as
/// part of a tree
#[derive(Debug, Clone, PartialEq)]
pub enum TreeLineType {
    File,
    Dir,
    SymLinkToDir(String),
    SymLinkToFile(String), // (to file or to symlink)
    Pruning,               // a "xxx unlisted" line
}
