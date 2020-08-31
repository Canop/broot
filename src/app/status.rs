
/// the status contains information written on the grey line
///  near the bottom of the screen
#[derive(Debug, Clone)]
pub struct Status {
    pub message: String, // markdown
    pub error: bool,     // is the current message an error?
}

impl Status {
    pub fn new<S: Into<String>>(message: S, error: bool) -> Status {
        Self {
            message: message.into(),
            error,
        }
    }

    pub fn from_message<S: Into<String>>(message: S) -> Status {
        Self {
            message: message.into(),
            error: false,
        }
    }

    pub fn from_error<S: Into<String>>(message: S) -> Status {
        Self {
            message: message.into(),
            error: true,
        }
    }
}
