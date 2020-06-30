use {
    super::FileSum,
    crate::task_sync::Dam,
    crossbeam::channel,
    rayon::{ThreadPool, ThreadPoolBuilder},
    std::{
        convert::TryInto,
        fs,
        path::{Path, PathBuf},
        sync::{
            atomic::{AtomicIsize, Ordering},
            Arc,
        },
    },
};

#[cfg(unix)]
use {
    std::{
        collections::HashSet,
        os::unix::fs::MetadataExt,
        sync::Mutex,
    },
};

// threads used by one computation
const THREADS_COUNT: usize = 6;

/// compute the consolidated numbers for a directory, with implementation
/// varying depending on the OS:
/// On unix, the computation is done on blocks of 512 bytes
/// see https://doc.rust-lang.org/std/os/unix/fs/trait.MetadataExt.html#tymethod.blocks
pub fn compute_dir_sum(path: &Path, dam: &Dam) -> Option<FileSum> {
    //debug!("compute size of dir {:?} --------------- ", path);

    lazy_static! {
        static ref THREAD_POOL: ThreadPool = ThreadPoolBuilder::new().num_threads(THREADS_COUNT*2).build().unwrap();
    }

    // to avoid counting twice an inode, we store them in a set
    #[cfg(unix)]
    let inodes = Arc::new(Mutex::new(HashSet::<u64>::default()));

    // this MPMC channel contains the directory paths which must be handled.
    // A None means there's nothing left and the thread may send its result and stop
    let (dirs_sender, dirs_receiver) = channel::unbounded();

    // this MPMC channel is here for the threads to send their results
    // at end of computation
    let (thread_sum_sender, thread_sum_receiver) = channel::bounded(THREADS_COUNT);

    // busy is the number of directories which are either being processed or queued
    // We use this count to determine when threads can stop waiting for tasks
    let busy = Arc::new(AtomicIsize::new(1));
    dirs_sender.send(Some(PathBuf::from(path))).unwrap();

    // Each  thread does a summation without merge and the data are merged
    // at the end (this avoids waiting for a mutex during computation)
    for _ in 0..THREADS_COUNT {
        let busy = Arc::clone(&busy);
        let (dirs_sender, dirs_receiver) = (dirs_sender.clone(), dirs_receiver.clone());

        #[cfg(unix)]
        let inodes = inodes.clone();

        let observer = dam.observer();
        let thread_sum_sender = thread_sum_sender.clone();
        THREAD_POOL.spawn(move || {
            let mut thread_sum = FileSum::zero();
            loop {
                let o = dirs_receiver.recv();
                if let Ok(Some(open_dir)) = o {
                    if let Ok(entries) = fs::read_dir(&open_dir) {
                        for e in entries.flatten() {
                            if let Ok(md) = e.metadata() {
                                if md.is_dir() {
                                    // we add the directory to the channel of dirs needing
                                    // processing
                                    busy.fetch_add(1, Ordering::Relaxed);
                                    dirs_sender.send(Some(e.path())).unwrap();
                                } else {

                                    #[cfg(unix)]
                                    if md.nlink() > 1 {
                                        let mut inodes = inodes.lock().unwrap();
                                        if !inodes.insert(md.ino()) {
                                            // it was already in the set
                                            continue;
                                        }
                                    }

                                }

                                #[cfg(unix)]
                                let size = md.blocks() * 512;

                                #[cfg(not(unix))]
                                let size = md.len();

                                let seconds = extract_seconds(&md);
                                let entry_sum = FileSum::new(size, false, 1, seconds);
                                thread_sum += entry_sum;
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
    let mut sum = compute_file_sum(path);
    for _ in 0..THREADS_COUNT {
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

/// compute the sum for a regular file (not a folder)
pub fn compute_file_sum(path: &Path) -> FileSum {
    match fs::metadata(path) {
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
fn extract_seconds(md: &fs::Metadata) -> u32 {
    md.mtime().try_into().unwrap_or(0)
}

#[cfg(not(unix))]
fn extract_seconds(md: &fs::Metadata) -> u32 {
    if let Ok(st) = md.modified() {
        if let Ok(d) = st.duration_since(std::time::UNIX_EPOCH) {
            if let Ok(secs) = d.as_secs().try_into() {
                return secs
            }
        }
    }
    0
}

