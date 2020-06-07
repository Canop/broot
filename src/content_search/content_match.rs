use {
    memmap::Mmap,
};

/// a displayable representation of where
/// the needle was found, with some text around
#[derive(Debug)]
pub struct ContentMatch {
    pub extract: String,
    pub needle_start: usize, // position in the extract, in bytes
    pub needle_end: usize, // length in bytes
}

impl ContentMatch {
    pub fn build(
        hay: &Mmap,
        pos: usize, // position in the hay
        needle_len: usize, // in bytes
        desired_len: usize, // max length of the extract
    ) -> Self {
        let mut extract_start = pos;
        let mut extract_end = pos + needle_len; // not included
        loop {
            if extract_start == 0 || extract_end - extract_start >= desired_len / 2 {
                break;
            }
            let c = hay[extract_start-1];
            if c < 32 {
                break;
            }
            extract_start -= 1;
        }
        // left trimming
        while hay[extract_start]==32 && extract_start<pos {
            extract_start += 1;
        }
        loop {
            if extract_end == hay.len() || extract_end - extract_start >= desired_len {
                break;
            }
            let c = hay[extract_end];
            if c < 32 {
                break;
            }
            extract_end += 1;
        }
        let extract = String::from_utf8((&hay[extract_start..extract_end]).to_vec())
            .unwrap_or_else(|_| "invalid UTF8".to_string());
        let needle_start = pos - extract_start;
        Self {
            extract,
            needle_start,
            needle_end: needle_start + needle_len,
        }
    }
}
