
use {
    crate::{
        flat_tree::{
            Tree,
            TreeLine,
        },
    },
    git2::{
        self,
        Delta,
        Repository,
        Status,
    },
    pathdiff,
    std::{
        collections::HashMap,
        path::{
            Path,
            PathBuf,
        },
    },
};


// if I add nothing, I'll remove this useless struct
// and only use git2.Status
#[derive(Debug, Clone, Copy)]
pub struct LineGitStatus {
    pub status: Status,
}

///
#[derive(Debug, Clone)]
pub struct TreeGitStatus {
    pub current_branch_name: Option<String>,
    pub insertions: usize,
    pub deletions: usize,
}

pub struct GitStatusBuilder {
    root: PathBuf,
    repo: Repository,
    root_status: TreeGitStatus,
    //delta_map: HashMap<PathBuf, Status>,
}

impl GitStatusBuilder {
    pub fn from(tree: &Tree) -> Option<Self> {
        let repo = match Repository::discover(&tree.root()) {
            Ok(repo) => repo,
            Err(e) => {
                debug!("opening failed : {:?}", e);
                return None;
            }
        };
        let root = match repo.workdir() {
            Some(path) => path,
            None => {
                debug!("workdir not found");
                return None;
            }
        };
        // MAP
        /*
        let mut delta_map = HashMap::new();
        if let Ok(statuses) = &repo.statuses(None) {
            for entry in statuses.iter() {
                if let Some(path) = entry.path() {
                    let path = root.join(path);
                    debug!(" inserting {:?} {:?}", &path, entry.status());
                    delta_map.insert(path, entry.status());
                } else {
                    debug!("failed to get path from status entry");
                }
                //debug!("{:?} {:?}", entry.path(), entry.status());
            }
        } else {
            debug!("get statuses failed");
            return None;
        }*/
        let current_branch_name = repo.head().ok()
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
        let root_status = TreeGitStatus {
            current_branch_name,
            insertions: stats.insertions(),
            deletions: stats.deletions(),
        };
        Some(Self {
            root: root.to_path_buf(),
            repo,
            root_status,
            //delta_map,
        })
    }
    pub fn line_status(&self, line: &TreeLine) -> Option<LineGitStatus> {
        let relative_path = match pathdiff::diff_paths(&line.path, &self.root) {
            None => {
                debug!("getting relative path failed???");
                return None;
            }
            Some(p) => p,
        };
        self.repo
            .status_file(&relative_path).ok()
            .map(|status| LineGitStatus { status })
        // MAP
        //self.delta_map.get(&line.path).map(|status| LineGitStatus { status: *status })
    }
    pub fn try_enrich(tree: &mut Tree) {
        if let Some(builder) = Self::from(tree) {
            for mut line in tree.lines.iter_mut() {
                line.git_status = builder.line_status(&line);
            }
            tree.git_status = Some(builder.root_status);
        }
        debug!("tree_status: {:?}", tree.git_status);
    }
}

pub fn is_repo(root: &Path) -> bool {
    root.join(".git").exists()
}

