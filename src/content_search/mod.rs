
mod content_match;
mod content_search_result;
mod magic_numbers;
mod extensions;
mod needle;

pub use {
    content_match::ContentMatch,
    content_search_result::ContentSearchResult,
    needle::Needle,
};

use {
    memmap::Mmap,
    std::{
        fs::File,
        io,
        path::Path,
    },
};

pub const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

pub fn get_mmap<P: AsRef<Path>>(hay_path: P) -> io::Result<Mmap> {
    let file = File::open(hay_path.as_ref())?;
    let hay = unsafe { Mmap::map(&file)? };
    Ok(hay)
}

/// return the memmap to the file except if it was determined
/// that the file is binary (from its extension, size, or first bytes)
pub fn get_mmap_if_not_binary<P: AsRef<Path>>(hay_path: P) -> io::Result<Option<Mmap>> {
    if let Some(ext) = hay_path.as_ref().extension().and_then(|s| s.to_str()) {
        if extensions::is_known_binary(&ext) {
            return Ok(None);
        }
    }
    let hay = get_mmap(&hay_path)?;
    if hay.len() > MAX_FILE_SIZE || magic_numbers::is_known_binary(&hay) {
        return Ok(None);
    }
    Ok(Some(hay))
}

/// return false when the file looks suitable for searching as text.
///
/// This function is quite slow as it creates a memmap just to check
/// a few bytes. If the memmap can be used, prefer `get_mmap_if_not_binary`
pub fn is_path_binary<P: AsRef<Path>>(path: P) -> bool {
    match get_mmap_if_not_binary(path) {
        Ok(Some(_)) => false,
        _ => true,
    }
}
