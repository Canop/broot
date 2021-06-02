use {
    super::TreeGitStatus,
    crate::{
        git,
        task_sync::{Computation, ComputationResult, Dam},
    },
    crossbeam::channel::bounded,
    ahash::AHashMap,
    git2::Repository,
    once_cell::sync::Lazy,
    std::{
        path::{Path, PathBuf},
        sync::Mutex,
    },
};

fn compute_tree_status(root_path: &Path) -> ComputationResult<TreeGitStatus> {
    match Repository::open(root_path) {
        Ok(git_repo) => {
            let tree_git_status = time!(TreeGitStatus::from(&git_repo),);
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

    // the key is the path of the repository
static TS_CACHE_MX: Lazy<Mutex<AHashMap<PathBuf, Computation<TreeGitStatus>>>> = Lazy::new(|| {
        Mutex::new(AHashMap::default())
});

/// try to get the result of the computation of the tree git status.
/// This may be immediate if a previous computation was finished.
/// This may wait for the result of a new computation or of a previously
/// launched one.
/// In any case:
/// - this function returns as soon as the dam asks for it (ie when there's an event)
/// - computations are never dropped unless the program ends: they continue in background
///    and the result may be available for following queries
pub fn get_tree_status(root_path: &Path, dam: &mut Dam) -> ComputationResult<TreeGitStatus> {
    match git::closest_repo_dir(root_path) {
        None => ComputationResult::None,
        Some(repo_path) => {
            let comp = TS_CACHE_MX
                .lock()
                .unwrap()
                .get(&repo_path)
                .map(|c| (*c).clone());
            match comp {
                Some(Computation::Finished(comp_res)) => {
                    // already computed
                    comp_res
                }
                Some(Computation::InProgress(comp_receiver)) => {
                    // computation in progress
                    // We do a select! to wait for either the dam
                    // or the receiver
                    debug!("start select on in progress computation");
                    dam.select(comp_receiver)
                }
                None => {
                    // not yet started. We launch the computation and store
                    // the receiver immediately.
                    // We use the dam to return from this function when
                    // needed (while letting the underlying thread finish
                    // the job)
                    //
                    // note: must also update the TS_CACHE entry at end
                    let (s, r) = bounded(1);
                    TS_CACHE_MX
                        .lock()
                        .unwrap()
                        .insert(repo_path.clone(), Computation::InProgress(r));
                    dam.try_compute(move || {
                        let comp_res = compute_tree_status(&repo_path);
                        TS_CACHE_MX
                            .lock()
                            .unwrap()
                            .insert(repo_path.clone(), Computation::Finished(comp_res.clone()));
                        if let Err(e) = s.send(comp_res.clone()) {
                            debug!("error while sending comp result: {:?}", e);
                        }
                        comp_res
                    })
                }
            }
        }
    }
}

/// clear the finished or in progress computation.
/// Limit: we may receive in cache the result of a computation
/// which started before the clear (if this is a problem we could
/// store a cleaning counter alongside the cache to prevent insertions)
pub fn clear_status_computer_cache() {
    let mut ts_cache = TS_CACHE_MX.lock().unwrap();
    ts_cache.clear();
}
