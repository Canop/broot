use std::io;
use std::path::Path;

use crate::app::AppContext;

pub fn relativize_path(path: &Path, con: &AppContext) -> io::Result<String> {
    let relative_path = match pathdiff::diff_paths(path, &con.initial_working_dir) {
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Cannot relativize {path:?}"), // does this happen ? how ?
            ));
        }
        Some(p) => p,
    };
    Ok(
        if relative_path.components().next().is_some() {
            relative_path.to_string_lossy().to_string()
        } else {
            ".".to_string()
        }
    )
}
