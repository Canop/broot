/// The directory from which a relative path is resolved: the closest
/// directory of a base path (itself when it's a directory), or its parent
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PathAnchor {
    #[default]
    Directory,
    Parent,
}
