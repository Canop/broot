mod detect_support;
mod image_renderer;

pub use image_renderer::*;

use crate::{
    app::AppContext,
    graphics::terminal::is_tmux,
};

pub use detect_support::detect_kitty_graphics_protocol_display;

/// Build a Kitty renderer if this terminal supports the protocol.
pub fn build_kitty_renderer(con: &AppContext) -> Option<KittyImageRenderer> {
    let options = KittyImageRendererOptions {
        display: con.kitty_graphics_display,
        transmission_medium: con.kitty_graphics_transmission,
        kept_temp_files: con.kept_kitty_temp_files,
        is_tmux: is_tmux(),
    };
    KittyImageRenderer::new(options)
}
