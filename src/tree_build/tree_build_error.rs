use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
// Note: the error messages are markdown, rendered with termimad by [`ProgramError::report`] or in the status line
pub enum TreeBuildError {
    #[error("File not found: `{}`", .path.display())]
    FileNotFound { path: PathBuf },
    #[error("Task Interrupted")]
    Interrupted,
    #[error("Invalid UTF-8 in `{}`", .path.display())]
    InvalidUtf8 { path: PathBuf },
    #[error("Not a directory: `{}`", .path.display())]
    NotADirectory { path: PathBuf },
    #[error("Not a descendant of the root: `{}`", .path.display())]
    NotARootDescendant { path: PathBuf },
    #[error("Too many matches (max allowed: {max})")]
    TooManyMatches { max: usize },
    #[error("Inconsistent data: {message}")]
    InconsistentData { message: String }, // maybe refresh ?
}
