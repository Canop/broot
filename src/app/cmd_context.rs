use {
    super::*,
    crate::{
        command::*,
        display::{Areas, Screen},
        skin::PanelSkin,
    },
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
    pub panel_skin: &'c PanelSkin,
    pub preview_panel: Option<PanelId>, // id of the app's preview panel
    pub stage_panel: Option<PanelId>, // id of the app's preview panel
    pub screen: Screen,
    pub con: &'c AppContext,
}

/// the part of the command execution context which comes from the panel
pub struct PanelCmdContext<'c> {
    pub areas: &'c Areas,
    pub purpose: PanelPurpose,
}
