use {
    super::*,
    crate::{
        display::Screen,
        skin::PanelSkin,
    },
    termimad::Area,
};

/// short lived wrapping of a few things which are needed for displaying
/// panels
pub struct DisplayContext<'c> {
    pub count: usize,
    pub active: bool,
    pub screen: Screen,
    pub state_area: Area,
    pub panel_skin: &'c PanelSkin,
    pub app_state: &'c AppState,
    pub con: &'c AppContext,
}

