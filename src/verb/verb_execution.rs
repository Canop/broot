
use {
    crate::{
        errors::ConfError,
    },
    super::{
        Internal,
    },
};

/// how a verb must be executed, as described in the configuration
#[derive(Debug, Clone)]
pub enum VerbExecution {

    /// the verb execution is an internal or refers to another verb.
    /// Executions in conf starting with ":" are of this type.
    Internal {
        internal: Internal,
        bang: bool,
    },

    /// the verb execution refers to a command that will be executed by the system
    External(String),
}

impl VerbExecution {
    pub fn try_from(mut s: &str) -> Result<Self, ConfError> {
        Ok(
            if s.starts_with(':') || s.starts_with(' ') {
                s = &s[1..];
                let mut bang = false;
                if s.starts_with('!') {
                    bang = true;
                    s = &s[1..];
                }
                let internal = Internal::try_from(s)?;
                Self::Internal{ internal, bang }
            } else {
                Self::External(s.to_string())
            }
        )
    }
}
