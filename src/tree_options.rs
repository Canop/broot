use std::fs;
use std::path::PathBuf;

use patterns::Pattern;

#[derive(Debug, Clone)]
pub struct TreeOptions {
    pub show_hidden: bool,
    pub pattern: Option<Pattern>,
}

impl TreeOptions {
    pub fn new() -> TreeOptions {
        TreeOptions {
            show_hidden: false,
            pattern: None,
        }
    }
    pub fn accepts(&self, path: &PathBuf, depth_decr: usize) -> bool {
        if let Some(filename) = path.file_name() {
            let filename = filename.to_string_lossy();
            if !self.show_hidden {
                // FIXME what's the proper way to check whether a file is hidden ?
                if filename.starts_with(".") {
                    return false;
                }
            }
            if let Some(pattern) = &self.pattern {
                if let Some(_) = pattern.test(&filename) {
                    return true;
                }
                if let Ok(metadata) = fs::metadata(&path) {
                    if metadata.is_dir() {
                        if depth_decr == 0 {
                            return false;
                        }
                        if let Ok(entries) = fs::read_dir(&path) {
                            for e in entries {
                                if let Ok(e) = e {
                                    if self.accepts(&e.path(), depth_decr - 1) {
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }
                return false;
            }
        }
        true
    }
}
