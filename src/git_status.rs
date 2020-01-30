
use {
    git2::{
        self,
        Repository,
        Status,
    },
    std::{
        path::{
            Path,
        },
        time::Instant,
    },
};

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
}

///
#[derive(Debug, Clone)]
pub struct TreeGitStatus {
    pub current_branch_name: Option<String>,
    pub insertions: usize,
    pub deletions: usize,
}

impl TreeGitStatus {
    pub fn from(repo: &Repository) -> Option<TreeGitStatus> {
        let start = Instant::now();
        let current_branch_name = repo.head().ok()
            .and_then(|head| head.shorthand().map(String::from));
        debug!("finding current branch name took {:?}", start.elapsed());
        let start = Instant::now();
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
                debug!("get diff took {:?} and failed : {:?}", start.elapsed(), e);
                return None;
            }
        };
        debug!("get diff and stats took {:?}", start.elapsed());
        Some(Self {
            current_branch_name,
            insertions: stats.insertions(),
            deletions: stats.deletions(),
        })
    }
}


