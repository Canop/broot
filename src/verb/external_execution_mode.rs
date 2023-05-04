#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExternalExecutionMode {
    /// executed in the parent shell, on broot leaving, using the `br` function
    FromParentShell,

    /// executed on broot leaving, not necessarily in the parent shell
    LeaveBroot,

    /// executed in a sub process without quitting broot
    StayInBroot,
}

impl ExternalExecutionMode {
    pub fn is_from_shell(self) -> bool {
        matches!(self, Self::FromParentShell)
    }
    pub fn is_leave_broot(self) -> bool {
        !matches!(self, Self::StayInBroot)
    }

    pub fn from_conf(
        from_shell: Option<bool>,  // default is false
        leave_broot: Option<bool>, // default is true
    ) -> Self {
        if from_shell.unwrap_or(false) {
            Self::FromParentShell
        } else if leave_broot.unwrap_or(true) {
            Self::LeaveBroot
        } else {
            Self::StayInBroot
        }
    }
}
