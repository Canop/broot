use crate::{
    cli::AppLaunchArgs,
    conf::Conf,
    pattern::SearchModeMap,
    tree::SpecialPath,
    verb::VerbStore,
};

/// The immutable container that can be passed around
/// to provide the configuration things
pub struct AppContext {
    pub config_path: String,
    pub launch_args: AppLaunchArgs,
    pub verb_store: VerbStore,
    pub special_paths: Vec<SpecialPath>,
    pub search_modes: SearchModeMap,
}

impl AppContext {
    pub fn from(
        launch_args: AppLaunchArgs,
        verb_store: VerbStore,
        special_paths: Vec<SpecialPath>,
        search_modes: SearchModeMap,
    ) -> Self {
        let config_path = Conf::default_location().to_string_lossy().to_string();
        Self {
            config_path,
            launch_args,
            verb_store,
            special_paths,
            search_modes,
        }
    }
}
