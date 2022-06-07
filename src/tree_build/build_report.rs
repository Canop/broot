

/// Information from the builder about the
/// tree operation
///
/// A file is counted at most once here
#[derive(Debug, Clone, Copy, Default)]
pub struct BuildReport {

    /// number of times a gitignore pattern excluded a file
    pub gitignored_count: usize,

    /// number of times a file was excluded because hidden
    /// (this count stays at zero if hidden files are displayed)
    pub hidden_count: usize,

    /// number of errors excluding a file
    pub error_count: usize,

}
