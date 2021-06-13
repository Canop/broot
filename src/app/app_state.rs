use {
    crate::{
        stage::Stage,
    },
    std::path::PathBuf,
};


/// global mutable state
#[derive(Debug)]
pub struct AppState {
    pub stage: Stage,

    /// the current root, updated when a panel with this concept
    /// becomes active or changes its root
    pub root: PathBuf,

    /// the selected path in another panel than the currently
    /// active one, if any
    pub other_panel_path: Option<PathBuf>,
}

impl AppState {
}
