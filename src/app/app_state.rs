use {
    crate::stage::Stage,
    std::path::PathBuf,
};

/// global mutable state
#[derive(Debug)]
pub struct AppState {
    pub stage: Stage,

    /// the current root, updated when a panel with this concept
    /// becomes active or changes its root
    pub root: PathBuf,

    /// Whether to refresh the tree view when in case of inotify event
    /// on the current root
    pub watch_root: bool,

    /// the selected path in another panel than the currently
    /// active one, if any
    pub other_panel_path: Option<PathBuf>,
}

impl AppState {
    pub fn new<P: Into<PathBuf>>(root: P) -> Self {
        Self {
            stage: Stage::default(),
            root: root.into(),
            watch_root: false,
            other_panel_path: None,
        }
    }
}
