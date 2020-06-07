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

fn get_hay<P: AsRef<Path>>(hay_path: P) -> io::Result<Mmap> {
    let file = File::open(hay_path.as_ref())?;
    let hay = unsafe { Mmap::map(&file)? };
    Ok(hay)
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
        unsafe {
            for (i, b) in self.bytes.iter().enumerate() {
                if *hay_stack.get_unchecked(i+pos) != *b {
                    return false;
                }
            }
        }
        true
    }

    /// placeholder implementation. I'll do a faster one once I've solved
    /// the other application problems related to content searches
    fn search_hay(&self, hay: &Mmap) -> ContentSearchResult {
        if hay.len() > MAX_FILE_SIZE {
            return ContentSearchResult::NotSuitable;
        }
        if magic_numbers::is_known_binary(&hay) {
            return ContentSearchResult::NotSuitable;
        }
        if hay.len() < self.bytes.len() {
            return ContentSearchResult::NotFound;
        }
        let n = hay.len() - self.bytes.len();
        for pos in 0..n {
            if self.is_at_pos(&hay, pos) {
                return ContentSearchResult::Found { pos };
            }
        }
        ContentSearchResult::NotFound
    }

    /// determine whether the file contains the needle
    pub fn search<P: AsRef<Path>>(&self, hay_path: P) -> io::Result<ContentSearchResult> {
        let hay = get_hay(hay_path)?;
        Ok(self.search_hay(&hay))
    }

    /// this is supposed to be called only when it's known that there's
    /// a match
    pub fn get_match<P: AsRef<Path>>(
        &self,
        hay_path: P,
        desired_len: usize,
    ) -> Option<ContentMatch> {
        let hay = match get_hay(hay_path) {
            Ok(hay) => hay,
            _ => { return None; }
        };
        match self.search_hay(&hay) {
            ContentSearchResult::Found { pos } => Some(ContentMatch::build(
                &hay, pos, self.bytes.len(), desired_len,
            )),
            _ => None,
        }
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
