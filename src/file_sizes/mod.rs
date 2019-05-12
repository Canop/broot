/// compute the summed size of directories.
/// A cache is used to avoid recomputing the
///  same directories again and again.
/// Hard links are checked to avoid counting
///  twice an inode.
use crate::task_sync::TaskLifetime;
use std::fs;
use std::ops::AddAssign;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Instant;
use std::thread;
use crossbeam::channel::unbounded;
use crossbeam::sync::WaitGroup;
use std::sync::Arc;
use std::sync::atomic::{AtomicIsize, AtomicUsize, Ordering};
use std::time::Duration;

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
        if let Some(s) = compute_dir_size(path, tl) {
            let size = Size::from(s);
            size_cache.insert(PathBuf::from(path), size);
            debug!("size computation for {:?} took {:?}", path, start.elapsed());
            Some(size)
        } else {
            None
        }
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

// ---------------- OS dependent implementations

#[cfg(unix)]
mod file_sizes_unix;

#[cfg(unix)]
fn compute_dir_size(path: &Path, tl: &TaskLifetime) -> Option<u64> {
    file_sizes_unix::compute_dir_size(path, tl)
}

#[cfg(windows)]
mod file_sizes_windows;

#[cfg(windows)]
fn compute_dir_size(path: &Path, tl: &TaskLifetime) -> Option<u64> {
    file_sizes_windows::compute_dir_size(path, tl)
}
