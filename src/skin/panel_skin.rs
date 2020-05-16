use {
    super::*,
    termimad::MadSkin,
};

/// the various skin things used in a panel.
///
/// There are normally two instances of this struct in
/// a broot application: one is used for the focused panel
/// and one is used for the other panels.
pub struct PanelSkin {
    pub styles: StyleMap,
    pub status_skin: StatusMadSkinSet,
    pub help_skin: MadSkin,
}


impl PanelSkin {
    pub fn new(styles: StyleMap) -> Self {
        let status_skin = StatusMadSkinSet::from_skin(&styles);
        let help_skin = make_help_mad_skin(&styles);
        Self {
            styles,
            status_skin,
            help_skin,
        }
    }
}
