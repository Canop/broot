/// how a verb is described in the help screen
#[derive(Debug, Clone)]
pub struct VerbDescription {
    pub code: bool,
    pub content: String,
}

impl VerbDescription {
    pub fn from_code(content: String) -> Self {
        Self {
            code: true,
            content,
        }
    }
    pub fn from_text(content: String) -> Self {
        Self {
            code: false,
            content,
        }
    }
}
