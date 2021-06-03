/// compute consolidated data for directories: modified date, size, and count.
/// A cache is used to avoid recomputing the same directories again and again.
/// On unix, hard links are checked to avoid counting twice an inode.

mod sum_computation;

use {
    crate::{
        app::*,
        task_sync::Dam,
    },
    ahash::AHashMap,
    once_cell::sync::Lazy,
    std::{
        ops::AddAssign,
        path::{Path, PathBuf},
        sync::Mutex,
    },
};

pub const DEFAULT_THREAD_COUNT: usize = 5;

static SUM_CACHE: Lazy<Mutex<AHashMap<PathBuf, FileSum>>> = Lazy::new(|| {
    Mutex::new(AHashMap::default())
});

pub fn clear_cache() {
    SUM_CACHE.lock().unwrap().clear();
}

/// Reduction of counts, dates and sizes on a file or directory
#[derive(Debug, Copy, Clone)]
pub struct FileSum {
    real_size: u64, // bytes, the space it takes on disk
    count: usize,   // number of files
    modified: u32,  // seconds from Epoch to last modification, or 0 if there was an error
    sparse: bool,   // only for non directories: tells whether the file is sparse
}

impl FileSum {
    pub fn new(
        real_size: u64,
        sparse: bool,
        count: usize,
        modified: u32,
    ) -> Self {
        Self { real_size, count, modified, sparse }
    }

    pub fn zero() -> Self {
        Self::new(0, false, 0, 0)
    }

    pub fn incr(&mut self) {
        self.count += 1;
    }

    /// return the sum of the given file, which is assumed
    /// to be a normal file (ie not a directory)
    pub fn from_file(path: &Path) -> Self {
        sum_computation::compute_file_sum(path)
    }

    /// Return the sum of the directory, either by computing it of by
    ///  fetching it from cache.
    /// If the lifetime expires before complete computation, None is returned.
    pub fn from_dir(path: &Path, dam: &Dam, con: &AppContext) -> Option<Self> {
        let mut sum_cache = SUM_CACHE.lock().unwrap();
        match sum_cache.get(path) {
            Some(sum) => Some(*sum),
            None => {
                let sum = time!(
                    "sum computation",
                    path,
                    sum_computation::compute_dir_sum(path, &mut sum_cache, dam, con),
                );
                if let Some(sum) = sum {
                    sum_cache.insert(PathBuf::from(path), sum);
                }
                sum
            }
        }
    }

    pub fn part_of_size(self, total: Self) -> f32 {
        if total.real_size == 0 {
            0.0
        } else {
            self.real_size as f32 / total.real_size as f32
        }
    }
    /// return the number of files (normally at least 1)
    pub fn to_count(self) -> usize {
        self.count
    }
    /// return the number of seconds from Epoch to last modification,
    /// or 0 if the computation failed
    pub fn to_seconds(self) -> u32 {
        self.modified
    }
    /// return the size in bytes
    pub fn to_size(self) -> u64 {
        self.real_size
    }
    pub fn to_valid_seconds(self) -> Option<i64> {
        if self.modified != 0 {
            Some(self.modified as i64)
        } else {
            None
        }
    }
    /// tell whether the file has holes (in which case the size displayed by
    /// other tools may be greater than the "real" one returned by broot).
    /// Not computed (return false) on windows or for directories.
    pub fn is_sparse(self) -> bool {
        self.sparse
    }
}

impl AddAssign for FileSum {
    #[allow(clippy::suspicious_op_assign_impl)]
    fn add_assign(&mut self, other: Self) {
        *self = Self::new(
            self.real_size + other.real_size,
            self.sparse | other.sparse,
            self.count + other.count,
            self.modified.max(other.modified),
        );
    }
}

