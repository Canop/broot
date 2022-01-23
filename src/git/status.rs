use {
    git2::{self, Repository, Status},
    ahash::AHashMap,
    std::{
        path::{Path, PathBuf},
    },
};

const INTERESTING: Status = Status::from_bits_truncate(
    Status::WT_NEW.bits() | Status::CONFLICTED.bits() | Status::WT_MODIFIED.bits(),
);

/// A git status
#[derive(Debug, Clone, Copy)]
pub struct LineGitStatus {
    pub status: Status,
}

impl LineGitStatus {
    pub fn from(repo: &Repository, relative_path: &Path) -> Option<LineGitStatus> {
        repo.status_file(relative_path)
            .ok()
            .map(|status| LineGitStatus { status })
    }
    pub fn is_interesting(self) -> bool {
        self.status.intersects(INTERESTING)
    }
}

/// As a git repo can't tell whether a path has a status, this computer
/// looks at all the statuses of the repo and build a map path->status
/// which can then be efficiently queried
pub struct LineStatusComputer {
    interesting_statuses: AHashMap<PathBuf, Status>,
}
impl LineStatusComputer {
    pub fn from(repo: Repository) -> Option<Self> {
        let workdir = repo.workdir()?;
        let mut interesting_statuses = AHashMap::default();
        let statuses = repo.statuses(None).ok()?;
        for entry in statuses.iter() {
            let status = entry.status();
            if status.intersects(INTERESTING) {
                if let Some(path) = entry.path() {
                    let path = workdir.join(path);
                    interesting_statuses.insert(path, status);
                }
            }
        }
        Some(Self { interesting_statuses })
    }
    pub fn line_status(&self, path: &Path) -> Option<LineGitStatus> {
        self.interesting_statuses
            .get(path)
            .map(|&status| LineGitStatus { status })
    }
    pub fn is_interesting(&self, path: &Path) -> bool {
        self.interesting_statuses.contains_key(path)
    }
}

///
#[derive(Debug, Clone)]
pub struct TreeGitStatus {
    pub current_branch_name: Option<String>,
    pub insertions: usize,
    pub deletions: usize,
}

impl TreeGitStatus {
    pub fn from(repo: &Repository) -> Option<Self> {
        let current_branch_name = repo
            .head()
            .ok()
            .and_then(|head| head.shorthand().map(String::from));
        let stats = match repo.diff_index_to_workdir(None, None) {
            Ok(diff) => {
                match diff.stats() {
                    Ok(stats) => stats,
                    Err(e) => {
                        debug!("get stats failed : {:?}", e);
                        return None;
                    }
                }
            }
            Err(e) => {
                debug!("get diff failed : {:?}", e);
                return None;
            }
        };
        Some(Self {
            current_branch_name,
            insertions: stats.insertions(),
            deletions: stats.deletions(),
        })
    }
}
