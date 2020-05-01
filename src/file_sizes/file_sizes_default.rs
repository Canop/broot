//! size computation for non linux

use {
    super::FileSize,
    crate::task_sync::Dam,
    crossbeam::{channel::unbounded, sync::WaitGroup},
    std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicIsize, AtomicUsize, Ordering},
        sync::Arc,
        thread,
        time::Duration,
    },
};

// Note that this version doesn't try to compute the real size taken
// on disk but report the value given by the `len` function
pub fn compute_dir_size(path: &Path, dam: &Dam) -> Option<u64> {
    let size = Arc::new(AtomicUsize::new(0));

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
                                }
                                size.fetch_add(md.len() as usize, Ordering::Relaxed);
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
    let size: usize = size.load(Ordering::Relaxed);
    let size: u64 = size as u64;
    Some(size)
}

pub fn compute_file_size(path: &Path) -> FileSize {
    match fs::metadata(path) {
        Ok(m) => FileSize::new(m.len(), false),
        Err(_) => FileSize::new(0, false),
    }
}
