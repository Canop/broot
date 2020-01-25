/// compute the summed size of directories.
/// A cache is used to avoid recomputing the
///  same directories again and again.
/// Hard links are checked to avoid counting
///  twice an inode.
///
use {
    crate::task_sync::TaskLifetime,
    std::{
        collections::HashMap,
        fmt,
        ops::AddAssign,
        path::{Path, PathBuf},
        sync::Mutex,
        time::Instant,
    },
};

const SIZE_NAMES: &[&str] = &["", "K", "M", "G", "T", "P", "E", "Z", "Y"];

lazy_static! {
    static ref SIZE_CACHE_MUTEX: Mutex<HashMap<PathBuf, u64>> = Mutex::new(HashMap::new());
}

pub fn clear_cache() {
    let mut size_cache = SIZE_CACHE_MUTEX.lock().unwrap();
    size_cache.clear();
}

#[derive(Debug, Copy, Clone)]
pub struct FileSize {
    real_size: u64, // bytes, the space it takes on disk
    pub sparse: bool,   // only for non directories: tells whether the file is sparse
}

impl FileSize {

    pub fn new(real_size: u64, sparse: bool) -> Self {
        Self { real_size, sparse }
    }

    /// return the size of the given file, which is assumed
    /// to be a normal file (ie not a directory)
    pub fn from_file(path: &Path) -> Self {
        compute_file_size(path)
    }

    /// Return the size of the directory, either by computing it of by
    ///  fetching it from cache.
    /// If the lifetime expires before complete computation, None is returned.
    pub fn from_dir(path: &Path, tl: &TaskLifetime) -> Option<Self> {
        let mut size_cache = SIZE_CACHE_MUTEX.lock().unwrap();
        if let Some(s) = size_cache.get(path) {
            return Some(Self::new(*s, false));
        }
        let start = Instant::now();
        if let Some(s) = compute_dir_size(path, tl) {
            size_cache.insert(PathBuf::from(path), s);
            debug!("size computation for {:?} took {:?}", path, start.elapsed());
            Some(FileSize::new(s, false))
        } else {
            None
        }
    }

    pub fn part_of(self, total: Self) -> f32 {
        if total.real_size == 0 {
            0.0
        } else {
            self.real_size as f32 / total.real_size as f32
        }
    }
}

impl fmt::Display for FileSize {
    /// format a number of bytes as a string, for example 247K
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut v = self.real_size;
        let mut i = 0;
        while v >= 5000 && i < SIZE_NAMES.len() - 1 {
            //v >>= 10;
            v /= 1000;
            i += 1;
        }
        write!(f, "{}{}", v, &SIZE_NAMES[i])
    }
}

impl AddAssign for FileSize {
    fn add_assign(&mut self, other: Self) {
        *self = Self::new(
            self.real_size + other.real_size,
            self.sparse | other.sparse,
        );
    }
}

impl Into<u64> for FileSize {
    fn into(self) -> u64 {
        self.real_size
    }
}

// ---------------- OS dependent implementations

#[cfg(unix)]
mod file_sizes_unix;
#[cfg(unix)]
use file_sizes_unix::*;

#[cfg(not(unix))]
mod file_sizes_default;
#[cfg(not(unix))]
use file_sizes_default::*;

