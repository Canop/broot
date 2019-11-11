/// This module builds Termimad `MadSkin` from Broot `Skin`

use termimad::{
    Alignment,
    LineStyle,
    MadSkin,
};

use crate::{
    skin::Skin,
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
