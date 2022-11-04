// Don't look here for search functions to reuse or even for
// efficient or proven tricks. This is fast-made, and not fit
// for reuse out of broot.

use {
    super::*,
    memmap2::Mmap,
    std::{
        convert::TryInto,
        fmt,
        io,
        path::{Path},
    },
};

/// a strict (non fuzzy, case sensitive) pattern which may
/// be searched in file contents
#[derive(Clone)]
pub struct Needle {

    /// bytes of the searched string
    /// (guaranteed to be valid UTF8 by construct)
    bytes: Box<[u8]>,

    max_file_size: usize,
}

impl fmt::Debug for Needle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Needle")
            .field("bytes", &self.bytes)
            .finish()
    }
}


impl Needle {

    pub fn new(pat: &str, max_file_size: usize) -> Self {
        let bytes = pat.as_bytes().to_vec().into_boxed_slice();
        Self { bytes, max_file_size }
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.bytes) }
    }

    // no, it doesn't bring more than a few % in speed
    fn find_naive_1(&self, hay: &Mmap) -> Option<usize> {
        let n = self.bytes[0];
        hay.iter().position(|&b| b == n)
    }

    fn find_naive_2(&self, mut pos: usize, hay: &Mmap) -> Option<usize> {
        let max_pos = hay.len() - 2;
        let b0 = self.bytes[0];
        let b1 = self.bytes[1];
        unsafe {
            while pos <= max_pos {
                if *hay.get_unchecked(pos) == b0 && *hay.get_unchecked(pos + 1) == b1 {
                    return Some(pos);
                }
                pos += 1;
            }
        }
        None
    }

    fn find_naive_3(&self, mut pos: usize, hay: &Mmap) -> Option<usize> {
        let max_pos = hay.len() - 3;
        let b0 = self.bytes[0];
        let b1 = self.bytes[1];
        let b2 = self.bytes[2];
        unsafe {
            while pos <= max_pos {
                if *hay.get_unchecked(pos) == b0
                    && *hay.get_unchecked(pos + 1) == b1
                    && *hay.get_unchecked(pos + 2) == b2
                {
                    return Some(pos);
                }
                pos += 1;
            }
        }
        None
    }

    fn find_naive_4(&self, mut pos: usize, hay: &Mmap) -> Option<usize> {
        use std::mem::transmute;
        let max_pos = hay.len() - 4;
        unsafe {
            let needle: u32 = transmute::<[u8; 4], u32>((&*self.bytes).try_into().unwrap());
            while pos <= max_pos {
                if transmute::<[u8; 4], u32>((&hay[pos..pos + 4]).try_into().unwrap()) == needle {
                    return Some(pos);
                }
                pos += 1;
            }
        }
        None
    }

    fn find_naive_6(&self, mut pos: usize, hay: &Mmap) -> Option<usize> {
        let max_pos = hay.len() - 6;
        let b0 = self.bytes[0];
        let b1 = self.bytes[1];
        let b2 = self.bytes[2];
        let b3 = self.bytes[3];
        let b4 = self.bytes[4];
        let b5 = self.bytes[5];
        unsafe {
            while pos <= max_pos {
                if *hay.get_unchecked(pos) == b0
                    && *hay.get_unchecked(pos + 1) == b1
                    && *hay.get_unchecked(pos + 2) == b2
                    && *hay.get_unchecked(pos + 3) == b3
                    && *hay.get_unchecked(pos + 4) == b4
                    && *hay.get_unchecked(pos + 5) == b5
                {
                    return Some(pos);
                }
                pos += 1;
            }
        }
        None
    }

    fn is_at_pos(&self, hay_stack: &Mmap, pos: usize) -> bool {
        unsafe {
            for (i, b) in self.bytes.iter().enumerate() {
                if hay_stack.get_unchecked(i + pos) != b {
                    return false;
                }
            }
        }
        true
    }

    fn find_naive(&self, mut pos: usize, hay: &Mmap) -> Option<usize> {
        let max_pos = hay.len() - self.bytes.len();
        while pos <= max_pos {
            if self.is_at_pos(hay, pos) {
                return Some(pos);
            }
            pos += 1;
        }
        None
    }

    /// search the mem map to find the first occurrence of the needle.
    ///
    /// Known limit: if the file has an encoding where the needle would
    /// be represented in a way different than UTF-8, the needle won't
    /// be found (I noticed the problem with other grepping tools, too,
    /// which is understandable as detecting the encoding and translating
    /// the needle would multiply the search time).
    ///
    ///
    /// The exact search algorithm used here (I removed Boyer-Moore)
    /// and the optimizations (loop unrolling, etc.) don't really matter
    /// as their impact is dwarfed by the whole mem map related set
    /// of problems. An alternate implementation should probably focus
    /// on avoiding mem maps.
    fn search_mmap(&self, hay: &Mmap) -> ContentSearchResult {
        if hay.len() < self.bytes.len() {
            return ContentSearchResult::NotFound;
        }

        // we tell the system how we intent to use the mmap
        // to increase the likehod the memory is available
        // for our loop
        #[cfg(not(any(target_family = "windows", target_os = "android")))]
        unsafe {
            libc::posix_madvise(
                hay.as_ptr() as *mut std::ffi::c_void,
                hay.len(),
                libc::POSIX_MADV_SEQUENTIAL,
            );
            // TODO the Windows equivalent might be PrefetchVirtualMemory
        }

        let pos = match self.bytes.len() {
            1 => self.find_naive_1(hay),
            2 => self.find_naive_2(0, hay),
            3 => self.find_naive_3(0, hay),
            4 => self.find_naive_4(0, hay),
            6 => self.find_naive_6(0, hay),
            _ => self.find_naive(0, hay),
        };
        pos.map_or(
            ContentSearchResult::NotFound,
            |pos| ContentSearchResult::Found { pos },
        )
    }

    /// determine whether the file contains the needle
    pub fn search<P: AsRef<Path>>(&self, hay_path: P) -> io::Result<ContentSearchResult> {
        super::get_mmap_if_suitable(hay_path, self.max_file_size)
            .map(|om| om.map_or(
                ContentSearchResult::NotSuitable,
                |hay| self.search_mmap(&hay),
            ))
    }

    /// this is supposed to be called only when it's known that there's
    /// a match
    pub fn get_match<P: AsRef<Path>>(
        &self,
        hay_path: P,
        desired_len: usize,
    ) -> Option<ContentMatch> {
        let hay = match get_mmap(hay_path) {
            Ok(hay) => hay,
            _ => { return None; }
        };
        match self.search_mmap(&hay) {
            ContentSearchResult::Found { pos } => {
                Some(ContentMatch::build(&hay, pos, self.as_str(), desired_len))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod content_search_tests {
    use super::*;

    #[test]
    fn test_found() -> Result<(), io::Error> {
        let needle = Needle::new("inception", 1_000_000);
        let res = needle.search("src/content_search/needle.rs")?;
        assert!(res.is_found());
        Ok(())
    }
}
