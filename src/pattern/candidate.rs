use {
    crate::{
        tree::TreeLine,
    },
    std::{
        path::Path,
    },
};

/// something which can be evaluated by a pattern to produce
/// either a score or a more precise match
#[derive(Debug, Clone, Copy)]
pub struct Candidate<'c> {

    /// path to the file to open if the pattern searches into files
    pub path: &'c Path,

    /// path from the current root
    pub subpath: &'c str,

    /// filename
    pub name: &'c str,

    /// whether the file is regular (ie has a searchable content)
    pub regular_file: bool,
}

impl<'c> Candidate<'c> {
    pub fn from(line: &'c TreeLine) -> Self {
        Self {
            path: &line.path,
            subpath: &line.subpath,
            name: &line.name,
            regular_file: line.is_file(),
        }
    }
}
