use {
    ahash::AHashSet,
    std::{
        fs,
        io,
        path::{Path, PathBuf},
    },
};

/// The maximum number of symlink hops before giving up.
const MAX_LINK_CHAIN_LENGTH: usize = 128;

/// The type of a line which can be displayed as
/// part of a tree
#[derive(Debug, Clone, PartialEq)]
pub enum TreeLineType {
    File,
    Dir,
    BrokenSymLink(String),
    SymLink {
        direct_target: String,
        final_is_dir: bool,
        final_target: PathBuf,
    },
    Pruning, // a "xxx unlisted" line
}

pub fn read_link(path: &Path) -> io::Result<PathBuf> {
    let mut target = fs::read_link(path)?;
    if target.is_relative() {
        target = path.parent().unwrap().join(&target);
    }
    Ok(target)
}

impl TreeLineType {

    fn resolve(direct_target: &Path) -> io::Result<Self> {
        let mut final_target = direct_target.to_path_buf();
        let mut final_metadata = fs::symlink_metadata(&final_target)?;
        let mut final_ft = final_metadata.file_type();
        let mut final_is_dir = final_ft.is_dir();
        let mut link_chain_length = 0;
        let mut visited = AHashSet::default();
        while final_ft.is_symlink() {
            final_target = read_link(&final_target)?;
            if visited.contains(&final_target) {
                info!(
                    "circular symlink opened by {} and closed by {}",
                    direct_target.display(),
                    final_target.display(),
                );
                return Ok(Self::BrokenSymLink(
                    direct_target.to_string_lossy().into_owned(),
                ));
            }
            visited.insert(final_target.clone());
            final_metadata = fs::symlink_metadata(&final_target)?;
            final_ft = final_metadata.file_type();
            final_is_dir = final_ft.is_dir();
            link_chain_length += 1;
            if link_chain_length > MAX_LINK_CHAIN_LENGTH {
                info!("too long link chain at {}", direct_target.display());
                return Ok(Self::BrokenSymLink(
                    direct_target.to_string_lossy().into_owned(),
                ));
            }
        }
        let direct_target = direct_target.to_string_lossy().into_owned();
        Ok(Self::SymLink {
            direct_target,
            final_is_dir,
            final_target,
        })
    }

    pub fn new(path: &Path, ft: &fs::FileType) -> Self {
        if ft.is_dir() {
            Self::Dir
        } else if ft.is_symlink() {
            if let Ok(direct_target) = read_link(path) {
                Self::resolve(&direct_target)
                    .unwrap_or_else(|_| {
                        Self::BrokenSymLink(direct_target.to_string_lossy().to_string()
                    )})
            } else {
                Self::BrokenSymLink("???".to_string())
            }
        } else {
            Self::File
        }
    }
}
