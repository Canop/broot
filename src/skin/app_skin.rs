use {
    super::*,
    crate::{
        conf::Conf,
    },
};


/// all the skin things used by the broot application
/// during runing
pub struct AppSkin {

    /// the skin used in the focused panel
    pub focused: PanelSkin,

    /// the skin used in unfocused panels
    pub unfocused: PanelSkin,

}

impl AppSkin {

    pub fn new(conf: &Conf) -> Self {
        let StyleMaps { focused, unfocused } = StyleMaps::create(&conf.skin);
        let focused = PanelSkin::new(focused);
        let unfocused = PanelSkin::new(unfocused);
        Self {
            focused,
            unfocused,
        }
    }

}
