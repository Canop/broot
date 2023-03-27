#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExternalExecutionMode {
    /// executed in the parent shell, on broot leaving, using the `br` function
    FromParentShell,

    /// executed on broot leaving, not necessarly in the parent shell
    LeaveBroot,

    /// executed in a sub process without quitting broot, while restoring the terminal
    StayInBrootTerminal,

    /// executed in a sub process without quitting broot, without restoring the terminal
    StayInBrootGUI,
}

impl ExternalExecutionMode {
    pub fn is_from_shell(self) -> bool {
        matches!(self, Self::FromParentShell)
    }
    pub fn is_leave_broot(self) -> bool {
        !matches!(self, Self::StayInBrootTerminal)
    }

    pub fn from_conf(
        from_shell: Option<bool>,  // default is false
        leave_broot: Option<bool>, // default is true
        is_terminal: Option<bool>, // default is true
    ) -> Self {
        if from_shell.unwrap_or(false) {
            Self::FromParentShell
        } else if leave_broot.unwrap_or(true) {
            Self::LeaveBroot
        } else if is_terminal.unwrap_or(true) {
            Self::StayInBrootTerminal
        } else {
            Self::StayInBrootGUI
        }
    }
}
