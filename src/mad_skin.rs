/// This module builds Termimad `MadSkin` from Broot `Skin`

use {
    crate::{
        skin::Skin,
    },
    crossterm::{
        style::Color,
    },
    termimad::{
        Alignment,
        CompoundStyle,
        LineStyle,
        gray,
        MadSkin,
    },
};

/// the mad skin applying to the status depending whether it's an
/// error or not
pub struct StatusMadSkinSet {
    pub normal: MadSkin,
    pub error: MadSkin,
}

/// build a MadSkin which will be used to display the status
/// when there's no error
fn make_normal_status_mad_skin(skin: &Skin) -> MadSkin {
    let mut mad_skin = MadSkin::default();
    mad_skin.paragraph = LineStyle {
        compound_style: skin.status_normal.clone(),
        align: Alignment::Left,
    };
    mad_skin.italic = skin.status_italic.clone();
    mad_skin.bold = skin.status_bold.clone();
    mad_skin.inline_code = skin.status_code.clone();
    mad_skin.ellipsis = skin.status_ellipsis.clone();
    mad_skin
}

/// build a MadSkin which will be used to display the status
/// when there's a error
fn make_error_status_mad_skin(skin: &Skin) -> MadSkin {
    let mut mad_skin = MadSkin::default();
    mad_skin.paragraph = LineStyle {
        compound_style: skin.status_error.clone(),
        align: Alignment::Left,
    };
    mad_skin.ellipsis = skin.status_ellipsis.clone();
    mad_skin
}

impl StatusMadSkinSet {
    pub fn from_skin(skin: &Skin) -> Self {
        Self {
            normal: make_normal_status_mad_skin(skin),
            error: make_error_status_mad_skin(skin),
        }
    }
}

/// build a MadSkin, which will be used for markdown formatting
/// for the help screen by applying the `help_*` entries
/// of the skin.
pub fn make_help_mad_skin(skin: &Skin) -> MadSkin {
    let mut ms = MadSkin::default();
    ms.paragraph.compound_style = CompoundStyle::from(skin.help_paragraph.clone());
    ms.inline_code = CompoundStyle::from(skin.help_code.clone());
    ms.code_block.compound_style = ms.inline_code.clone();
    ms.bold = CompoundStyle::from(skin.help_bold.clone());
    ms.italic = CompoundStyle::from(skin.help_italic.clone());
    ms.table = LineStyle {
        compound_style: CompoundStyle::from(skin.help_table_border.clone()),
        align: Alignment::Center,
    };
    if let Some(c) = skin.help_headers.get_fg() {
        ms.set_headers_fg(c);
    }
    if let Some(c) = skin.help_headers.get_bg() {
        ms.set_headers_bg(c);
    }
    ms.bullet.set_compound_style(ms.paragraph.compound_style.clone());
    ms.scrollbar
        .track
        .set_compound_style(CompoundStyle::from(skin.scrollbar_track.clone()));
    ms.scrollbar
        .thumb
        .set_compound_style(CompoundStyle::from(skin.scrollbar_thumb.clone()));
    ms
}

/// build a termimad skin for cli output (mostly
/// for the install process)
pub fn make_cli_mad_skin() -> MadSkin {
    let mut skin = MadSkin::default();
    skin.set_headers_fg(Color::AnsiValue(178));
    skin.bold.set_fg(gray(12));
    skin.inline_code.set_bg(gray(2));
    skin.inline_code.set_fg(gray(18));
    skin.code_block.set_bg(gray(2));
    skin.code_block.set_fg(gray(18));
    skin.italic.set_fg(Color::Magenta);
    skin
}
