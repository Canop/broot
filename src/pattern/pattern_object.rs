

/// on what the search applies
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PatternObject {
    FileName,
    FileSubpath,
    FileContent,
}
