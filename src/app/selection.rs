use {
    super::{
        AppContext,
        CmdResult,
    },
    crate::{
        errors::ProgramError,
        launchable::Launchable,
    },
    std::{
        fs::OpenOptions,
        io::Write,
        path::Path,
    },
};

/// the id of a line, starting at 1
/// (0 if not specified)
pub type LineNumber = usize;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SelectionType {
    File,
    Directory,
    Any,
}

/// light information about the currently selected
/// file and maybe line number
#[derive(Debug, Clone, Copy)]
pub struct Selection<'s> {
    pub path: &'s Path,
    pub line: LineNumber, // the line number in the file (0 if none selected)
    pub stype: SelectionType,
    pub is_exe: bool,
}

impl SelectionType {
    #[must_use]
    pub fn respects(
        self,
        constraint: Self,
    ) -> bool {
        constraint == Self::Any || self == constraint
    }
    #[must_use]
    pub fn is_respected_by(
        self,
        sel_type: Option<Self>,
    ) -> bool {
        match (self, sel_type) {
            (Self::File, Some(Self::File)) => true,
            (Self::Directory, Some(Self::Directory)) => true,
            (Self::Any, _) => true,
            _ => false,
        }
    }
    #[must_use]
    pub fn from(path: &Path) -> Self {
        if path.is_dir() {
            Self::Directory
        } else {
            Self::File
        }
    }
}

impl Selection<'_> {
    /// build a `CmdResult` with a launchable which will be used to
    /// open the relevant file the best possible way
    pub fn to_opener(
        self,
        con: &AppContext,
    ) -> Result<CmdResult, ProgramError> {
        Ok(if self.is_exe {
            let path = self.path.to_string_lossy().to_string();
            if let Some(export_path) = &con.launch_args.outcmd {
                // broot was launched as br, we can launch the executable from the shell
                let f = OpenOptions::new().append(true).open(export_path)?;
                writeln!(&f, "{path}")?;
                CmdResult::Quit
            } else {
                CmdResult::from(Launchable::program(
                    vec![path],
                    None, // we don't set the working directory
                    true, // we switch the terminal during execution
                    con,
                )?)
            }
        } else {
            CmdResult::from(Launchable::opener(self.path.to_path_buf()))
        })
    }
}
