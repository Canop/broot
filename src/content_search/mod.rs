mod content_match;
mod content_search_result;
mod needle;

pub use {
    crate::content_type::{
        self,
        extensions,
        magic_numbers,
    },
    content_match::ContentMatch,
    content_search_result::ContentSearchResult,
    needle::Needle,
    std::io::{
        BufRead,
        BufReader,
    },
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
pub fn get_mmap_if_suitable<P: AsRef<Path>>(
    hay_path: P,
    max_size: usize,
) -> io::Result<Option<Mmap>> {
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
/// If a memmap will be needed afterwards, prefer to use `get_mmap_if_not_binary`
/// which optimizes testing and getting the mmap.
pub fn is_path_suitable<P: AsRef<Path>>(
    path: P,
    max_size: usize,
) -> bool {
    let path = path.as_ref();
    let Ok(metadata) = path.metadata() else {
        return false;
    };
    if metadata.len() > max_size as u64 {
        return false;
    }
    content_type::is_file_text(path).unwrap_or(false)
}

/// Return the 1-indexed line number for the byte at position pos
pub fn line_count_at_pos<P: AsRef<Path>>(
    path: P,
    pos: usize,
) -> io::Result<usize> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut line = String::new();
    let mut line_count = 1;
    let mut bytes_count = 0;
    while reader.read_line(&mut line)? > 0 {
        bytes_count += line.len();
        if bytes_count > pos {
            return Ok(line_count);
        }
        line_count += 1;
        line.clear();
    }
    Err(io::Error::new(
        io::ErrorKind::UnexpectedEof,
        "too short".to_string(),
    ))
}

#[cfg(test)]
mod line_count_at_pos_tests {
    use {
        super::line_count_at_pos,
        std::io::Write,
        tempfile::NamedTempFile,
    };

    /// Regression: a match whose first byte is exactly the first byte of a
    /// line (here `T` of `TARGET`, the first byte of line 2) must report the
    /// line it is actually on, not the line above. Before the fix,
    /// `bytes_count >= pos` returned line 1 because `bytes_count` after
    /// reading line 1 equals the byte index where line 2 begins.
    #[test]
    fn match_at_start_of_line_returns_that_line() {
        let mut tmp = NamedTempFile::new().unwrap();
        // byte indices: xxx=0..2, \n=3, T=4 (first byte of line 2)
        writeln!(tmp, "xxx\nTARGET\nrest").unwrap();
        tmp.flush().unwrap();
        // The exact-content search path passes the byte offset from Needle::search.
        // "TARGET" starts at byte index 4.
        let pos = "xxx\n".len(); // 4
        assert_eq!(pos, 4);
        let line = line_count_at_pos(tmp.path(), pos).unwrap();
        assert_eq!(
            line, 2,
            "a match starting at the first byte of line 2 must report line 2, not line 1"
        );
    }

    /// Guard against the opposite regression: a match in the MIDDLE of line 1
    /// must still report line 1 after switching `>=` to `>`.
    #[test]
    fn match_mid_first_line_still_returns_one() {
        let mut tmp = NamedTempFile::new().unwrap();
        writeln!(tmp, "hello TARGET world\nrest").unwrap();
        tmp.flush().unwrap();
        // "TARGET" starts at byte index 6, well inside line 1.
        let pos = "hello ".len(); // 6
        assert_eq!(pos, 6);
        let line = line_count_at_pos(tmp.path(), pos).unwrap();
        assert_eq!(line, 1);
    }

    /// Multi-line gap: a match at the first byte of line 4 must report 4,
    /// exercising the off-by-one across more than one preceding line.
    #[test]
    fn match_at_start_of_deeper_line() {
        let mut tmp = NamedTempFile::new().unwrap();
        writeln!(tmp, "a\nbb\nccc\nTARGET\nrest").unwrap();
        tmp.flush().unwrap();
        // bytes: 'a'=0 \n=1 'b'=2 'b'=3 \n=4 'c'=5 'c'=6 'c'=7 \n=8 'T'=9
        let pos = "a\nbb\nccc\n".len(); // 9
        assert_eq!(pos, 9);
        let line = line_count_at_pos(tmp.path(), pos).unwrap();
        assert_eq!(line, 4);
    }

    /// pos past EOF still yields the documented UnexpectedEof error (unchanged
    /// by the fix — locks the error contract).
    #[test]
    fn pos_past_eof_is_error() {
        let mut tmp = NamedTempFile::new().unwrap();
        writeln!(tmp, "short\n").unwrap();
        tmp.flush().unwrap();
        let err = line_count_at_pos(tmp.path(), 9999).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::UnexpectedEof);
    }

    /// A match at the first byte of a line that follows a BLANK line: the
    /// `pos == bytes_count` fall-through must keep counting past empty lines.
    /// `a\n\nTARGET\n` -> "TARGET" at byte 3 (first byte of line 3) -> 3.
    #[test]
    fn match_after_blank_line() {
        let mut tmp = NamedTempFile::new().unwrap();
        // bytes: 'a'=0 \n=1 \n=2 'T'=3  (T is first byte of line 3)
        writeln!(tmp, "a\n\nTARGET\n").unwrap();
        tmp.flush().unwrap();
        let pos = "a\n\n".len(); // 3
        assert_eq!(pos, 3);
        let line = line_count_at_pos(tmp.path(), pos).unwrap();
        assert_eq!(line, 3);
    }

    /// The matching line is the last line and has NO trailing newline:
    /// `read_line` still returns it (length without `\n`), so the boundary
    /// holds. Uses `write!` to avoid `writeln!` appending a final `\n`.
    #[test]
    fn match_on_last_line_without_trailing_newline() {
        let mut tmp = NamedTempFile::new().unwrap();
        // "xxx\nTARGET" with no final newline; T is first byte of line 2.
        write!(tmp, "xxx\nTARGET").unwrap();
        tmp.flush().unwrap();
        let pos = "xxx\n".len(); // 4
        assert_eq!(pos, 4);
        let line = line_count_at_pos(tmp.path(), pos).unwrap();
        assert_eq!(line, 2);
    }

    /// CRLF (`\r\n`) line endings: `read_line` includes the `\r` as line
    /// content, so after reading line N `bytes_count` is still the start
    /// offset of line N+1. Locks that the fix is CRLF-safe.
    #[test]
    fn match_at_start_of_line_with_crlf() {
        let mut tmp = NamedTempFile::new().unwrap();
        // "xxx\r\nTARGET\r\n"; bytes: x0 x1 x2 \r3 \n4 T5 (T = first byte of line 2)
        write!(tmp, "xxx\r\nTARGET\r\n").unwrap();
        tmp.flush().unwrap();
        let pos = "xxx\r\n".len(); // 5
        assert_eq!(pos, 5);
        let line = line_count_at_pos(tmp.path(), pos).unwrap();
        assert_eq!(line, 2);
    }
}
