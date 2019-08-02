/// compute the summed size of directories.
/// A cache is used to avoid recomputing the
///  same directories again and again.
/// Hard links are checked to avoid counting
///  twice an inode.
use crate::task_sync::TaskLifetime;
use std::collections::HashMap;
use std::ops::AddAssign;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Instant;

const SIZE_NAMES: &[&str] = &["", "K", "M", "G", "T", "P", "E", "Z", "Y"]; // Y: for when your disk is bigger than 1024 ZB

#[derive(Debug, Copy, Clone)]
pub struct Size(u64);

impl Size {
    pub fn from_file(path: &Path) -> Size {
        Size(compute_file_size(path))
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
        while v >= 5000 && i < SIZE_NAMES.len() - 1 {
            v >>= 10;
            i += 1;
        }
        format!("{}{}", v, &SIZE_NAMES[i])
    }
    pub fn part_of(&self, total: Size) -> f32 {
        if total.0 <= 0 {
            0.0
        } else {
            self.0 as f32 / total.0 as f32
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

#[cfg(unix)]
fn compute_file_size(path: &Path) -> u64 {
    file_sizes_unix::compute_file_size(path)
}

#[cfg(not(unix))]
mod file_sizes_default;

#[cfg(not(unix))]
fn compute_dir_size(path: &Path, tl: &TaskLifetime) -> Option<u64> {
    file_sizes_default::compute_dir_size(path, tl)
}
#[cfg(not(unix))]
fn compute_file_size(path: &Path) -> u64 {
    file_sizes_default::compute_file_size(path)
}
