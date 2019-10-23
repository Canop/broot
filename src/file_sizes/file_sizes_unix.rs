use std::{
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
};

use crossbeam::{channel::unbounded, sync::WaitGroup};

use crate::task_sync::TaskLifetime;

pub fn compute_dir_size(path: &Path, tl: &TaskLifetime) -> Option<u64> {
    let inodes = Arc::new(Mutex::new(HashSet::<u64>::default())); // to avoid counting twice an inode
    let size = Arc::new(AtomicU64::new(0));

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
        let size = Arc::clone(&size);
        let busy = Arc::clone(&busy);
        let wg = wg.clone();
        let (dirs_sender, dirs_receiver) = (dirs_sender.clone(), dirs_receiver.clone());
        let tl = tl.clone();
        let inodes = inodes.clone();
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
                                        continue; // let's not add the size
                                    }
                                }
                                let file_size = if md.blocks() == 0 { 0 } else { md.size() };
                                size.fetch_add(file_size, Ordering::Relaxed);
                            }
                        }
                    }
                    busy.fetch_sub(1, Ordering::Relaxed);
                    dirs_sender.send(None).unwrap();
                } else if busy.load(Ordering::Relaxed) < 1 {
                    break;
                }
                if tl.is_expired() {
                    break;
                }
            }
            drop(wg);
        });
    }
    wg.wait();

    if tl.is_expired() {
        return None;
    }
    let size = size.load(Ordering::Relaxed);
    Some(size)
}

pub fn compute_file_size(path: &Path) -> u64 {
    match fs::metadata(path) {
        Ok(m) => {
            if m.blocks() == 0 {
                0
            } else {
                m.size()
            }
        }
        Err(_) => 0,
    }
}
