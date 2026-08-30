mod diff;
mod ignore;
mod status;
mod status_computer;

pub use {
    diff::{
        DiffLine,
        DiffLineKind,
        FileDiff,
        FileDiffContent,
        Hunk,
        diff_worktree_vs_head,
    },
    ignore::{
        IgnoreChain,
        Ignorer,
    },
    status::{
        LineGitStatus,
        LineStatusComputer,
        TreeGitStatus,
    },
    status_computer::{
        clear_status_computer_cache,
        get_tree_status,
    },
};

use std::path::{
    Path,
    PathBuf,
};

/// Find the git repository containing the given path (a file or a directory)
pub fn discover(path: &Path) -> Option<gix::Repository> {
    let dir = if path.is_dir() { path } else { path.parent()? };
    gix::discover(dir).ok()
}

/// Return the root of the working directory of the git repository
/// containing the given path, if any
pub fn workdir(path: &Path) -> Option<PathBuf> {
    discover(path)?.workdir().map(Path::to_path_buf)
}

/// return the closest parent (or self) containing a .git file
pub fn closest_repo_dir(mut path: &Path) -> Option<PathBuf> {
    loop {
        let c = path.join(".git");
        if c.exists() {
            return Some(path.to_path_buf());
        }
        path = match path.parent() {
            Some(path) => path,
            None => {
                return None;
            }
        };
    }
}
