//! This module is where is defined whether broot
//! writes on stdout, on stderr or elsewhere. It also provides helper
//! structs for io.

/// declare a style named `$dst` which is usually a reference to the `$src`
/// skin but, in case `selected` is true, is a clone with background changed
/// to the one of selected lines.
#[macro_export]
macro_rules! cond_bg {
    ($dst:ident, $self:ident, $selected:expr, $src:expr) => {
        let mut cloned_style;
        let $dst = if $selected {
            cloned_style = $src.clone();
            if let Some(c) = $self.skin.selected_line.get_bg() {
                cloned_style.set_bg(c);
            }
            &cloned_style
        } else {
            &$src
        };
    };
}

mod areas;
mod col;
mod crop_writer;
mod displayable_tree;
mod filling;
mod git_status_display;
pub mod flags_display;
pub mod status_line;
mod matched_string;
mod screen;
mod cell_size;

#[cfg(not(any(target_family="windows",target_os="android")))]
mod permissions;

pub use {
    areas::Areas,
    col::*,
    cond_bg,
    crop_writer::CropWriter,
    displayable_tree::DisplayableTree,
    filling::*,
    git_status_display::GitStatusDisplay,
    matched_string::MatchedString,
    screen::Screen,
    cell_size::*,
};
use {
    crate::{
        errors::ProgramError,
    },
    crossterm::{
        style::{
            Color,
            SetBackgroundColor,
        },
        QueueableCommand,
    },
    once_cell::sync::Lazy,
};

#[cfg(not(any(target_family="windows",target_os="android")))]
pub use {
    permissions::PermWriter,
};

pub static TAB_REPLACEMENT: &str = "  ";

pub static SPACE_FILLING: Lazy<Filling> = Lazy::new(|| { Filling::from_char(' ') });
pub static BRANCH_FILLING: Lazy<Filling> = Lazy::new(|| { Filling::from_char('─') });

/// if true then the status of a panel covers the whole width
/// of the terminal (over the other panels)
pub const WIDE_STATUS: bool = true;

/// the type used by all GUI writing functions
pub type W = std::io::BufWriter<std::io::Stderr>;

/// return the writer used by the application
pub fn writer() -> W {
    std::io::BufWriter::new(std::io::stderr())
}

pub fn fill_bg(
    w: &mut W,
    len: usize,
    bg: Color,
) -> Result<(), ProgramError> {
    w.queue(SetBackgroundColor(bg))?;
    SPACE_FILLING.queue_unstyled(w, len)?;
    Ok(())
}
