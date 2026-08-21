use {
    crate::conf,
    serde::{Deserialize, Serialize},
    std::path::{Path, PathBuf},
};

/// A persistent list of favorite paths.
///
/// The structure is versioned to allow caching
/// of derived structs (filtered list mainly).
#[derive(Debug, Serialize, Deserialize)]
pub struct Favorites {
    #[serde(skip)]
    version: usize,
    paths: Vec<PathBuf>,
}

impl Default for Favorites {
    fn default() -> Self {
        Self {
            version: 0,
            paths: Vec::new(),
        }
    }
}

impl Favorites {
    /// Load favorites from the data directory, or return
    /// an empty list if the file doesn't exist or can't be parsed.
    pub fn load() -> Self {
        let path = Self::file_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<Favorites>(&content) {
                    Ok(mut fav) => {
                        fav.version = 0;
                        fav
                    }
                    Err(e) => {
                        warn!("Failed to parse favorites file: {}", e);
                        Favorites::default()
                    }
                },
                Err(e) => {
                    warn!("Failed to read favorites file: {}", e);
                    Favorites::default()
                }
            }
        } else {
            Favorites::default()
        }
    }

    fn file_path() -> PathBuf {
        let dirs = conf::app_dirs();
        dirs.data_dir().join("favorites.json")
    }

    fn save(&self) {
        let path = Self::file_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match serde_json::to_string_pretty(&self) {
            Ok(content) => {
                if let Err(e) = std::fs::write(&path, content) {
                    warn!("Failed to write favorites file: {}", e);
                }
            }
            Err(e) => {
                warn!("Failed to serialize favorites: {}", e);
            }
        }
    }

    pub fn contains(&self, path: &Path) -> bool {
        self.paths.iter().any(|p| p == path)
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
            self.save();
            true
        }
    }

    /// return true when there's a change
    pub fn remove(&mut self, path: &Path) -> bool {
        if let Some(pos) = self.paths.iter().position(|p| p == path) {
            self.version += 1;
            self.paths.remove(pos);
            self.save();
            true
        } else {
            false
        }
    }

    pub fn remove_idx(&mut self, idx: usize) {
        if idx < self.paths.len() {
            self.version += 1;
            self.paths.remove(idx);
            self.save();
        }
    }

    pub fn clear(&mut self) {
        self.version += 1;
        self.paths.clear();
        self.save();
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
            self.save();
        }
    }

    pub fn len(&self) -> usize {
        self.paths.len()
    }

    pub fn version(&self) -> usize {
        self.version
    }
}
