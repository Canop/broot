use lazy_regex::regex;

/// Error which can be raised when parsing a pattern the user typed
#[derive(thiserror::Error, Debug)]
// Note: the error messages are markdown, rendered with termimad by [`ProgramError::report`] or in the status line
pub enum PatternError {
    #[error("Invalid search mode: `{mode}`")]
    InvalidMode { mode: String },
    #[error(
        "Invalid Regular Expression: {}",
        .source.to_string().lines().last().unwrap_or(""),
    )]
    InvalidRegex {
        #[from]
        source: regex::Error,
    },
    #[error("Unknown regular expression flag: `{bad}`")]
    UnknownRegexFlag { bad: char },
}
