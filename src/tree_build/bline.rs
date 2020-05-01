use {
    super::bid::BId,
    crate::{
        errors::TreeBuildError,
        flat_tree::{LineType, TreeLine},
        git::GitIgnoreChain,
    },
    id_arena::Arena,
    std::{fs, path::PathBuf, result::Result},
};

/// like a tree line, but with the info needed during the build
/// This structure isn't usable independantly from the tree builder
pub struct BLine {
    pub parent_id: Option<BId>,
    pub path: PathBuf,
    pub depth: u16,
    pub name: String,
    pub file_type: fs::FileType,
    pub children: Option<Vec<BId>>, // sorted and filtered
    pub next_child_idx: usize,      // index for iteration, among the children
    pub has_error: bool,
    pub has_match: bool,
    pub score: i32,
    pub nb_kept_children: i32, // used during the trimming step
    pub git_ignore_chain: GitIgnoreChain,
}

impl BLine {
    /// a special constructor, checking nothing
    pub fn from_root(
        blines: &mut Arena<BLine>,
        path: PathBuf,
        git_ignore_chain: GitIgnoreChain,
    ) -> Result<BId, TreeBuildError> {
        let name = match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => String::from("???"), // should not happen
        };
        if let Ok(md) = fs::metadata(&path) {
            let file_type = md.file_type();
            Ok(blines.alloc(BLine {
                parent_id: None,
                path,
                depth: 0,
                name,
                children: None,
                next_child_idx: 0,
                file_type,
                has_error: false,
                has_match: true,
                score: 0,
                nb_kept_children: 0,
                git_ignore_chain,
            }))
        } else {
            Err(TreeBuildError::FileNotFound {
                path: format!("{:?}", path),
            })
        }
    }
    pub fn to_tree_line(&self) -> std::io::Result<TreeLine> {
        let mut has_error = self.has_error;
        let line_type = if self.file_type.is_dir() {
            LineType::Dir
        } else if self.file_type.is_symlink() {
            if let Ok(target) = fs::read_link(&self.path) {
                let target = target.to_string_lossy().into_owned();
                let mut target_path = PathBuf::from(&target);
                if target_path.is_relative() {
                    target_path = self.path.parent().unwrap().join(target_path)
                }
                if let Ok(target_metadata) = fs::symlink_metadata(&target_path) {
                    if target_metadata.file_type().is_dir() {
                        LineType::SymLinkToDir(target)
                    } else {
                        LineType::SymLinkToFile(target)
                    }
                } else {
                    has_error = true;
                    LineType::SymLinkToFile(target)
                }
            } else {
                has_error = true;
                LineType::SymLinkToFile(String::from("????"))
            }
        } else {
            LineType::File
        };
        let unlisted = if let Some(children) = &self.children {
            // number of not listed children
            children.len() - self.next_child_idx
        } else {
            0
        };
        let metadata = fs::symlink_metadata(&self.path)?;
        Ok(TreeLine {
            left_branchs: vec![false; self.depth as usize].into_boxed_slice(),
            depth: self.depth,
            name: self.name.to_string(),
            path: self.path.clone(),
            line_type,
            has_error,
            nb_kept_children: self.nb_kept_children as usize,
            unlisted,
            score: self.score,
            size: None,
            metadata,
            git_status: None,
        })
    }
}
