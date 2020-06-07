use {
    super::*,
    memmap::Mmap,
    std::{
        fs::File,
        io,
        path::{Path},
    },
};


/// a strict (non fuzzy, case sensitive) pattern which may
/// be searched in file contents
#[derive(Debug, Clone)]
pub struct Needle {

    /// bytes of the searched string
    /// (guaranteed to be valid UTF8 by construct)
    bytes: Box<[u8]>,
}

impl Needle {

    pub fn new(pat: &str) -> Self {
        Self {
            bytes: pat.as_bytes().to_vec().into_boxed_slice(),
        }
    }

    pub fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.bytes) }
    }

    fn is_at_pos(&self, hay_stack: &Mmap, pos: usize) -> bool {
        for (i, b) in self.bytes.iter().enumerate() {
            if hay_stack[i+pos] != *b {
                return false;
            }
        }
        true
    }

    /// placeholder implementation. I'll do a faster one once I've solved
    /// the other application problems related to content searches
    pub fn search<P: AsRef<Path>>(&self, hay_path: P) -> io::Result<ContentSearchResult> {
        let file = File::open(hay_path.as_ref())?;
        let hay = unsafe { Mmap::map(&file)? };
        if hay.len() > MAX_FILE_SIZE {
            return Ok(ContentSearchResult::NotSuitable);
        }
        if magic_numbers::is_known_binary(&hay) {
            return Ok(ContentSearchResult::NotSuitable);
        }
        if hay.len() < self.bytes.len() {
            return Ok(ContentSearchResult::NotFound)
        }
        let n = hay.len() - self.bytes.len();
        for pos in 0..n {
            if self.is_at_pos(&hay, pos) {
                return Ok(ContentSearchResult::Found { pos });
            }
        }
        Ok(ContentSearchResult::NotFound)
    }
}

#[cfg(test)]
mod content_search_tests {

    use super::*;

    #[test]
    fn test_found() -> Result<(), io::Error> {
        let needle = Needle::new("inception");
        let res = needle.search("src/content_search/needle.rs")?;
        assert!(res.is_found());
        Ok(())
    }
}
