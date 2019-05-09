/// compute the summed size of directories.
/// A cache is used to avoid recomputing the
///  same directories again and again.
/// Hard links are checked to avoid counting
///  twice an inode.
use crate::task_sync::TaskLifetime;
use std::fs;
use std::ops::AddAssign;
use std::path::{Path, PathBuf};

#[cfg(unix)]
mod file_sizes_unix;

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

    #[cfg(windows)]

    pub fn from_dir(path: &Path, tl: &TaskLifetime) -> Option<Size> {
        Some(Size::from(0)) // it's correct, sometimes
    }

    #[cfg(unix)]
    /// Return the size of the directory, either by computing it of by
    ///  fetching it from cache.
    /// If the lifetime expires before complete computation, None is returned.
    pub fn from_dir(path: &Path, tl: &TaskLifetime) -> Option<Size> {
        file_sizes_unix::compute_dir_size(path, tl).map(|s| Size::from(s))
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
