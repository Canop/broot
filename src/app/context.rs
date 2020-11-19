use {
    super::*,
    crate::{
        cli::AppLaunchArgs,
        conf::Conf,
        display::{Cols, DEFAULT_COLS},
        icon::*,
        pattern::SearchModeMap,
        skin::ExtColorMap,
        tree::SpecialPath,
        verb::VerbStore,
    },
};

/// The immutable container that can be passed around
/// to provide the configuration things for the whole
/// life of the App
pub struct AppContext {

    /// where's the config file we're using
    pub config_path: String,

    /// all the arguments specified at launch
    pub launch_args: AppLaunchArgs,

    /// the verbs in use (builtins and configured ones)
    pub verb_store: VerbStore,

    /// the paths for which there's a special behavior to follow (comes from conf)
    pub special_paths: Vec<SpecialPath>,

    /// the map between search prefixes and the search mode to apply
    pub search_modes: SearchModeMap,

    /// order of columns in tree display
    pub cols: Cols,

    pub show_selection_mark: bool,

    /// mapping from file extension to colors (comes from conf)
    pub ext_colors: ExtColorMap,

    /// the syntect theme to use for text files previewing
    pub syntax_theme: Option<String>,

    /// precomputed status to display in standard cases
    /// (ie when no verb is involved)
    pub standard_status: StandardStatus,

    /// whether we can use 24 bits colors for previewed images
    pub true_colors: bool,

    /// map extensions to icons, icon set chosen based on config
    /// Send, Sync safely beause once created, everything is immutable
    pub icons: Option<Box<dyn IconPlugin + Send + Sync >>,
}

impl AppContext {
    pub fn from(
        launch_args: AppLaunchArgs,
        verb_store: VerbStore,
        config: &Conf,
    ) -> Self {
        let config_path = Conf::default_location().to_string_lossy().to_string();
        let standard_status = StandardStatus::new(&verb_store);
        let true_colors = if let Some(value) = config.true_colors {
            value
        } else {
            are_true_colors_available()
        };
        let icons = config.icon_theme.as_ref()
            .and_then(|itn| icon_plugin(itn));
        Self {
            config_path,
            launch_args,
            verb_store,
            special_paths: config.special_paths.clone(),
            search_modes: config.search_modes.clone(),
            cols: config.cols_order.unwrap_or(DEFAULT_COLS),
            show_selection_mark: config.show_selection_mark.unwrap_or(false),
            ext_colors: config.ext_colors.clone(),
            syntax_theme: config.syntax_theme.clone(),
            standard_status,
            true_colors,
            icons,
        }
    }
}

/// try to determine whether the terminal supports true
/// colors. This doesn't work well, hence the use of an
/// optional config setting.
/// Based on https://gist.github.com/XVilka/8346728#true-color-detection
fn are_true_colors_available() -> bool {
    if let Ok(colorterm) = std::env::var("COLORTERM") {
        debug!("COLORTERM env variable = {:?}", colorterm);
        if colorterm.contains("truecolor") || colorterm.contains("24bit") {
            debug!("true colors are available");
            true
        } else {
            false
        }
    } else {
        // this is debatable... I've found some terminals with COLORTERM
        // unset but supporting true colors. As it's easy to determine
        // that true colors aren't supported when looking at previewed
        // images I prefer this value
        true
    }
}
