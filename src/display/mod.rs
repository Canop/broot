//! This module is where is defined whether broot
//! writes on stdout, on stderr or elsewhere. It also provides helper
//! structs for io.


/// declare a style named `$dst` which is usually a reference to the `$src`
/// skin but, in case `selected` is true, is a clone with background changed
/// to the one of selected lines.
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
mod crop_writer;
mod displayable_tree;
mod screen;
mod status;
mod git_status_display;

use std::{
    io::BufWriter,
};

pub use {
    areas::Areas,
    crop_writer::CropWriter,
    displayable_tree::DisplayableTree,
    screen::Screen,
    status::Status,
    git_status_display::GitStatusDisplay,
};

pub static FLAGS_AREA_WIDTH: u16 = 10;

/// the type used by all GUI writing functions
pub type W = BufWriter<std::io::Stderr>;

/// return the writer used by the application
pub fn writer() -> W {
    BufWriter::new(std::io::stderr())
}



