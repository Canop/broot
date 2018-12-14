use std::collections::VecDeque;
use std::fs;
use std::ops::AddAssign;
use std::path::{Path, PathBuf};
//use std::time::Instant;
use crate::task_sync::TaskLifetime;

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

    pub fn from_dir(path: &Path, tl: &TaskLifetime) -> Option<Size> {
        let mut s = Size::from(0);
        //let start = Instant::now();
        // todo try to use &Path instead of PathBuf
        let mut dirs: VecDeque<PathBuf> = VecDeque::new();
        dirs.push_back(PathBuf::from(path));
        while let Some(open_dir) = dirs.pop_front() {
            if let Ok(entries) = fs::read_dir(&open_dir) {
                for e in entries {
                    if let Ok(e) = e {
                        let p = e.path();
                        if let Ok(md) = fs::symlink_metadata(&p) {
                            s += Size::from(md.len());
                            if md.is_dir() {
                                dirs.push_back(p);
                            }
                        }
                    }
                }
            }
            if tl.is_expired() {
                return None;
            }
        }
        //debug!("size computation for {:?} took {:?}", path, start.elapsed());
        Some(s)
    }

    /// format a number of bytes as a string
    /// (probably fast enough but not benchmarked)
    pub fn to_string(&self) -> String {
        let mut v = self.0;
        let mut i = 0;
        while v >= 1024 && i < SIZE_NAMES.len() - 1 {
            v /= 1024;
            i += 1;
        }
        format!("{}{}", v, &SIZE_NAMES[i])
    }
    pub fn discreet_ratio(&self, max: Size, r: u64) -> u64 {
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
