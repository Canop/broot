use {
    super::StyleMap,
    termimad::{
        Alignment,
        LineStyle,
        MadSkin,
    },
};

/// build a MadSkin which will be used to display the status
/// when there's no error
pub fn make_purpose_mad_skin(skin: &StyleMap) -> MadSkin {
    MadSkin {
        paragraph: LineStyle::new(skin.purpose_normal, Alignment::Left),
        italic: skin.purpose_italic,
        bold: skin.purpose_bold,
        ellipsis: skin.purpose_ellipsis,
        ..Default::default()
    }
}
