use {
    super::*,
    crate::errors::ConfError,
    std::fmt,
};

/// A verb execution definition based on an internal
#[derive(Debug, Clone)]
pub struct InternalExecution {
    /// the internal to use
    pub internal: Internal,

    /// whether to open the resulting state in a new panel
    /// instead of the current ones
    pub bang: bool,

    /// arguments
    ///
    /// For example it's `"~"` when a verb execution is `":!focus ~"`
    /// and it's execution: `"OO {v} {file}\n"` when the verb execution is
    /// `":write_output OO {v} {file}\n"`
    pub arg: Option<String>,
}

impl InternalExecution {
    pub fn from_internal(internal: Internal) -> Self {
        Self {
            internal,
            bang: false,
            arg: None,
        }
    }
    pub fn from_internal_bang(
        internal: Internal,
        bang: bool,
    ) -> Self {
        Self {
            internal,
            bang,
            arg: None,
        }
    }
    pub fn try_from(invocation_str: &str) -> Result<Self, ConfError> {
        let invocation = VerbInvocation::from(invocation_str);
        let internal = Internal::try_from(&invocation.name)?;
        Ok(Self {
            internal,
            bang: invocation.bang,
            arg: invocation.args,
        })
    }
    pub fn needs_selection(&self) -> bool {
        self.internal.needs_selection(&self.arg)
    }
    fn has_merging_arg(&self) -> bool {
        let Some(args) = &self.arg else {
            return false;
        };
        for capture in ARG_DEF_GROUP.captures_iter(args) {
            let arg_def = VerbArgDef::from_capture(&capture);
            for flag in &arg_def.flags {
                if flag.is_merging() {
                    return true;
                }
            }
        }
        false
    }
    /// Tell whether, in case of a multiple selection, the command should be executed once per
    /// selection or once for all selections together (meaning the selections will be merged).
    pub fn coarity(&self) -> CommandCoarity {
        if self.has_merging_arg() {
            CommandCoarity::Merged
        } else {
            CommandCoarity::PerSelection
        }
    }
}
impl fmt::Display for InternalExecution {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(f, ":{}", self.internal.name())?;
        if self.bang {
            write!(f, "!")?;
        }
        if let Some(arg) = &self.arg {
            write!(f, " {arg}")?;
        }
        Ok(())
    }
}
