
use {
    super::StyleMap,
    termimad::{Alignment, LineStyle, MadSkin},
};

/// build a MadSkin which will be used to display the status
/// when there's no error
pub fn make_purpose_mad_skin(skin: &StyleMap) -> MadSkin {
    let mut mad_skin = MadSkin::default();
    mad_skin.paragraph = LineStyle {
        compound_style: skin.purpose_normal.clone(),
        align: Alignment::Left,
    };
    mad_skin.italic = skin.purpose_italic.clone();
    mad_skin.bold = skin.purpose_bold.clone();
    mad_skin.ellipsis = skin.purpose_ellipsis.clone();
    mad_skin
}

