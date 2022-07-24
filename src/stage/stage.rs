use {
    crate::{
        app::AppContext,
        file_sum::FileSum,
        task_sync::Dam,
    },
    std::{
        path::{Path, PathBuf},
    },
};

/// a staging area: selection of several paths
/// for later user
///
/// The structure is versioned to allow caching
/// of derived structs (filtered list mainly). This
/// scheme implies the stage isn't cloned, and that
/// it exists in only one instance
#[derive(Default, Debug)]
pub struct Stage {
    version: usize,
    paths: Vec<PathBuf>,
}

impl Stage {
    pub fn contains(&self, path: &Path) -> bool {
        self.paths
            .iter()
            .any(|p| p==path)
    }
    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }
    /// return true when there's a change
    pub fn add(&mut self, path: PathBuf) -> bool {
        if self.contains(&path) {
            false
        } else {
            self.version += 1;
            self.paths.push(path);
            true
        }
    }
    /// return true when there's a change
    pub fn remove(&mut self, path: &Path) -> bool {
        if let Some(pos) = self.paths.iter().position(|p| p == path) {
            self.version += 1;
            self.paths.remove(pos);
            true
        } else {
            false
        }
    }
    pub fn remove_idx(&mut self, idx: usize) {
        if idx < self.paths.len() {
            self.version += 1;
            self.paths.remove(idx);
        }
    }
    pub fn clear(&mut self) {
        self.version += 1;
        self.paths.clear();
    }
    pub fn paths(&self) -> &[PathBuf] {
        &self.paths
    }
    /// removes paths to non existing files
    pub fn refresh(&mut self) {
        let len_before = self.paths.len();
        self.paths.retain(|p| p.exists());
        if self.paths.len() != len_before {
            self.version += 1;
        }
    }
    pub fn len(&self) -> usize {
        self.paths.len()
    }
    pub fn version(&self) -> usize {
        self.version
    }
    pub fn compute_sum(&self, dam: &Dam, con: &AppContext) -> Option<FileSum> {
        let mut sum = FileSum::zero();
        for path in &self.paths {
            if path.is_dir() {
                let dir_sum = FileSum::from_dir(path, dam, con);
                if let Some(dir_sum) = dir_sum {
                    sum += dir_sum;
                } else {
                    return None; // computation was interrupted
                }
            } else {
                sum += FileSum::from_file(path);
            }
        }
        Some(sum)
    }
}
