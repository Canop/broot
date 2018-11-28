use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct TreeOptions {
    pub show_hidden: bool,
}

impl TreeOptions {
    pub fn new() -> TreeOptions {
        TreeOptions { show_hidden: false }
    }
    pub fn accepts(&self, path: &PathBuf) -> bool {
        if !self.show_hidden {
            // FIXME what's the proper way to check whether a file is hidden ?
            if let Some(filename) = path.file_name() {
                let first_char = filename.to_string_lossy().chars().next();
                if let Some('.') = first_char {
                    return false;
                }
            }
        }
        true
    }
}
