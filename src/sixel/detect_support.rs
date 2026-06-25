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

/// Parse an XTSMGRAPHICS reply for the Sixel-geometry item
/// (`CSI ? 2 ; 0 ; Pw ; Ph S`) into `(Pw, Ph)` = the maximum Sixel image size
/// in pixels. Returns `None` for a different item, a non-success status, or a
/// malformed reply.
pub(crate) fn parse_xtsmgraphics(response: &str) -> Option<(u32, u32)> {
    let start = response.find("\x1b[?")? + 3;
    let rel_end = response[start..].find('S')?;
    let mut parts = response[start..start + rel_end].split(';');
    if parts.next()? != "2" {
        return None; // item 2 = Sixel graphics geometry
    }
    if parts.next()? != "0" {
        return None; // status 0 = success
    }
    let width: u32 = parts.next()?.parse().ok()?;
    let height: u32 = parts.next()?.parse().ok()?;
    if parts.next().is_some() || width == 0 || height == 0 {
        return None;
    }
    Some((width, height))
}

/// Query the terminal's maximum Sixel image geometry via XTSMGRAPHICS
/// (`CSI ? 2 ; 1 ; 0 S`). Returns the max `(width, height)` in pixels, or `None`
/// if the terminal reports no limit / doesn't support the query.
///
/// Some terminals (e.g. xterm) silently crop Sixel images larger than this, so
/// callers should scale images to fit within it.
pub fn detect_sixel_max_geometry() -> Option<(u32, u32)> {
    match query_xtsmgraphics(crate::graphics::terminal::TERMINAL_QUERY_TIMEOUT_MS) {
        Ok(reply) => {
            debug!("XTSMGRAPHICS reply: {reply:?}");
            parse_xtsmgraphics(&reply)
        }
        Err(e) => {
            debug!("XTSMGRAPHICS query failed: {e}");
            None
        }
    }
}

fn query_xtsmgraphics(timeout_ms: u64) -> std::io::Result<String> {
    xterm_query::query("\x1b[?2;1;0S", timeout_ms)
        .map_err(|e| std::io::Error::other(format!("XTSMGRAPHICS query failed: {e}")))
}

#[cfg(test)]
mod tests {
    use super::{parse_sixel_da1, parse_xtsmgraphics};

    #[test]
    fn parses_sixel_geometry() {
        // CSI ? 2 ; 0 ; Pw ; Ph S  (status 0 = success)
        assert_eq!(parse_xtsmgraphics("\x1b[?2;0;1000;1000S"), Some((1000, 1000)));
        assert_eq!(parse_xtsmgraphics("\x1b[?2;0;1920;1080S"), Some((1920, 1080)));
    }

    #[test]
    fn rejects_failure_status() {
        // status 3 = unsupported
        assert_eq!(parse_xtsmgraphics("\x1b[?2;3;0;0S"), None);
    }

    #[test]
    fn rejects_wrong_item() {
        // item 1 = color registers, not geometry
        assert_eq!(parse_xtsmgraphics("\x1b[?1;0;256S"), None);
    }

    #[test]
    fn rejects_malformed_geometry() {
        assert_eq!(parse_xtsmgraphics("garbage"), None);
        assert_eq!(parse_xtsmgraphics("\x1b[?2;0;1000S"), None); // missing height
        assert_eq!(parse_xtsmgraphics("\x1b[?2;0;0;0S"), None); // zero dims
    }

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
