use {
    super::*,
    crate::{
        cli::AppLaunchArgs,
        conf::Conf,
        errors::ConfError,
        file_sum,
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

    /// Whether to support mouse interactions
    pub capture_mouse: bool,

    /// max number of panels (including preview) that can be
    /// open. Guaranteed to be at least 2.
    pub max_panels_count: usize,

    /// whether to quit broot when the user hits "escape"
    /// and there's nothing to cancel
    pub quit_on_last_cancel: bool,

    /// number of threads used by file_sum (count, size, date)
    /// computation
    pub file_sum_threads_count: usize,

    /// number of files which may be staged in one staging operation
    pub max_staged_count: usize,
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
        let file_sum_threads_count = config.file_sum_threads_count
            .unwrap_or(file_sum::DEFAULT_THREAD_COUNT);
        if file_sum_threads_count < 1 || file_sum_threads_count > 50 {
            return Err(ConfError::InvalidThreadsCount{ count: file_sum_threads_count });
        }
        let max_panels_count = config.max_panels_count
            .unwrap_or(2)
            .clamp(2, 100);
        let capture_mouse = match (config.capture_mouse, config.disable_mouse_capture) {
            (Some(b), _) => b, // the new "capture_mouse" argument takes precedence
            (_, Some(b)) => !b,
            _ => true,
        };
        let max_staged_count = config.max_staged_count
            .unwrap_or(10_000)
            .clamp(10, 100_000);
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
            capture_mouse,
            max_panels_count,
            quit_on_last_cancel: config.quit_on_last_cancel.unwrap_or(false),
            file_sum_threads_count,
            max_staged_count,
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
