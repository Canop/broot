
mod content_match;
mod content_search_result;
mod magic_numbers;
mod extensions;
mod needle;

pub use {
    content_match::ContentMatch,
    content_search_result::ContentSearchResult,
    needle::Needle,
    std::io::{ BufRead, BufReader},
};

use {
    memmap2::Mmap,
    std::{
        fs::File,
        io,
        path::Path,
    },
};

pub const DEFAULT_MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

pub fn get_mmap<P: AsRef<Path>>(hay_path: P) -> io::Result<Mmap> {
    let file = File::open(hay_path.as_ref())?;
    let hay = unsafe { Mmap::map(&file)? };
    Ok(hay)
}

/// return the memmap to the file except if it was determined
/// that the file is binary (from its extension, size, or first bytes)
/// or is too big
pub fn get_mmap_if_suitable<P: AsRef<Path>>(hay_path: P, max_size: usize) -> io::Result<Option<Mmap>> {
    if let Some(ext) = hay_path.as_ref().extension().and_then(|s| s.to_str()) {
        if extensions::is_known_binary(ext) {
            return Ok(None);
        }
    }
    let hay = get_mmap(&hay_path)?;
    if hay.len() > max_size || magic_numbers::is_known_binary(&hay) {
        return Ok(None);
    }
    Ok(Some(hay))
}

/// return true when the file looks suitable for searching as text.
///
/// This function is quite slow as it creates a memmap just to check
/// a few bytes. If the memmap can be used, prefer `get_mmap_if_not_binary`
pub fn is_path_suitable<P: AsRef<Path>>(path: P, max_size: usize) -> bool {
    matches!(get_mmap_if_suitable(path, max_size), Ok(Some(_)))
}

pub fn line_count_at_pos<P: AsRef<Path>>(path: P, pos: usize) -> io::Result<usize> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut line = String::new();
    let mut line_count = 1;
    let mut bytes_count = 0;
    while reader.read_line(&mut line)? > 0 {
        bytes_count += line.len();
        if bytes_count >= pos {
            return Ok(line_count);
        }
        line_count += 1;
        line.clear();
    }
    Err(io::Error::new(io::ErrorKind::UnexpectedEof, "too short".to_string()))
}
