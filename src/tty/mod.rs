mod tline;
mod tline_builder;
mod trange;
mod tstring;
mod tty_view;

pub const CSI_RESET: &str = "\u{1b}[0m";
pub const CSI_BOLD: &str = "\u{1b}[1m";
pub const CSI_ITALIC: &str = "\u{1b}[3m";

static TAB_REPLACEMENT: &str = "    ";

use {
    crate::{
        display::W,
        errors::ProgramError,
    },
    std::io::Write,
};

pub use {
    tline::*,
    tline_builder::*,
    trange::*,
    tstring::*,
    tty_view::*,
};

fn draw(
    w: &mut W,
    csi: &str,
    raw: &str,
) -> Result<(), ProgramError> {
    if csi.is_empty() {
        write!(w, "{}", raw)?;
    } else {
        write!(w, "{}{}{}", csi, raw, CSI_RESET,)?;
    }
    Ok(())
}
