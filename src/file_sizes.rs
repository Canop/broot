/// compute the summed size of directories.
/// A cache is used to avoid recomputing the
///  same directories again and again.
/// Hard links are checked to avoid counting
///  twice an inode.
use crate::task_sync::TaskLifetime;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::ops::AddAssign;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Instant;

const SIZE_NAMES: &[&str] = &["", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"]; // YB: for when your disk is bigger than 1024 ZB

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
            static ref size_cache_mutex: Mutex<HashMap<PathBuf, Size>> = Mutex::new(HashMap::new());
        }
        let mut size_cache = size_cache_mutex.lock().unwrap();
        if let Some(s) = size_cache.get(path) {
            return Some(*s);
        }
        let start = Instant::now();
        let mut s = Size::from(0);
        let mut dirs: Vec<PathBuf> = Vec::new();
        dirs.push(PathBuf::from(path));
        let mut inodes: HashSet<u64> = HashSet::new(); // to avoid counting twice an inode
        let mut nb_duplicate_inodes = 0;
        while let Some(open_dir) = dirs.pop() {
            if let Ok(entries) = fs::read_dir(&open_dir) {
                for e in entries {
                    if let Ok(e) = e {
                        if let Ok(md) = e.metadata() {
                            if md.is_dir() {
                                dirs.push(e.path());
                            } else if md.nlink() > 1 && !inodes.insert(md.ino()) {
                                // it was already in the set
                                nb_duplicate_inodes += 1;
                                continue; // let's not add the size
                            }
                            s += Size::from(md.len());
                        }
                    }
                }
            }
            if tl.is_expired() {
                return None;
            }
        }
        size_cache.insert(PathBuf::from(path), s);
        debug!("size computation for {:?} took {:?}", path, start.elapsed());
        if nb_duplicate_inodes > 0 {
            debug!(
                " (found {} inodes used more than once)",
                nb_duplicate_inodes
            );
        }
        Some(s)
    }

    /// format a number of bytes as a string
    /// (probably fast enough but not benchmarked)
    pub fn to_string(&self) -> String {
        let mut v = self.0;
        let mut i = 0;
        while v >= 1024 && i < SIZE_NAMES.len() - 1 {
            v >>= 10;
            i += 1;
        }
        format!("{}{}", v, &SIZE_NAMES[i])
    }
    pub fn discreet_ratio(self, max: Size, r: u64) -> u64 {
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
