
use {
    crate::{
        git_status::*,
        task_sync::{
            Dam,
            ComputationResult,
        },
    },
    git2::{
        Repository,
    },
    //rayon::{
    //    ThreadPool,
    //    ThreadPoolBuilder,
    //},
    crossbeam::{channel::bounded, sync::WaitGroup},
    std::{
        collections::HashSet,
        fs,
        os::unix::fs::MetadataExt,
        path::{Path, PathBuf},
        sync::{
            atomic::{AtomicIsize, AtomicU64, Ordering},
            Arc, Mutex,
        },
        thread,
        time::Duration,
    },
};


//struct Computer {
//
//}
//impl Computer {
//    pub fn new() -> Self {
//
//    }
//}


pub fn compute_tree_status(root_path: PathBuf) -> ComputationResult<TreeGitStatus> {
    match Repository::discover(root_path) {
        Ok(git_repo) => {
            debug!("repo opened");
            for _ in 0..20 {
                time!(
                    Debug,
                    "compute_tree_status",
                    TreeGitStatus::from(&git_repo),
                );
            }
            let tree_git_status = time!(
                Debug,
                "compute_tree_status",
                TreeGitStatus::from(&git_repo),
            );
            match tree_git_status {
                Some(gs) => ComputationResult::Done(gs),
                None => ComputationResult::None,
            }
        }
        Err(e) => {
            debug!("failed to discover repo: {:?}", e);
            ComputationResult::None
        }
    }
}
