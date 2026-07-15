/// find and return the size of a cell (a char location) in pixels
/// as (width, height).
#[cfg(unix)]
pub fn cell_size_in_pixels() -> std::io::Result<(u32, u32)> {
    use libc::{
        STDOUT_FILENO,
        TIOCGWINSZ,
        c_ushort,
        ioctl,
    };
    // see http://www.delorie.com/djgpp/doc/libc/libc_495.html
    #[repr(C)]
    struct winsize {
        ws_row: c_ushort,    /* rows, in characters */
        ws_col: c_ushort,    /* columns, in characters */
        ws_xpixel: c_ushort, /* horizontal size, pixels */
        ws_ypixel: c_ushort, /* vertical size, pixels */
    }
    let mut w = winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    #[allow(clippy::useless_conversion)]
    let r = unsafe { ioctl(STDOUT_FILENO, TIOCGWINSZ.into(), &mut w) };
    if r == 0 && w.ws_xpixel > w.ws_col && w.ws_ypixel > w.ws_row {
        Ok((
            (w.ws_xpixel / w.ws_col) as u32,
            (w.ws_ypixel / w.ws_row) as u32,
        ))
    } else {
        // ioctl pixel sizes aren't guaranteed (telnet, some SSH setups,
        // GNU screen, ... leave them zero): ask the terminal directly.
        debug!("no usable cell size from ioctl; falling back to CSI 16 t query");
        cell_size_via_query()
    }
}

#[cfg(windows)]
pub fn cell_size_in_pixels() -> std::io::Result<(u32, u32)> {
    cell_size_via_query()
}

#[cfg(all(not(unix), not(windows)))]
pub fn cell_size_in_pixels() -> std::io::Result<(u32, u32)> {
    Err(std::io::Error::other("fetching cell size isn't supported on this platform"))
}

/// Query the terminal for the cell size via `CSI 16 t` ("report character
/// cell size in pixels"; reply `ESC [ 6 ; height ; width t`). Primary source
/// on Windows (Windows Terminal 1.22+), fallback on Unix when ioctl reports
/// no usable pixel sizes.
#[cfg(any(unix, windows))]
fn cell_size_via_query() -> std::io::Result<(u32, u32)> {
    use std::io;
    let response = xterm_query::query("\x1b[16t", crate::graphics::terminal::TERMINAL_QUERY_TIMEOUT_MS)
        .map_err(|e| io::Error::other(format!("cell-size query failed: {e}")))?;
    parse_cell_size(&response)
        .ok_or_else(|| io::Error::other("unparseable cell-size reply"))
}

/// Parse a `CSI 16 t` reply of the form `ESC [ 6 ; height ; width t`
/// into `(width, height)` in pixels. Returns `None` if it isn't a
/// well-formed cell-size (kind `6`) report.
#[cfg(any(unix, windows, test))]
pub(crate) fn parse_cell_size(response: &str) -> Option<(u32, u32)> {
    let body = response
        .trim_start_matches('\u{1b}')
        .trim_start_matches('[')
        .trim_end_matches('t');
    let mut parts = body.split(';');
    if parts.next()? != "6" {
        return None;
    }
    let height: u32 = parts.next()?.parse().ok()?;
    let width: u32 = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    if width == 0 || height == 0 {
        return None;
    }
    Some((width, height))
}

#[cfg(test)]
mod tests {
    use super::parse_cell_size;

    #[test]
    fn parses_csi_16t_response() {
        // CSI 6 ; height ; width t  -> returns (width, height)
        assert_eq!(parse_cell_size("\x1b[6;20;10t"), Some((10, 20)));
    }

    #[test]
    fn parses_response_without_leading_esc() {
        assert_eq!(parse_cell_size("6;20;10t"), Some((10, 20)));
    }

    #[test]
    fn rejects_wrong_kind() {
        // 4 = text-area-in-pixels, not cell size
        assert_eq!(parse_cell_size("\x1b[4;200;100t"), None);
    }

    #[test]
    fn rejects_malformed() {
        assert_eq!(parse_cell_size("garbage"), None);
        assert_eq!(parse_cell_size("\x1b[6;20t"), None);
    }
}
