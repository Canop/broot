
use {
    super::{
        AppContext,
        CmdResult,
    },
    crate::{
        errors::ProgramError,
        launchable::Launchable,
        stage::Stage,
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

#[derive(Debug)]
pub enum SelInfo<'s> {
    None,
    One(Selection<'s>),
    More(&'s Stage), // by contract the stage contains at least 2 paths
}

impl SelectionType {
    pub fn respects(self, constraint: Self) -> bool {
        constraint == Self::Any || self == constraint
    }
    pub fn is_respected_by(self, sel_type: Option<Self>) -> bool {
        match (self, sel_type) {
            (Self::File, Some(Self::File)) => true,
            (Self::Directory, Some(Self::Directory)) => true,
            (Self::Any, _) => true,
            _ => false,
        }
    }
    pub fn from(path: &Path) -> Self {
        if path.is_dir() {
            Self::Directory
        } else {
            Self::File
        }
    }
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

impl<'a> SelInfo<'a> {
    //pub fn common_path(&self) -> Option<PathBuf> {
    //    match self {
    //        SelInfo::None => None,
    //        SelInfo::One(sel) => Some(sel.path.into()), // TODO way to avoid this clone ?
    //        SelInfo::More(stage) => Some(longest_common_ancestor(&stage.paths))
    //    }
    //}
    pub fn count_paths(&self) -> usize {
        match self {
            SelInfo::None => 0,
            SelInfo::One(_) => 1,
            SelInfo::More(stage) => stage.len(),
        }
    }
    pub fn common_stype(&self) -> Option<SelectionType> {
        match self {
            SelInfo::None => None,
            SelInfo::One(sel) => Some(sel.stype),
            SelInfo::More(stage) => {
                let stype = SelectionType::from(&stage.paths()[0]);
                for path in stage.paths().iter().skip(1) {
                    if stype != SelectionType::from(path) {
                        return None;
                    }
                }
                Some(stype)
            }
        }
    }
    pub fn as_one_sel(self) -> Option<Selection<'a>> {
        match self {
            SelInfo::One(sel) => Some(sel),
            _ => None,
        }
    }
}
