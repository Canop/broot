use {
    super::*,
    crate::{
        cli::AppLaunchArgs,
        conf::Conf,
        display::{Cols, DEFAULT_COLS},
        pattern::SearchModeMap,
        tree::SpecialPath,
        verb::VerbStore,
    },
};

/// The immutable container that can be passed around
/// to provide the configuration things
pub struct AppContext {

    /// where's the config file we're using
    pub config_path: String,

    /// all the arguments specified at launch
    pub launch_args: AppLaunchArgs,

    /// the verbs in use (builtins and configured ones)
    pub verb_store: VerbStore,

    /// the paths for which there's a special behavior to follow (come from conf)
    pub special_paths: Vec<SpecialPath>,

    /// the map between search prefixes and the search mode to apply
    pub search_modes: SearchModeMap,

    /// order of columns in tree display
    pub cols: Cols,

    pub standard_status: StandardStatus,
}

impl AppContext {
    pub fn from(
        launch_args: AppLaunchArgs,
        verb_store: VerbStore,
        config: &Conf,
    ) -> Self {
        let config_path = Conf::default_location().to_string_lossy().to_string();
        let standard_status = StandardStatus::new(&verb_store);
        Self {
            config_path,
            launch_args,
            verb_store,
            special_paths: config.special_paths.clone(),
            search_modes: config.search_modes.clone(),
            cols: config.cols_order.unwrap_or(DEFAULT_COLS).clone(),
            standard_status,
        }
    }
}
