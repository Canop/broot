
use {
    super::StyleMap,
    termimad::{Alignment, LineStyle, MadSkin},
};

/// build a MadSkin which will be used to display the status
/// when there's no error
pub fn make_purpose_mad_skin(skin: &StyleMap) -> MadSkin {
    MadSkin {
        paragraph: LineStyle::new(
            skin.purpose_normal.clone(),
            Alignment::Left,
        ),
        italic: skin.purpose_italic.clone(),
        bold: skin.purpose_bold.clone(),
        ellipsis: skin.purpose_ellipsis.clone(),
        ..Default::default()
    }
}

