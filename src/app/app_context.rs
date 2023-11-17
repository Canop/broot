use {
    super::*,
    crate::{
        cli::{Args, TriBool},
        conf::*,
        content_search,
        errors::*,
        file_sum,
        icon::*,
        path,
        pattern::SearchModeMap,
        skin::ExtColorMap,
        syntactic::SyntaxTheme,
        tree::TreeOptions,
        verb::*,
    },
    std::{
        convert::{TryFrom, TryInto},
        io,
        path::{Path, PathBuf},
    },
};

/// The container that can be passed around to provide the configuration things
/// for the whole life of the App
pub struct AppContext {

    /// The initial tree root
    pub initial_root: PathBuf,

    /// The initial file to select and preview
    pub initial_file: Option<PathBuf>,

    /// Initial tree options
    pub initial_tree_options: TreeOptions,

    /// where's the config file we're using
    /// This vec can't be empty
    pub config_paths: Vec<PathBuf>,

    /// all the arguments specified at launch
    pub launch_args: Args,

    /// the "launch arguments" found in the default_flags
    /// of the config file(s)
    pub config_default_args: Option<Args>,

    /// the verbs in use (builtins and configured ones)
    pub verb_store: VerbStore,

    /// the paths for which there's a special behavior to follow (comes from conf)
    pub special_paths: Vec<path::SpecialPath>,

    /// the map between search prefixes and the search mode to apply
    pub search_modes: SearchModeMap,

    /// whether to show a triangle left to selected lines
    pub show_selection_mark: bool,

    /// mapping from file extension to colors (comes from conf)
    pub ext_colors: ExtColorMap,

    /// the syntect theme to use for text files previewing
    pub syntax_theme: Option<SyntaxTheme>,

    /// precomputed status to display in standard cases
    /// (ie when no verb is involved)
    pub standard_status: StandardStatus,

    /// whether we can use 24 bits colors for previewed images
    pub true_colors: bool,

    /// map extensions to icons, icon set chosen based on config
    /// Send, Sync safely because once created, everything is immutable
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

    /// max file size when searching file content
    pub content_search_max_file_size: usize,

    /// the optional pattern used to change the terminal's title
    /// (if none, the title isn't modified)
    pub terminal_title_pattern: Option<ExecPattern>,
}

impl AppContext {
    pub fn from(
        launch_args: Args,
        verb_store: VerbStore,
        config: &Conf,
    ) -> Result<Self, ProgramError> {
        let config_default_args = config
            .default_flags
            .as_ref()
            .map(|flags| parse_default_flags(flags))
            .transpose()?;
        let config_paths = config.files.clone();
        let standard_status = StandardStatus::new(&verb_store);
        let true_colors = if let Some(value) = config.true_colors {
            value
        } else {
            are_true_colors_available()
        };
        let icons = config.icon_theme.as_ref()
            .and_then(|itn| icon_plugin(itn));
        let mut special_paths = config.special_paths
            .iter()
            .map(|(k, v)| path::SpecialPath::new(k.clone(), *v))
            .collect();
        path::add_defaults(&mut special_paths);
        let search_modes = config
            .search_modes
            .as_ref()
            .map(|map| map.try_into())
            .transpose()?
            .unwrap_or_default();
        let ext_colors = ExtColorMap::try_from(&config.ext_colors)
            .map_err(ConfError::from)?;
        let file_sum_threads_count = config.file_sum_threads_count
            .unwrap_or(file_sum::DEFAULT_THREAD_COUNT);
        if !(1..=50).contains(&file_sum_threads_count) {
            return Err(ConfError::InvalidThreadsCount{ count: file_sum_threads_count }.into());
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
        let (initial_root, initial_file) = initial_root_file(&launch_args)?;

        // tree options are built from the default_flags
        // found in the config file(s) (if any) then overridden
        // by the cli args (order is important)
        let mut initial_tree_options = TreeOptions::default();
        initial_tree_options.apply_config(config)?;
        if let Some(args) = &config_default_args {
            initial_tree_options.apply_launch_args(args);
        }
        initial_tree_options.apply_launch_args(&launch_args);
        if launch_args.color == TriBool::No {
            initial_tree_options.show_selection_mark = true;
        }

        let content_search_max_file_size = config.content_search_max_file_size
            .map(|u64value| usize::try_from(u64value).unwrap_or(usize::MAX))
            .unwrap_or(content_search::DEFAULT_MAX_FILE_SIZE);

        let terminal_title_pattern = config.terminal_title.clone();

        Ok(Self {
            initial_root,
            initial_file,
            initial_tree_options,
            config_paths,
            launch_args,
            config_default_args,
            verb_store,
            special_paths,
            search_modes,
            show_selection_mark: config.show_selection_mark.unwrap_or(false),
            ext_colors,
            syntax_theme: config.syntax_theme,
            standard_status,
            true_colors,
            icons,
            modal: config.modal.unwrap_or(false),
            capture_mouse,
            max_panels_count,
            quit_on_last_cancel: config.quit_on_last_cancel.unwrap_or(false),
            file_sum_threads_count,
            max_staged_count,
            content_search_max_file_size,
            terminal_title_pattern,
        })
    }
    /// Return the --cmd argument, coming from the launch arguments (prefered)
    /// or from the default_flags parameter of a config file
    pub fn cmd(&self) -> Option<&str> {
        self.launch_args.cmd.as_ref().or(
            self.config_default_args.as_ref().and_then(|args| args.cmd.as_ref())
        ).map(|s| s.as_str())
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

/// Determine the initial root folder to show, and the optional
/// initial file to open in preview
fn initial_root_file(cli_args: &Args) -> Result<(PathBuf, Option<PathBuf>), ProgramError> {
    let mut file = None;
    let mut root = match cli_args.root.as_ref() {
        Some(path) => canonicalize_root(path)?,
        None => std::env::current_dir()?,
    };
    if !root.exists() {
        return Err(TreeBuildError::FileNotFound {
            path: format!("{:?}", &root),
        }.into());
    }
    if !root.is_dir() {
        // we try to open the parent directory if the passed file isn't one
        if let Some(parent) = root.parent() {
            file = Some(root.clone());
            info!("Passed path isn't a directory => opening parent instead");
            root = parent.to_path_buf();
        } else {
            // this is a weird filesystem, let's give up
            return Err(TreeBuildError::NotADirectory {
                path: format!("{:?}", &root),
            }.into());
        }
    }
    Ok((root, file))
}

#[cfg(not(windows))]
fn canonicalize_root(root: &Path) -> io::Result<PathBuf> {
    root.canonicalize()
}

#[cfg(windows)]
fn canonicalize_root(root: &Path) -> io::Result<PathBuf> {
    Ok(if root.is_relative() {
        std::env::current_dir()?.join(root)
    } else {
        root.to_path_buf()
    })
}


