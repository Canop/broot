use cli_log::*;

/// Parse a Primary Device Attributes reply (`CSI ? Ps ; Ps ; ... c`)
/// and report whether the Sixel attribute (`4`) is present.
pub(crate) fn parse_sixel_da1(response: &str) -> bool {
    let Some(start) = response.find("\x1b[?").map(|i| i + 3) else {
        return false;
    };
    let Some(rel_end) = response[start..].find('c') else {
        return false;
    };
    let body = &response[start..start + rel_end];
    body.split(';').any(|p| p == "4")
}

/// Probe the terminal for Sixel support via Primary Device Attributes.
/// Called at most once (cached by the GraphicsManager). Any failure -> false.
pub fn detect_sixel_support() -> bool {
    match query_da1(crate::graphics::terminal::TERMINAL_QUERY_TIMEOUT_MS) {
        Ok(reply) => {
            debug!("DA1 reply: {reply:?}");
            parse_sixel_da1(&reply)
        }
        Err(e) => {
            debug!("DA1 query failed: {e}");
            false
        }
    }
}

/// Issue a Primary Device Attributes query (`ESC [ c`) and return the raw reply.
///
/// `xterm_query::query` handles the per-platform terminal I/O (Unix `/dev/tty`,
/// Windows console) and returns an error on platforms without a backend, so any
/// failure simply resolves to "no Sixel" in `detect_sixel_support`.
fn query_da1(timeout_ms: u64) -> std::io::Result<String> {
    xterm_query::query("\x1b[c", timeout_ms)
        .map_err(|e| std::io::Error::other(format!("DA1 query failed: {e}")))
}

#[cfg(test)]
mod tests {
    use super::parse_sixel_da1;

    #[test]
    fn detects_sixel_capability() {
        // Primary DA reply: CSI ? 62 ; 4 ; 6 c  (4 = Sixel)
        assert!(parse_sixel_da1("\x1b[?62;4;6c"));
    }

    #[test]
    fn detects_sixel_among_many_params() {
        assert!(parse_sixel_da1("\x1b[?63;1;2;4;6;9;15c"));
    }

    #[test]
    fn rejects_without_sixel() {
        assert!(!parse_sixel_da1("\x1b[?62;1;6c"));
    }

    #[test]
    fn rejects_substring_false_positive() {
        // "14" or "40" must not count as "4"
        assert!(!parse_sixel_da1("\x1b[?62;14;40c"));
    }

    #[test]
    fn rejects_malformed() {
        assert!(!parse_sixel_da1("garbage"));
    }
}
