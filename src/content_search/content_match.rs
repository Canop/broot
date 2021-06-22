
/// a displayable representation of where
/// the needle was found, with some text around
#[derive(Debug, Clone)]
pub struct ContentMatch {
    pub extract: String,
    pub needle_start: usize, // position in the extract, in bytes
    pub needle_end: usize,   // length in bytes
}

impl ContentMatch {
    pub fn build(
        hay: &[u8],
        pos: usize, // position in the hay
        needle: &str,
        desired_len: usize, // max length of the extract in bytes
    ) -> Self {
        if hay.is_empty() {
            // this happens if you search `cr/.*` and a file starts with an empty line
            return Self {
                extract: "".to_string(),
                needle_start: 0,
                needle_end: 0,
            };
        }
        let mut extract_start = pos;
        let mut extract_end = pos + needle.len(); // not included
        loop {
            if extract_start == 0 || extract_end - extract_start >= desired_len / 2 {
                break;
            }
            let c = hay[extract_start - 1];
            if c < 32 {
                break;
            }
            extract_start -= 1;
        }
        // left trimming
        while (hay[extract_start] == 32) && extract_start < pos {
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
        // at this point we're unsure whether we start at a correct char boundary, hence
        // the from_utf8_lossy
        let extract = String::from_utf8_lossy(&hay[extract_start..extract_end]).to_string();
        let needle_start = extract.find(needle).unwrap_or(0);
        Self {
            extract,
            needle_start,
            needle_end: needle_start + needle.len(),
        }
    }
}
