use {
    std::{
        path::{Path, PathBuf},
    },
};

/// a staging area: selection of several paths
/// for later user
#[derive(Default, Debug)]
pub struct Stage {
    pub paths: Vec<PathBuf>,
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
            self.paths.push(path);
            true
        }
    }
    /// return true when there's a change
    pub fn remove(&mut self, path: &Path) -> bool {
        if let Some(pos) = self.paths.iter().position(|p| p == path) {
            self.paths.remove(pos);
            true
        } else {
            false
        }
    }
    /// removes paths to non existing files
    pub fn refresh(&mut self) {
        self.paths.retain(|p| p.exists());
    }
}
