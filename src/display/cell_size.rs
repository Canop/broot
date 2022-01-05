
/// find and return the size of a cell (a char location) in pixels
/// as (width, height).
/// Many terminals don't fill this information correctly, so an
/// error is expected (it works on kitty, where I use the data
/// to compute the rendering dimensions of images)
#[cfg(unix)]
pub fn cell_size_in_pixels() -> std::io::Result<(u32, u32)> {
    use {
        libc::{
            c_ushort,
            ioctl,
            STDOUT_FILENO,
            TIOCGWINSZ,
        },
        std::io,
    };
    // see http://www.delorie.com/djgpp/doc/libc/libc_495.html
    #[repr(C)]
    struct winsize {
        ws_row: c_ushort,    /* rows, in characters */
        ws_col: c_ushort,    /* columns, in characters */
        ws_xpixel: c_ushort, /* horizontal size, pixels */
        ws_ypixel: c_ushort, /* vertical size, pixels */
    }
    let w = winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    #[allow(clippy::useless_conversion)]
    let r = unsafe { ioctl(STDOUT_FILENO, TIOCGWINSZ.into(), &w) };
    if r == 0 && w.ws_xpixel > w.ws_col && w.ws_ypixel > w.ws_row {
        Ok((
            (w.ws_xpixel / w.ws_col) as u32,
            (w.ws_ypixel / w.ws_row) as u32,
        ))
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "failed to fetch terminal dimension with ioctl",
        ))
    }
}

#[cfg(not(unix))]
pub fn cell_size_in_pixels() -> std::io::Result<(u32, u32)> {
    // there's probably a way but I don't know it
    Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "fetching cell size isn't supported on Windows",
    ))
}
