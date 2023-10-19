pub mod magic_numbers;
pub mod extensions;

use {
    std::{
        io,
        path::Path,
    },
};

/// Assuming the path is already checked to be to a file
/// (not a link or directory), tell whether it looks like a text file
pub fn is_file_text<P: AsRef<Path>>(path: P) -> io::Result<bool> {
    // the current algorithm is rather crude. If needed I'll add
    // more, like checking the start of the file is UTF8 compatible
    Ok(!is_file_binary(path)?)
}

/// Assuming the path is already checked to be to a file
/// (not a link or directory), tell whether it looks like a binary file
pub fn is_file_binary<P: AsRef<Path>>(path: P) -> io::Result<bool> {
    if let Some(ext) = path.as_ref().extension().and_then(|s| s.to_str()) {
        if extensions::is_known_binary(ext) {
            return Ok(true);
        }
    }
    magic_numbers::is_file_known_binary(path)
}
