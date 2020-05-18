use crate::{
    cli::AppLaunchArgs,
    conf::Conf,
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
}

impl AppContext {
    pub fn from(
        launch_args: AppLaunchArgs,
        verb_store: VerbStore,
        special_paths: Vec<SpecialPath>,
    ) -> Self {
        let config_path = Conf::default_location().to_string_lossy().to_string();
        Self {
            config_path,
            launch_args,
            verb_store,
            special_paths,
        }
    }
}
