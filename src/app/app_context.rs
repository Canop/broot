use {
    super::*,
    crate::{
        cli::AppLaunchArgs,
        conf::Conf,
        errors::ConfError,
        icon::*,
        pattern::SearchModeMap,
        path::SpecialPath,
        skin::ExtColorMap,
        verb::VerbStore,
    },
    std::{
        convert::{TryFrom, TryInto},
        path::PathBuf,
    },
};

/// The immutable container that can be passed around
/// to provide the configuration things for the whole
/// life of the App
pub struct AppContext {

    /// where's the config file we're using
    /// This vec can't be empty
    pub config_paths: Vec<PathBuf>,

    /// all the arguments specified at launch
    pub launch_args: AppLaunchArgs,

    /// the verbs in use (builtins and configured ones)
    pub verb_store: VerbStore,

    /// the paths for which there's a special behavior to follow (comes from conf)
    pub special_paths: Vec<SpecialPath>,

    /// the map between search prefixes and the search mode to apply
    pub search_modes: SearchModeMap,

    /// whether to show a triangle left to selected lines
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
    pub icons: Option<Box<dyn IconPlugin + Send + Sync>>,

    /// modal (aka "vim) mode enabled
    pub modal: bool,

    pub mouse_capture_disabled: bool,

    /// max number of panels (including preview) that can be
    /// open. Guaranteed to be at least 2.
    pub max_panels_count: usize,
}

impl AppContext {
    pub fn from(
        launch_args: AppLaunchArgs,
        verb_store: VerbStore,
        config: &Conf,
    ) -> Result<Self, ConfError> {
        let config_paths = config.files.clone();
        let standard_status = StandardStatus::new(&verb_store);
        let true_colors = if let Some(value) = config.true_colors {
            value
        } else {
            are_true_colors_available()
        };
        let icons = config.icon_theme.as_ref()
            .and_then(|itn| icon_plugin(itn));
        let special_paths = config.special_paths
            .iter()
            .map(|(k, v)| SpecialPath::new(k.clone(), *v))
            .collect();
        let search_modes = config
            .search_modes
            .as_ref()
            .map(|map| map.try_into())
            .transpose()?
            .unwrap_or_default();
        let ext_colors = ExtColorMap::try_from(&config.ext_colors)?;
        let max_panels_count = config.max_panels_count
            .unwrap_or(2)
            .clamp(2, 100);
        Ok(Self {
            config_paths,
            launch_args,
            verb_store,
            special_paths,
            search_modes,
            show_selection_mark: config.show_selection_mark.unwrap_or(false),
            ext_colors,
            syntax_theme: config.syntax_theme.clone(),
            standard_status,
            true_colors,
            icons,
            modal: config.modal.unwrap_or(false),
            mouse_capture_disabled: config.disable_mouse_capture.unwrap_or(false),
            max_panels_count,
        })
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
