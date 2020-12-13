mod app_skin;
mod cli_mad_skin;
pub mod colors;
mod ext_colors;
mod help_mad_skin;
mod panel_skin;
mod purpose_mad_skin;
mod skin_entry;
mod style_map;
mod status_mad_skin;

pub use {
    app_skin::AppSkin,
    cli_mad_skin::*,
    ext_colors::ExtColorMap,
    help_mad_skin::*,
    panel_skin::PanelSkin,
    purpose_mad_skin::*,
    skin_entry::SkinEntry,
    style_map::{StyleMap, StyleMaps},
    status_mad_skin::StatusMadSkinSet,
};

use crossterm::style::Color::{self, *};

pub fn gray(level: u8) -> Option<Color> {
    Some(AnsiValue(0xE8 + level))
}

pub fn rgb(r: u8, g: u8, b: u8) -> Option<Color> {
    Some(Rgb { r, g, b })
}

pub fn ansi(v: u8) -> Option<Color> {
    Some(AnsiValue(v))
}
