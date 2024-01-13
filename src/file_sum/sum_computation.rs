use {
    super::FileSum,
    crate::{
        app::*,
        path::*,
        task_sync::Dam,
    },
    ahash::AHashMap,
    crossbeam::channel,
    rayon::{ThreadPool, ThreadPoolBuilder},
    std::{
        convert::TryInto,
        fs,
        path::{Path, PathBuf},
        sync::{
            atomic::{AtomicIsize, Ordering},
            Arc,
            Mutex,
        },
    },
};

#[cfg(unix)]
use {
    fnv::FnvHashSet,
    std::{
        os::unix::fs::MetadataExt,
    },
};

struct DirSummer {
    thread_count: usize,
    thread_pool: ThreadPool,
}

/// a node id, taking the device into account to be sure to discriminate
/// nodes with the same inode but on different devices
#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
struct NodeId {
    /// inode number
    inode: u64,
    /// device number
    dev: u64,
}

impl DirSummer {
    pub fn new(thread_count: usize) -> Self {
        let thread_pool = ThreadPoolBuilder::new()
            .num_threads(thread_count)
            .build()
            .unwrap();
        Self {
            thread_count,
            thread_pool,
        }
    }
    /// compute the consolidated numbers for a directory, with implementation
    /// varying depending on the OS:
    /// On unix, the computation is done on blocks of 512 bytes
    /// see https://doc.rust-lang.org/std/os/unix/fs/trait.MetadataExt.html#tymethod.blocks
    pub fn compute_dir_sum(
        &mut self,
        path: &Path,
        cache: &mut AHashMap<PathBuf, FileSum>,
        dam: &Dam,
        con: &AppContext,
    ) -> Option<FileSum> {
        let threads_count = self.thread_count;

        if con.special_paths.sum(path) == Directive::Never {
            return Some(FileSum::zero());
        }

        // there are problems in /proc - See issue #637
        if path.starts_with("/proc") {
            debug!("not summing in /proc");
            return Some(FileSum::zero());
        }
        if path.starts_with("/run") && ! path.starts_with("/run/media") {
            debug!("not summing in /run");
            return Some(FileSum::zero());
        }

        // to avoid counting twice a node, we store their id in a set
        #[cfg(unix)]
        let nodes = Arc::new(Mutex::new(FnvHashSet::<NodeId>::default()));

        // busy is the number of directories which are either being processed or queued
        // We use this count to determine when threads can stop waiting for tasks
        let mut busy = 0;
        let mut sum = compute_file_sum(path);

        // this MPMC channel contains the directory paths which must be handled.
        // A None means there's nothing left and the thread may send its result and stop
        let (dirs_sender, dirs_receiver) = channel::unbounded();

        let special_paths = con.special_paths.reduce(path);

        // the first level is managed a little differently: we look at the cache
        // before adding. This enables faster computations in two cases:
        // - for the root line (assuming it's computed after the content)
        // - when we navigate up the tree
        if let Ok(entries) = fs::read_dir(path) {
            for e in entries.flatten() {
                if let Ok(md) = e.metadata() {
                    if md.is_dir() {
                        let entry_path = e.path();

                        if con.special_paths.sum(&entry_path) == Directive::Never {
                            debug!("not summing special path {:?}", entry_path);
                            continue;
                        }

                        // we check the cache
                        if let Some(entry_sum) = cache.get(&entry_path) {
                            sum += *entry_sum;
                            continue;
                        }

                        // we add the directory to the channel of dirs needing
                        // processing
                        busy += 1;
                        dirs_sender.send(Some(entry_path)).unwrap();
                    } else {

                        #[cfg(unix)]
                        if md.nlink() > 1 {
                            let mut nodes = nodes.lock().unwrap();
                            let node_id = NodeId {
                                inode: md.ino(),
                                dev: md.dev(),
                            };
                            if !nodes.insert(node_id) {
                                // it was already in the set
                                continue;
                            }
                        }

                    }
                    sum += md_sum(&md);
                }
            }
        }

        if busy == 0 {
            return Some(sum);
        }

        let busy = Arc::new(AtomicIsize::new(busy));

        // this MPMC channel is here for the threads to send their results
        // at end of computation
        let (thread_sum_sender, thread_sum_receiver) = channel::bounded(threads_count);

        // Each  thread does a summation without merge and the data are merged
        // at the end (this avoids waiting for a mutex during computation)
        for _ in 0..threads_count {
            let busy = Arc::clone(&busy);
            let (dirs_sender, dirs_receiver) = (dirs_sender.clone(), dirs_receiver.clone());

            #[cfg(unix)]
            let nodes = nodes.clone();

            let special_paths = special_paths.clone();

            let observer = dam.observer();
            let thread_sum_sender = thread_sum_sender.clone();
            self.thread_pool.spawn(move || {
                let mut thread_sum = FileSum::zero();
                loop {
                    let o = dirs_receiver.recv();
                    if let Ok(Some(open_dir)) = o {
                        if let Ok(entries) = fs::read_dir(open_dir) {
                            for e in entries.flatten() {
                                if let Ok(md) = e.metadata() {
                                    if md.is_dir() {

                                        let path = e.path();

                                        if special_paths.sum(&path) == Directive::Never {
                                            debug!("not summing (deep) special path {:?}", path);
                                            continue;
                                        }

                                        // we add the directory to the channel of dirs needing
                                        // processing
                                        busy.fetch_add(1, Ordering::Relaxed);
                                        dirs_sender.send(Some(path)).unwrap();
                                    } else {

                                        #[cfg(unix)]
                                        if md.nlink() > 1 {
                                            let mut nodes = nodes.lock().unwrap();
                                            let node_id = NodeId {
                                                inode: md.ino(),
                                                dev: md.dev(),
                                            };
                                            if !nodes.insert(node_id) {
                                                // it was already in the set
                                                continue;
                                            }
                                        }

                                    }
                                    thread_sum += md_sum(&md);
                                } else {
                                    // we can't measure much but we can count the file
                                    thread_sum.incr();
                                }
                            }
                        }
                        busy.fetch_sub(1, Ordering::Relaxed);
                    }
                    if observer.has_event() {
                        dirs_sender.send(None).unwrap(); // to unlock the next waiting thread
                        break;
                    }
                    if busy.load(Ordering::Relaxed) < 1 {
                        dirs_sender.send(None).unwrap(); // to unlock the next waiting thread
                        break;
                    }
                }
                thread_sum_sender.send(thread_sum).unwrap();
            });
        }
        // Wait for the threads to finish and consolidate their results
        for _ in 0..threads_count {
            match thread_sum_receiver.recv() {
                Ok(thread_sum) => {
                    sum += thread_sum;
                }
                Err(e) => {
                    warn!("Error while recv summing thread result : {:?}", e);
                }
            }
        }
        if dam.has_event() {
            return None;
        }
        Some(sum)
    }
}


/// compute the consolidated numbers for a directory, with implementation
/// varying depending on the OS:
/// On unix, the computation is done on blocks of 512 bytes
/// see https://doc.rust-lang.org/std/os/unix/fs/trait.MetadataExt.html#tymethod.blocks
pub fn compute_dir_sum(
    path: &Path,
    cache: &mut AHashMap<PathBuf, FileSum>,
    dam: &Dam,
    con: &AppContext,
) -> Option<FileSum> {
    use once_cell::sync::OnceCell;
    static DIR_SUMMER: OnceCell<Mutex<DirSummer>> = OnceCell::new();
    DIR_SUMMER
        .get_or_init(|| {
            Mutex::new(DirSummer::new(con.file_sum_threads_count))
        })
        .lock().unwrap()
        .compute_dir_sum(path, cache, dam, con)
}

/// compute the sum for a regular file (not a folder)
pub fn compute_file_sum(path: &Path) -> FileSum {
    match fs::symlink_metadata(path) {
        Ok(md) => {
            let seconds = extract_seconds(&md);

            #[cfg(unix)]
            {
                let nominal_size = md.size();
                let block_size = md.blocks() * 512;
                FileSum::new(
                    block_size.min(nominal_size),
                    block_size < nominal_size,
                    1,
                    seconds,
                )
            }

            #[cfg(not(unix))]
            FileSum::new(md.len(), false, 1, seconds)
        }
        Err(_) => FileSum::new(0, false, 1, 0),
    }
}

#[cfg(unix)]
#[inline(always)]
fn extract_seconds(md: &fs::Metadata) -> u32 {
    md.mtime().try_into().unwrap_or(0)
}

#[cfg(not(unix))]
#[inline(always)]
fn extract_seconds(md: &fs::Metadata) -> u32 {
    if let Ok(st) = md.modified() {
        if let Ok(d) = st.duration_since(std::time::UNIX_EPOCH) {
            if let Ok(secs) = d.as_secs().try_into() {
                return secs;
            }
        }
    }
    0
}


#[inline(always)]
fn md_sum(md: &fs::Metadata) -> FileSum {
    #[cfg(unix)]
    let size = md.blocks() * 512;

    #[cfg(not(unix))]
    let size = md.len();

    let seconds = extract_seconds(md);
    FileSum::new(size, false, 1, seconds)
}
