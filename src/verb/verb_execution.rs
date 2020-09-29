use {
    super::*,
    std::fmt,
};

/// how a verb must be executed
#[derive(Debug, Clone)]
pub enum VerbExecution {
    /// the verb execution is based on a behavior defined in code in Broot.
    /// Executions in conf starting with ":" are of this type.
    Internal(InternalExecution),

    /// the verb execution refers to a command that will be executed by the system,
    /// outside of broot.
    External(ExternalExecution),

    /// the execution is a sequence similar to what can be given
    /// to broot with --cmd
    Sequence(SequenceExecution),
}

impl fmt::Display for VerbExecution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Internal(ie) => ie.fmt(f),
            Self::External(ee) => ee.exec_pattern.fmt(f),
            Self::Sequence(se) => se.sequence.raw.fmt(f),
        }
    }
}

