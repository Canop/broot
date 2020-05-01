use {
    super::FileSize,
    crate::task_sync::Dam,
    crossbeam::{channel::unbounded, sync::WaitGroup},
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

pub fn compute_dir_size(path: &Path, dam: &Dam) -> Option<u64> {
    //debug!("compute size of dir {:?} --------------- ", path);
    let inodes = Arc::new(Mutex::new(HashSet::<u64>::default())); // to avoid counting twice an inode
                                                                  // the computation is done on blocks of 512 bytes
                                                                  // see https://doc.rust-lang.org/std/os/unix/fs/trait.MetadataExt.html#tymethod.blocks
    let blocks = Arc::new(AtomicU64::new(0));

    // this MPMC channel contains the directory paths which must be handled
    let (dirs_sender, dirs_receiver) = unbounded();

    // busy is the number of directories which are either being processed or queued
    // We use this count to determine when threads can stop waiting for tasks
    let busy = Arc::new(AtomicIsize::new(0));
    busy.fetch_add(1, Ordering::Relaxed);
    dirs_sender.send(Some(PathBuf::from(path))).unwrap();

    let wg = WaitGroup::new();
    let period = Duration::from_micros(50);
    for _ in 0..8 {
        let blocks = Arc::clone(&blocks);
        let busy = Arc::clone(&busy);
        let wg = wg.clone();
        let (dirs_sender, dirs_receiver) = (dirs_sender.clone(), dirs_receiver.clone());
        let inodes = inodes.clone();
        let observer = dam.observer();
        thread::spawn(move || {
            loop {
                let o = dirs_receiver.recv_timeout(period);
                if let Ok(Some(open_dir)) = o {
                    if let Ok(entries) = fs::read_dir(&open_dir) {
                        for e in entries.flatten() {
                            if let Ok(md) = e.metadata() {
                                if md.is_dir() {
                                    busy.fetch_add(1, Ordering::Relaxed);
                                    dirs_sender.send(Some(e.path())).unwrap();
                                } else if md.nlink() > 1 {
                                    let mut inodes = inodes.lock().unwrap();
                                    if !inodes.insert(md.ino()) {
                                        // it was already in the set
                                        continue; // let's not add the blocks
                                    }
                                }
                                blocks.fetch_add(md.blocks(), Ordering::Relaxed);
                            }
                        }
                    }
                    busy.fetch_sub(1, Ordering::Relaxed);
                    dirs_sender.send(None).unwrap();
                } else if busy.load(Ordering::Relaxed) < 1 {
                    break;
                }
                if observer.has_event() {
                    break;
                }
            }
            drop(wg);
        });
    }
    wg.wait();

    if dam.has_event() {
        return None;
    }
    let blocks = blocks.load(Ordering::Relaxed);
    Some(blocks * 512)
}

pub fn compute_file_size(path: &Path) -> FileSize {
    match fs::metadata(path) {
        Ok(md) => {
            let nominal_size = md.size();
            let block_size = md.blocks() * 512;
            FileSize::new(block_size.min(nominal_size), block_size < nominal_size)
        }
        Err(_) => FileSize::new(0, false),
    }
}
