/// compute the summed size of directories.
/// A cache is used to avoid recomputing the
///  same directories again and again.
/// Hard links are checked to avoid counting
///  twice an inode.
use crate::task_sync::TaskLifetime;
use crossbeam::channel::unbounded;
use crossbeam::sync::WaitGroup;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::ops::AddAssign;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicIsize, AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::time::Instant;

const SIZE_NAMES: &[&str] = &["", "K", "M", "G", "T", "P", "E", "Z", "Y"]; // Y: for when your disk is bigger than 1024 ZB

#[derive(Debug, Copy, Clone)]
pub struct Size(u64);

impl Size {
    pub fn from_file(path: &Path) -> Size {
        Size(match fs::metadata(path) {
            Ok(m) => m.len(),
            Err(_) => 0,
        })
    }

    /// Return the size of the directory, either by computing it of by
    ///  fetching it from cache.
    /// If the lifetime expires before complete computation, None is returned.
    pub fn from_dir(path: &Path, tl: &TaskLifetime) -> Option<Size> {
        lazy_static! {
            static ref SIZE_CACHE_MUTEX: Mutex<HashMap<PathBuf, Size>> = Mutex::new(HashMap::new());
        }
        let mut size_cache = SIZE_CACHE_MUTEX.lock().unwrap();
        if let Some(s) = size_cache.get(path) {
            return Some(*s);
        }

        let start = Instant::now();
        let inodes = Arc::new(Mutex::new(HashSet::<u64>::new())); // to avoid counting twice an inode
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
                                    size.fetch_add(md.len() as usize, Ordering::Relaxed);
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
        let size: usize = size.load(Ordering::Relaxed);
        let size: u64 = size as u64;
        let s = Size::from(size);

        size_cache.insert(PathBuf::from(path), s);
        debug!("size computation for {:?} took {:?}", path, start.elapsed());
        Some(s)
    }

    /// format a number of bytes as a string
    pub fn to_string(self) -> String {
        let mut v = self.0;
        let mut i = 0;
        while v >= 9000 && i < SIZE_NAMES.len() - 1 {
            v >>= 10;
            i += 1;
        }
        format!("{}{}", v, &SIZE_NAMES[i])
    }
    pub fn discrete_ratio(self, max: Size, r: u64) -> u64 {
        if max.0 == 0 || self.0 == 0 {
            0
        } else {
            ((r as f64) * (self.0 as f64).cbrt() / (max.0 as f64).cbrt()).round() as u64
        }
    }
}

impl From<u64> for Size {
    fn from(s: u64) -> Size {
        Size(s)
    }
}

impl AddAssign for Size {
    fn add_assign(&mut self, other: Size) {
        *self = Size(self.0 + other.0);
    }
}

impl Into<u64> for Size {
    fn into(self) -> u64 {
        self.0
    }
}
