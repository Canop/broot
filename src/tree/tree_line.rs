use {
    super::*,
    crate::{
        app::{Selection, SelectionType},
        file_sum::FileSum,
        git::LineGitStatus,
    },
    std::{
        cmp::{self, Ord, Ordering, PartialOrd},
        fs,
        path::PathBuf,
    },
};

#[cfg(unix)]
use {std::os::unix::fs::MetadataExt, umask::Mode};

#[cfg(windows)]
use is_executable::IsExecutable;

/// a line in the representation of the file hierarchy
#[derive(Debug, Clone)]
pub struct TreeLine {
    pub left_branchs: Box<[bool]>, // a depth-sized array telling whether a branch pass
    pub depth: u16,
    pub path: PathBuf,
    pub subpath: String,
    pub name: String, // a displayable name - some chars may have been stripped
    pub line_type: TreeLineType,
    pub has_error: bool,
    pub nb_kept_children: usize,
    pub unlisted: usize, // number of not listed children (Dir) or brothers (Pruning)
    pub score: i32,      // 0 if there's no pattern
    pub direct_match: bool,
    pub sum: Option<FileSum>, // None when not measured
    pub metadata: fs::Metadata,
    pub git_status: Option<LineGitStatus>,
}

impl TreeLine {
    pub fn make_displayable_name(name: &str) -> String {
        name.replace('\n', "")
    }
    pub fn is_selectable(&self) -> bool {
        match &self.line_type {
            TreeLineType::Pruning => false,
            _ => true,
        }
    }
    pub fn is_dir(&self) -> bool {
        match &self.line_type {
            TreeLineType::Dir => true,
            TreeLineType::SymLinkToDir(_) => true,
            _ => false,
        }
    }
    pub fn is_file(&self) -> bool {
        match &self.line_type {
            TreeLineType::File => true,
            _ => false,
        }
    }
    pub fn is_of(&self, selection_type: SelectionType) -> bool {
        match selection_type {
            SelectionType::Any => true,
            SelectionType::File => self.is_file(),
            SelectionType::Directory => self.is_dir(),
        }
    }
    pub fn extension(&self) -> Option<&str> {
        regex!(r"\.([^.]+)$")
            .captures(&self.name)
            .and_then(|c| c.get(1))
            .map(|e| e.as_str())
    }
    pub fn selection_type(&self) -> SelectionType {
        use TreeLineType::*;
        match &self.line_type {
            File | SymLinkToFile(_) => SelectionType::File,
            Dir | SymLinkToDir(_) => SelectionType::Directory,
            Pruning => SelectionType::Any, // should not happen today
        }
    }
    pub fn as_selection(&self) -> Selection<'_> {
        Selection {
            path: &self.path,
            stype: self.selection_type(),
            line: 0,
        }
    }
    #[cfg(unix)]
    pub fn mode(&self) -> Mode {
        Mode::from(self.metadata.mode())
    }
    pub fn is_exe(&self) -> bool {
        #[cfg(unix)]
        return self.mode().is_exe();

        #[cfg(windows)]
        return self.path.is_executable();
    }
    /// build and return the absolute targeted path: either self.path or the
    ///  solved canonicalized symlink
    /// (the path may be invalid if the symlink is)
    pub fn target(&self) -> PathBuf {
        match &self.line_type {
            TreeLineType::SymLinkToFile(target) | TreeLineType::SymLinkToDir(target) => {
                let mut target_path = PathBuf::from(target);
                if target_path.is_relative() {
                    target_path = self.path.parent().unwrap().join(target_path);
                }
                if let Ok(canonic) = fs::canonicalize(&target_path) {
                    target_path = canonic;
                }
                target_path
            }
            _ => self.path.clone(),
        }
    }
}
impl PartialEq for TreeLine {
    fn eq(&self, other: &TreeLine) -> bool {
        self.path == other.path
    }
}

impl Eq for TreeLine {}

impl Ord for TreeLine {
    // paths are sorted in a complete ignore case way
    // (A<a<B<b)
    fn cmp(&self, other: &TreeLine) -> Ordering {
        let mut sci = self.path.components();
        let mut oci = other.path.components();
        loop {
            match sci.next() {
                Some(sc) => {
                    match oci.next() {
                        Some(oc) => {
                            let scs = sc.as_os_str().to_string_lossy();
                            let ocs = oc.as_os_str().to_string_lossy();
                            let lower_ordering = scs.to_lowercase().cmp(&ocs.to_lowercase());
                            if lower_ordering != Ordering::Equal {
                                return lower_ordering;
                            }
                            let ordering = scs.cmp(&ocs);
                            if ordering != Ordering::Equal {
                                return ordering;
                            }
                        }
                        None => {
                            return Ordering::Greater;
                        }
                    };
                }
                None => {
                    if oci.next().is_some() {
                        return Ordering::Less;
                    } else {
                        return Ordering::Equal;
                    }
                }
            };
        }
    }
}

impl PartialOrd for TreeLine {
    fn partial_cmp(&self, other: &TreeLine) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}
