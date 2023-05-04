use {
    super::*,
    crate::{
        app::{Selection, SelectionType},
        file_sum::FileSum,
        git::LineGitStatus,
        tree_build::BId,
    },
    lazy_regex::regex_captures,
    std::{
        fs,
        path::{Path, PathBuf},
    },
};

#[cfg(unix)]
use {std::os::unix::fs::MetadataExt, umask::Mode};

#[cfg(windows)]
use is_executable::IsExecutable;

/// a line in the representation of the file hierarchy
#[derive(Debug, Clone)]
pub struct TreeLine {
    pub bid: BId,
    pub parent_bid: Option<BId>,
    pub left_branches: Box<[bool]>, // a depth-sized array telling whether a branch pass
    pub depth: u16,
    pub path: PathBuf,
    pub subpath: String,
    pub icon: Option<char>,
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

    pub fn double_extension_from_name(name: &str) -> Option<&str> {
        regex_captures!(r"\.([^.]+\.[^.]+)", name)
            .map(|(_, de)| de)
    }

    pub fn extension_from_name(name: &str) -> Option<&str> {
        regex_captures!(r"\.([^.]+)$", name)
            .map(|(_, ext)| ext)
    }

    pub fn is_selectable(&self) -> bool {
        !matches!(&self.line_type, TreeLineType::Pruning)
    }
    pub fn is_dir(&self) -> bool {
        match &self.line_type {
            TreeLineType::Dir => true,
            TreeLineType::SymLink { final_is_dir, .. } if *final_is_dir => true,
            _ => false,
        }
    }
    pub fn is_file(&self) -> bool {
        matches!(&self.line_type, TreeLineType::File)
    }
    pub fn is_of(&self, selection_type: SelectionType) -> bool {
        match selection_type {
            SelectionType::Any => true,
            SelectionType::File => self.is_file(),
            SelectionType::Directory => self.is_dir(),
        }
    }
    pub fn extension(&self) -> Option<&str> {
        Self::extension_from_name(&self.name)
    }
    pub fn selection_type(&self) -> SelectionType {
        use TreeLineType::*;
        match &self.line_type {
            File => SelectionType::File,
            Dir | BrokenSymLink(_) => SelectionType::Directory,
            SymLink { final_is_dir, .. } => {
                if *final_is_dir {
                    SelectionType::Directory
                } else {
                    SelectionType::File
                }
            }
            Pruning => SelectionType::Any, // should not happen today
        }
    }
    pub fn as_selection(&self) -> Selection<'_> {
        Selection {
            path: &self.path,
            stype: self.selection_type(),
            is_exe: self.is_exe(),
            line: 0,
        }
    }
    #[cfg(unix)]
    pub fn mode(&self) -> Mode {
        Mode::from(self.metadata.mode())
    }
    #[cfg(unix)]
    pub fn device_id(&self) -> lfs_core::DeviceId {
        self.metadata.dev().into()
    }
    #[cfg(unix)]
    pub fn mount(&self) -> Option<lfs_core::Mount> {
        use crate::filesystems::*;
        let mut mount_list = MOUNTS.lock().unwrap();
        if mount_list.load().is_ok() {
            mount_list
                .get_by_device_id(self.metadata.dev().into())
                .cloned()
        } else {
            None
        }
    }
    pub fn is_exe(&self) -> bool {
        #[cfg(unix)]
        return self.mode().is_exe();

        #[cfg(windows)]
        return self.path.is_executable();
    }
    /// build and return the absolute targeted path: either self.path or the
    ///  solved canonicalized symlink
    pub fn target(&self) -> &Path {
        match &self.line_type {
            TreeLineType::SymLink { final_target, .. } => final_target,
            _ => &self.path,
        }
    }
}


