//! manage reading the verb shortcuts from the configuration file,
//! initializing if if it doesn't yet exist

use {
    super::*,
    crate::{
        app::Mode,
        display::ColsConf,
        errors::{ConfError, ProgramError},
        path::{
            path_from,
            PathAnchor,
        },
        skin::SkinEntry,
        syntactic::SyntaxTheme,
        verb::ExecPattern,
    },
    ahash::AHashMap,
    crokey::crossterm::style::Attribute,
    fnv::FnvHashMap,
    serde::Deserialize,
    std::collections::HashMap,
    std::path::PathBuf,
};

macro_rules! overwrite {
    ($dst: ident, $prop: ident, $src: ident) => {
        if $src.$prop.is_some() {
            $dst.$prop = $src.$prop.take();
        }
    };
}

macro_rules! overwrite_map {
    ($dst: ident, $prop: ident, $src: ident) => {
        for (k, v) in $src.$prop {
            $dst.$prop.insert(k, v);
        }
    };
}

/// The configuration read from conf.toml or conf.hjson file(s)
#[derive(Default, Clone, Debug, Deserialize)]
pub struct Conf {
    /// the files used to load this configuration
    #[serde(skip)]
    pub files: Vec<PathBuf>,

    #[serde(alias="default-flags")]
    pub default_flags: Option<String>, // the flags to apply before cli ones

    #[serde(alias="date-time-format")]
    pub date_time_format: Option<String>,

    #[serde(default)]
    pub verbs: Vec<VerbConf>,

    pub skin: Option<AHashMap<String, SkinEntry>>,

    #[serde(default, alias="special-paths")]
    pub special_paths: HashMap<GlobConf, SpecialHandlingConf>,

    #[serde(alias="search-modes")]
    pub search_modes: Option<FnvHashMap<String, String>>,

    /// Obsolete, kept for compatibility: you should now use capture_mouse
    #[serde(alias="disable-mouse-capture")]
    pub disable_mouse_capture: Option<bool>,

    #[serde(alias="capture-mouse")]
    pub capture_mouse: Option<bool>,

    #[serde(alias="cols-order")]
    pub cols_order: Option<ColsConf>,

    #[serde(alias="show-selection-mark")]
    pub show_selection_mark: Option<bool>,

    #[serde(default, alias="ext-colors")]
    pub ext_colors: AHashMap<String, String>,

    #[serde(alias="syntax-theme")]
    pub syntax_theme: Option<SyntaxTheme>,

    #[serde(alias="true-colors")]
    pub true_colors: Option<bool>,

    #[serde(alias="icon-theme")]
    pub icon_theme: Option<String>,

    pub modal: Option<bool>,

    /// the initial mode (only relevant when modal is true)
    #[serde(alias="initial-mode")]
    pub initial_mode: Option<Mode>,

    pub max_panels_count: Option<usize>,

    #[serde(alias="quit-on-last-cancel")]
    pub quit_on_last_cancel: Option<bool>,

    pub file_sum_threads_count: Option<usize>,

    #[serde(alias="max_staged_count")]
    pub max_staged_count: Option<usize>,

    #[serde(default)]
    pub imports: Vec<Import>,

    #[serde(alias="show-matching-characters-on-path-searches")]
    pub show_matching_characters_on_path_searches: Option<bool>,

    #[serde(alias="content-search-max-file-size", deserialize_with="file_size::deserialize", default)]
    pub content_search_max_file_size: Option<u64>,

    #[serde(alias="terminal-title")]
    pub terminal_title: Option<ExecPattern>,

    #[serde(alias="update-work-dir")]
    pub update_work_dir: Option<bool>,

    // BEWARE: entries added here won't be usable unless also
    // added in read_file!
}

impl Conf {

    /// return the path to the default conf.toml file.
    /// If there's no conf.hjson file in the default conf directory,
    /// and if there's a toml file, return this toml file.
    pub fn default_location() -> PathBuf {
        let hjson_file = super::dir().join("conf.hjson");
        if !hjson_file.exists() {
            let toml_file = super::dir().join("conf.toml");
            if toml_file.exists() {
                return toml_file;
            }
        }
        // neither file exists, we return the default one
        hjson_file
    }

    /// read the configuration file from the default OS specific location.
    /// Create it if it doesn't exist
    pub fn from_default_location() -> Result<Conf, ProgramError> {
        let conf_dir = super::dir();
        let conf_filepath = Conf::default_location();
        if !conf_filepath.exists() {
            write_default_conf_in(conf_dir)?;
            println!(
                "New Configuration files written in {}{:?}{}.",
                Attribute::Bold,
                &conf_dir,
                Attribute::Reset,
            );
            println!("You should have a look at them: their comments will help you configure broot.");
            println!("You should especially set up your favourite editor in verbs.hjson.");
        }
        let mut conf = Conf::default();
        conf.read_file(conf_filepath)?;
        Ok(conf)
    }

    pub fn solve_conf_path(&self, path: &str) -> Option<PathBuf> {
        if path.ends_with(".toml") || path.ends_with(".hjson") {
            for conf_file in self.files.iter().rev() {
                let solved = path_from(conf_file, PathAnchor::Parent, path);
                if solved.exists() {
                    return Some(solved)
                }
            }
        }
        None
    }

    /// read the configuration from a given path. Assume it exists.
    /// Values set in the read file replace the ones of self.
    /// Errors are printed on stderr (assuming this function is called
    /// before terminal alternation).
    pub fn read_file(
        &mut self,
        path: PathBuf,
    ) -> Result<(), ProgramError> {
        debug!("reading conf file: {:?}", &path);
        let mut conf: Conf = SerdeFormat::read_file(&path)?;
        overwrite!(self, default_flags, conf);
        overwrite!(self, date_time_format, conf);
        overwrite!(self, icon_theme, conf);
        overwrite!(self, syntax_theme, conf);
        overwrite!(self, disable_mouse_capture, conf);
        overwrite!(self, capture_mouse, conf);
        overwrite!(self, true_colors, conf);
        overwrite!(self, show_selection_mark, conf);
        overwrite!(self, cols_order, conf);
        overwrite!(self, skin, conf);
        overwrite!(self, search_modes, conf);
        overwrite!(self, max_panels_count, conf);
        overwrite!(self, modal, conf);
        overwrite!(self, initial_mode, conf);
        overwrite!(self, quit_on_last_cancel, conf);
        overwrite!(self, file_sum_threads_count, conf);
        overwrite!(self, max_staged_count, conf);
        overwrite!(self, show_matching_characters_on_path_searches, conf);
        overwrite!(self, content_search_max_file_size, conf);
        overwrite!(self, terminal_title, conf);
        overwrite!(self, update_work_dir, conf);
        self.verbs.append(&mut conf.verbs);
        // the following maps are "additive": we can add entries from several
        // config files and they still make sense
        overwrite_map!(self, special_paths, conf);
        overwrite_map!(self, ext_colors, conf);
        self.files.push(path);
        // read the imports
        for import in &conf.imports {
            let file = import.file().trim();
            if !import.applies() {
                debug!("skipping not applying conf file : {:?}", file);
                continue;
            }
            let import_path = self.solve_conf_path(file)
                .ok_or_else(|| ConfError::ImportNotFound { path: file.to_string() })?;
            if self.files.contains(&import_path) {
                debug!("skipping import already read: {:?}", import_path);
                continue;
            }
            self.read_file(import_path)?;
        }
        Ok(())
    }
}



