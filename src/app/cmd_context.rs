use {
    super::*,
    crate::{
        command::*,
        display::{Areas, Screen},
        skin::PanelSkin,
    },
    std::path::PathBuf,
};

/// short lived wrapping of a few things which are needed for the handling
/// of a command in a panel and won't be modified during the operation.
pub struct CmdContext<'c> {
    pub cmd: &'c Command,
    pub app: &'c AppCmdContext<'c>,
    pub panel: PanelCmdContext<'c>,
}

/// the part of the immutable command execution context which comes from the app
pub struct AppCmdContext<'c> {
    pub app_state: &'c AppState,
    pub other_path: Option<PathBuf>,
    pub panel_skin: &'c PanelSkin,
    pub preview: Option<PanelId>, // id of the app's preview panel
    pub screen: Screen,
    pub con: &'c AppContext,
}

/// the part of the command execution context which comes from the panel
pub struct PanelCmdContext<'c> {
    pub areas: &'c Areas,
    pub purpose: PanelPurpose,
}

impl<'c> CmdContext<'c> {
    pub fn has_preview(&self) -> bool {
        self.app.preview.is_some()
    }
    pub fn has_no_preview(&self) -> bool {
        self.app.preview.is_none()
    }
}
