use {
    super::bid::BId,
    crate::{
        app::AppContext,
        errors::TreeBuildError,
        git::GitIgnoreChain,
        path::{normalize_path, Directive, SpecialHandling},
        tree::*,
    },
    id_arena::Arena,
    std::{
        fs,
        io,
        path::PathBuf,
        result::Result,
    },
};

/// like a tree line, but with the info needed during the build
/// This structure isn't usable independently from the tree builder
pub struct BLine {
    pub parent_id: Option<BId>,
    pub path: PathBuf,
    pub depth: u16,
    pub subpath: String,
    pub name: String,
    pub file_type: fs::FileType,
    pub children: Option<Vec<BId>>, // sorted and filtered
    pub next_child_idx: usize,      // index for iteration, among the children
    pub has_error: bool,
    pub has_match: bool,
    pub direct_match: bool,
    pub score: i32,
    pub nb_kept_children: i32, // used during the trimming step
    pub git_ignore_chain: GitIgnoreChain,
    pub special_handling: SpecialHandling,
}

impl BLine {
    /// a special constructor, checking nothing
    pub fn from_root(
        blines: &mut Arena<BLine>,
        path: PathBuf,
        git_ignore_chain: GitIgnoreChain,
        _options: &TreeOptions,
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
                subpath: String::new(),
                children: None,
                next_child_idx: 0,
                file_type,
                has_error: false,
                has_match: true,
                direct_match: false,
                score: 0,
                nb_kept_children: 0,
                git_ignore_chain,
                special_handling: Default::default(),
            }))
        } else {
            Err(TreeBuildError::FileNotFound {
                path: format!("{path:?}"),
            })
        }
    }
    /// execute read_dir either on the link if we're a link,
    /// or on the path otherwise.
    ///
    /// Assume the can_enter check has already be done.
    pub(crate) fn read_dir(&self) -> io::Result<fs::ReadDir> {
        if self.file_type.is_symlink() {
            if let Ok(target) = fs::read_link(&self.path) {
                let mut target_path = PathBuf::from(&target);
                if target_path.is_relative() {
                    target_path = self.path.parent().unwrap().join(target_path);
                    target_path = normalize_path(target_path);
                }
                return fs::read_dir(&target_path);
            }
        }
        fs::read_dir(&self.path)
    }
    /// tell whether we should list the children of the present line
    pub fn can_enter(&self) -> bool {
        if self.file_type.is_dir() && self.special_handling.list != Directive::Never {
            return true;
        }
        if self.special_handling.list == Directive::Always {
            // we must check we're a link to a directory
            if self.file_type.is_symlink() {
                if let Ok(target) = fs::read_link(&self.path) {
                    let mut target_path = PathBuf::from(&target);
                    if target_path.is_relative() {
                        target_path = self.path.parent().unwrap().join(target_path);
                    }
                    if let Ok(target_metadata) = fs::symlink_metadata(&target_path) {
                        if target_metadata.file_type().is_dir() {
                            if self.path.starts_with(target_path) {
                                debug!("not entering link because it's a parent"); // lets's not cycle
                            } else {
                                debug!("entering {:?} because of special path rule", &self.path);
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }
    pub fn to_tree_line(&self, bid: BId, con: &AppContext) -> std::io::Result<TreeLine> {
        let has_error = self.has_error;
        let line_type = TreeLineType::new(&self.path, &self.file_type);
        let unlisted = if let Some(children) = &self.children {
            // number of not listed children
            children.len() - self.next_child_idx
        } else {
            0
        };
        let metadata = fs::symlink_metadata(&self.path)?;
        let subpath = self.subpath.replace('\n', "");
        let name = self.name.replace('\n', "");
        let icon = con.icons.as_ref()
            .map(|icon_plugin| {
                let extension = TreeLine::extension_from_name(&name);
                let double_extension = extension
                    .and_then(|_| TreeLine::double_extension_from_name(&name));
                icon_plugin.get_icon(
                    &line_type,
                    &name,
                    double_extension,
                    extension,
                )
            });

        Ok(TreeLine {
            bid,
            parent_bid: self.parent_id,
            left_branches: vec![false; self.depth as usize].into_boxed_slice(),
            depth: self.depth,
            icon,
            name,
            subpath,
            path: self.path.clone(),
            line_type,
            has_error,
            nb_kept_children: self.nb_kept_children as usize,
            unlisted,
            score: self.score,
            direct_match: self.direct_match,
            sum: None,
            metadata,
            git_status: None,
        })
    }
}
