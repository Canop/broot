#[derive(Debug, Clone, Copy)]
pub enum ExternalExecutionMode {
    /// executed in the parent shell, on broot leaving, using the `br` function
    FromParentShell,

    /// executed on broot leaving, not necessarly in the parent shell
    LeaveBroot,

    /// executed in a sub process without quitting broot
    StayInBroot,
}

impl ExternalExecutionMode {
    pub fn from_shell(self) -> bool {
        match self {
            Self::FromParentShell => true,
            _ => false,
        }
    }
    pub fn leave_broot(self) -> bool {
        match self {
            Self::StayInBroot => false,
            _ => true,
        }
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
