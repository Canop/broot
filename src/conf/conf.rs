//! manage reading the verb shortcuts from the configuration file,
//! initializing if if it doesn't yet exist

use {
    super::*,
    crate::{
        display::ColsConf,
        errors::ProgramError,
        skin::SkinEntry,
        path::{Glob, SpecialHandling},
    },
    crossterm::style::Attribute,
    ahash::AHashMap,
    fnv::FnvHashMap,
    serde::Deserialize,
    std::{
        fs, io,
        path::{Path, PathBuf},
    },
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
    pub special_paths: AHashMap<Glob, SpecialHandling>,

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
    pub syntax_theme: Option<String>,

    #[serde(alias="true-colors")]
    pub true_colors: Option<bool>,

    #[serde(alias="icon-theme")]
    pub icon_theme: Option<String>,

    pub modal: Option<bool>,

    pub max_panels_count: Option<usize>,

    #[serde(alias="quit-on-last-cancel")]
    pub quit_on_last_cancel: Option<bool>,

    pub file_sum_threads_count: Option<usize>,

    #[serde(alias="max_staged_count")]
    pub max_staged_count: Option<usize>,
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
        let conf_filepath = Conf::default_location();
        if !conf_filepath.exists() {
            Conf::write_sample(&conf_filepath)?;
            println!(
                "New Configuration file written in {}{:?}{}.",
                Attribute::Bold,
                &conf_filepath,
                Attribute::Reset,
            );
            println!("You should have a look at it.");
        }
        let mut conf = Conf::default();
        match conf.read_file(conf_filepath) {
            Err(e) => Err(e),
            _ => Ok(conf),
        }
    }

    /// assume the file doesn't yet exist
    pub fn write_sample(filepath: &Path) -> Result<(), io::Error> {
        fs::create_dir_all(filepath.parent().unwrap())?;
        fs::write(filepath, DEFAULT_CONF_FILE)?;
        Ok(())
    }

    /// read the configuration from a given path. Assume it exists.
    /// Values set in the read file replace the ones of self.
    /// Errors are printed on stderr (assuming this function is called
    /// before terminal alternation).
    pub fn read_file(&mut self, path: PathBuf) -> Result<(), ProgramError> {
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
        overwrite!(self, quit_on_last_cancel, conf);
        overwrite!(self, file_sum_threads_count, conf);
        overwrite!(self, max_staged_count, conf);
        self.verbs.append(&mut conf.verbs);
        // the following maps are "additive": we can add entries from several
        // config files and they still make sense
        overwrite_map!(self, special_paths, conf);
        overwrite_map!(self, ext_colors, conf);
        self.files.push(path);
        Ok(())
    }
}

