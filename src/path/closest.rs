use std::path::{Path, PathBuf};

/// return the closest enclosing directory
pub fn closest_dir(mut path: &Path) -> PathBuf {
    loop {
        if path.exists() && path.is_dir() {
            return path.to_path_buf();
        }
        match path.parent() {
            Some(parent) => path = parent,
            None => {
                debug!("no existing parent"); // unexpected
                return path.to_path_buf();
            }
        }
    }
}
