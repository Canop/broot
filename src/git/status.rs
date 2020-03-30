
use {
    git2::{
        self,
        Repository,
        Status,
    },
    std::{
        collections::HashMap,
        path::{
            Path,
            PathBuf,
        },
    },
};

const INTERESTING: Status = Status::from_bits_truncate(
    Status::WT_NEW.bits() | Status::CONFLICTED.bits() | Status::WT_MODIFIED.bits()
);

// if I add nothing, I'll remove this useless struct
// and only use git2.Status
#[derive(Debug, Clone, Copy)]
pub struct LineGitStatus {
    pub status: Status,
}

impl LineGitStatus {
    pub fn from(repo: &Repository, relative_path: &Path) -> Option<LineGitStatus> {
        repo
            .status_file(&relative_path).ok()
            .map(|status| LineGitStatus { status })
    }
    pub fn is_interesting(&self) -> bool {
        self.status.intersects(INTERESTING)
    }
}

pub struct LineStatusComputer {
    interesting_statuses: HashMap<PathBuf, Status>,
}
impl LineStatusComputer {
    pub fn from(repo: Repository) -> Self {
        let repo_path = repo.path().parent().unwrap().to_path_buf();
        let mut interesting_statuses = HashMap::new();
        if let Ok(statuses) = &repo.statuses(None) {
            for entry in statuses.iter() {
                let status = entry.status();
                if status.intersects(INTERESTING) {
                    if let Some(path) = entry.path() {
                        let path = repo_path.join(path);
                        interesting_statuses.insert(path, status);
                    }
                }
            }
        } else {
            debug!("get statuses failed");
        }
        Self {
            interesting_statuses,
        }
    }
    pub fn line_status(&self, path: &Path) -> Option<LineGitStatus> {
        self.interesting_statuses.get(path).map(|&status| LineGitStatus { status })
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
        let current_branch_name = repo.head()
            .ok()
            .and_then(|head| head.shorthand().map(String::from));
        let stats = match repo.diff_index_to_workdir(None, None) {
            Ok(diff) => {
                debug!("deltas: {:?}", diff.deltas().count());
                let stats = match diff.stats() {
                    Ok(stats) => stats,
                    Err(e) => {
                        debug!("get stats failed : {:?}", e);
                        return None;
                    }
                };
                stats
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


