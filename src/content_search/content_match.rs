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
                extract: String::new(),
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
        while extract_start < pos && hay[extract_start] == 32 {
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

#[cfg(test)]
mod content_match_tests {
    use super::*;

    /// Regression: an empty match at the very end of a line (eg searching
    /// `cr/$/` in a file whose line ends with a control char, such as a tab
    /// or the `\r` of a CRLF line ending) gives `pos == hay.len()`. The left
    /// trimming loop used to read `hay[extract_start]` before checking
    /// `extract_start < pos`, panicking with "index out of bounds".
    #[test]
    fn empty_match_at_end_of_line_doesnt_panic() {
        // line ending with a tab: the extract_start loop stops on the control
        // char, leaving extract_start == pos == hay.len()
        let m = ContentMatch::build(b"abc\t", 4, "", 30);
        assert_eq!(m.extract, "");
        assert_eq!(m.needle_start, 0);
        assert_eq!(m.needle_end, 0);
        // same with the `\r` of a CRLF line ending
        let m = ContentMatch::build(b"hello\r", 6, "", 30);
        assert_eq!(m.extract, "");
    }

    /// The left trimming must still drop the spaces before the needle.
    #[test]
    fn left_trimming_still_drops_leading_spaces() {
        let m = ContentMatch::build(b"\t   needle here", 7, "needle", 30);
        assert_eq!(m.extract, "needle here");
        assert_eq!(m.needle_start, 0);
        assert_eq!(m.needle_end, 6);
    }

    /// Trimming must not eat the needle itself when the needle starts with a
    /// space: it stops at `pos`.
    #[test]
    fn left_trimming_stops_at_needle() {
        // bytes: \t=0 ' '=1 ' '=2 'a'=3 ' '=4 'x'=5, needle " a" at pos 2
        let m = ContentMatch::build(b"\t  a x", 2, " a", 30);
        assert_eq!(m.extract, " a x");
        assert_eq!(m.needle_start, 0);
    }
}
