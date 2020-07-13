use {
    super::*,
    crate::{
        command::*,
        display::Areas,
        skin::PanelSkin,
    },
    std::path::PathBuf,
};

/// short lived wrapping of a few things which are needed for the handling
/// of a command in a panel and won't be modified during the operation.
pub struct CmdContext<'c> {
    pub cmd: &'c Command,
    pub other_path: &'c Option<PathBuf>,
    pub panel_skin: &'c PanelSkin,
    pub con: &'c AppContext,
    pub areas: &'c Areas,
    pub preview: Option<PanelId>, // id of the app's preview panel
    pub panel_purpose: PanelPurpose,
}
