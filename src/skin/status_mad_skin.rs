use {
    super::StyleMap,
    termimad::{Alignment, LineStyle, MadSkin},
};

/// the mad skin applying to the status depending whether it's an
/// error or not
pub struct StatusMadSkinSet {
    pub normal: MadSkin,
    pub error: MadSkin,
}

/// build a MadSkin which will be used to display the status
/// when there's no error
fn make_normal_status_mad_skin(skin: &StyleMap) -> MadSkin {
    MadSkin {
        paragraph: LineStyle::new(
            skin.status_normal.clone(),
            Alignment::Left,
        ),
        italic: skin.status_italic.clone(),
        bold: skin.status_bold.clone(),
        inline_code: skin.status_code.clone(),
        ellipsis: skin.status_ellipsis.clone(),
        ..Default::default()
    }
}

/// build a MadSkin which will be used to display the status
/// when there's a error
fn make_error_status_mad_skin(skin: &StyleMap) -> MadSkin {
    MadSkin {
        paragraph: LineStyle::new(
            skin.status_error.clone(),
            Alignment::Left,
        ),
        ellipsis: skin.status_ellipsis.clone(),
        ..Default::default()
    }
}

impl StatusMadSkinSet {
    pub fn from_skin(skin: &StyleMap) -> Self {
        Self {
            normal: make_normal_status_mad_skin(skin),
            error: make_error_status_mad_skin(skin),
        }
    }
}

