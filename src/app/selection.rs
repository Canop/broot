
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

impl SelectionType {
    pub fn respects(self, constraint: Self) -> bool {
        constraint == Self::Any || self == constraint
    }
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

impl Selection<'_> {

    /// build a CmdResult with a launchable which will be used to
    ///  1/ quit broot
    ///  2/ open the relevant file the best possible way
    pub fn to_opener(
        self,
        con: &AppContext,
    ) -> Result<CmdResult, ProgramError> {
        Ok(if self.is_exe {
            let path = self.path.to_string_lossy().to_string();
            if let Some(export_path) = &con.launch_args.cmd_export_path {
                // broot was launched as br, we can launch the executable from the shell
                let f = OpenOptions::new().append(true).open(export_path)?;
                writeln!(&f, "{}", path)?;
                CmdResult::Quit
            } else {
                CmdResult::from(Launchable::program(
                    vec![path],
                    None, // we don't set the working directory
                    con,
                )?)
            }
        } else {
            CmdResult::from(Launchable::opener(self.path.to_path_buf()))
        })
    }

}
