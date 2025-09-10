use {
    super::*,
    crate::{
        app::{
            AppContext,
            Selection,
            SelectionType,
        },
        errors::TreeBuildError,
        file_sum::FileSum,
        git::LineGitStatus,
    },
    lazy_regex::regex_captures,
    std::{
        fs,
        path::{
            Path,
            PathBuf,
        },
    },
};

#[cfg(unix)]
use {
    std::os::unix::fs::MetadataExt,
    umask::Mode,
};

#[cfg(windows)]
use is_executable::IsExecutable;

pub type TreeLineId = usize;

/// a line in the representation of the file hierarchy
#[derive(Debug, Clone)]
pub struct TreeLine {
    pub id: TreeLineId,
    pub parent_id: Option<TreeLineId>,
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

pub struct TreeLineBuilder {
    pub path: PathBuf,
    pub subpath: String,
    pub id: TreeLineId,
    pub parent_id: Option<TreeLineId>,
    pub depth: u16,
    pub unlisted: usize,
    pub nb_kept_children: usize,
    pub has_error: bool,
    pub score: i32,
    pub direct_match: bool,
}

impl TreeLineBuilder {
    pub fn build(
        self,
        con: &AppContext,
    ) -> Result<TreeLine, TreeBuildError> {
        let Self {
            path,
            subpath,
            id,
            parent_id,
            depth,
            unlisted,
            nb_kept_children,
            has_error,
            score,
            direct_match,
        } = self;
        let metadata =
            fs::symlink_metadata(&path).map_err(
                |_| TreeBuildError::FileNotFound {
                    path: path.to_string_lossy().to_string(),
                },
            )?;
        let line_type = TreeLineType::new(&path, metadata.file_type());
        let name = path
            .file_name()
            .and_then(|os_str| os_str.to_str())
            .unwrap_or("")
            .replace('\n', "");
        let icon = con.icons.as_ref().map(|icon_plugin| {
            let extension = TreeLine::extension_from_name(&name);
            let double_extension =
                extension.and_then(|_| TreeLine::double_extension_from_name(&name));
            icon_plugin.get_icon(
                &line_type,
                &name,
                double_extension,
                extension,
            )
        });

        Ok(TreeLine {
            id,
            parent_id,
            left_branches: vec![false; depth as usize].into_boxed_slice(),
            depth,
            icon,
            name,
            subpath,
            path,
            line_type,
            has_error,
            nb_kept_children,
            unlisted,
            score,
            direct_match,
            sum: None,
            metadata,
            git_status: None,
        })
    }
}

impl TreeLine {
    pub fn double_extension_from_name(name: &str) -> Option<&str> {
        regex_captures!(r"\.([^.]+\.[^.]+)", name).map(|(_, de)| de)
    }

    pub fn extension_from_name(name: &str) -> Option<&str> {
        regex_captures!(r"\.([^.]+)$", name).map(|(_, ext)| ext)
    }

    pub fn is_selectable(&self) -> bool {
        !matches!(
            &self.line_type,
            TreeLineType::Pruning
        )
    }
    pub fn is_dir(&self) -> bool {
        match &self.line_type {
            TreeLineType::Dir => true,
            TreeLineType::SymLink {
                final_is_dir,
                ..
            } if *final_is_dir => true,
            _ => false,
        }
    }
    pub fn is_file(&self) -> bool {
        matches!(
            &self.line_type,
            TreeLineType::File
        )
    }
    pub fn is_of(
        &self,
        selection_type: SelectionType,
    ) -> bool {
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
            SymLink {
                final_is_dir,
                ..
            } => {
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
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    pub fn device_id(&self) -> lfs_core::DeviceId {
        self.metadata.dev().into()
    }
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    pub fn mount(&self) -> Option<lfs_core::Mount> {
        use crate::filesystems::*;
        let mut mount_list = MOUNTS.lock().unwrap();
        if mount_list.load().is_ok() {
            mount_list.get_by_device_id(self.metadata.dev().into()).cloned()
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
            TreeLineType::SymLink {
                final_target,
                ..
            } => final_target,
            _ => &self.path,
        }
    }
    pub fn unprune(&mut self) {
        self.line_type = TreeLineType::new(
            &self.path,
            self.metadata.file_type(),
        );
        self.name = self.path.file_name().map_or_else(
            || "???".to_string(),
            |n| n.to_string_lossy().to_string(),
        );
    }
}
