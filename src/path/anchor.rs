#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PathAnchor {
    #[default]
    Unspecified,
    Parent,
    Directory,
}
