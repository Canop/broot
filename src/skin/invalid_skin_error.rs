#[derive(thiserror::Error, Debug)]
// Note: the error messages are markdown, rendered with termimad by [`ProgramError::report`] or in the status line
pub enum InvalidSkinError {
    #[error("invalid color: {source}")]
    InvalidColor {
        #[from]
        source: termimad::ParseColorError,
    },
    #[error("`{raw}` is not a valid style attribute")]
    InvalidAttribute { raw: String },
    #[error("grey level must be between 0 and 23 (got {level})")]
    InvalidGreyLevel { level: u8 },
    #[error("Invalid skin style: `{style}`")]
    InvalidStyle { style: String },
    #[error(transparent)]
    InvalidStyleToken {
        #[from]
        source: termimad::ParseStyleTokenError,
    },
}
