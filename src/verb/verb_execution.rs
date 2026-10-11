use {
    super::*,
    crate::conf::ConfError,
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

    /// like `External` but the command line is run through a shell (`sh -c` /
    /// `cmd /C`) instead of executing a single program, so that shell features
    /// like `&&`, `;`, pipes and redirections work.
    ShellCommand(ExternalExecution),

    /// the execution is a sequence similar to what can be given
    /// to broot with --cmd
    Sequence(SequenceExecution),
}

impl VerbExecution {
    /// Parse the arg defs of the pattern(s) building the command
    /// (not including the working dir)
    pub fn arg_defs(&self) -> Result<Vec<ExecArgDef>, ConfError> {
        match self {
            Self::Internal(ie) => match &ie.arg {
                Some(arg) => parse_exec_arg_defs(arg),
                None => Ok(Vec::new()),
            },
            Self::External(ee) | Self::ShellCommand(ee) => ee.exec_pattern.arg_defs(),
            Self::Sequence(se) => parse_exec_arg_defs(&se.sequence.raw),
        }
    }
}

// This implementation builds a string used for description (eg in help)
impl fmt::Display for VerbExecution {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Self::Internal(ie) => ie.fmt(f),
            Self::External(ee) | Self::ShellCommand(ee) => ee.exec_pattern.fmt(f),
            Self::Sequence(se) => se.sequence.raw.fmt(f),
        }
    }
}
