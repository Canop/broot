//! manage reading the verb shortcuts from the configuration file,
//! initializing if if it doesn't yet exist

use {
    super::*,
    crate::{
        display::ColsConf,
        errors::ProgramError,
        skin::SkinEntry,
        tree::*,
    },
    crossterm::style::Attribute,
    fnv::FnvHashMap,
    serde::Deserialize,
    std::{
        fs, io,
        path::{Path, PathBuf},
    },
    toml,
};

macro_rules! overwrite {
    ($dst: ident, $prop: ident, $src: ident) => {
        if $src.$prop.is_some() {
            $dst.$prop = $src.$prop.take();
        }
    }
}

macro_rules! overwrite_map {
    ($dst: ident, $prop: ident, $src: ident) => {
        for (k, v) in $src.$prop {
            $dst.$prop.insert(k, v);
        }
    }
}

/// The configuration read from conf.toml file(s)
#[derive(Default, Clone, Deserialize)]
pub struct Conf {
    /// the files used to load this configuration
    #[serde(skip)]
    pub files: Vec<PathBuf>,
    pub default_flags: Option<String>, // the flags to apply before cli ones
    pub date_time_format: Option<String>,
    pub verbs: Vec<VerbConf>,
    pub skin: Option<FnvHashMap<String, SkinEntry>>,
    #[serde(default)]
    pub special_paths: FnvHashMap<Glob, SpecialHandling>,
    pub search_modes: Option<FnvHashMap<String, String>>,
    pub disable_mouse_capture: Option<bool>,
    pub cols_order: Option<ColsConf>,
    pub show_selection_mark: Option<bool>,
    #[serde(default)]
    pub ext_colors: FnvHashMap<String, String>,
    pub syntax_theme: Option<String>,
    pub true_colors: Option<bool>,
    pub icon_theme: Option<String>,
}

impl Conf {

    /// return the path to the default conf.toml file.
    /// If there's no conf.hjson file in the default conf directory,
    /// and if there's a toml file, return this toml file.
    pub fn default_location() -> PathBuf {
        super::dir() .join("conf.toml")
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
            Err(e) => {
                println!("{:?}", e);
                println!("Please delete or fix this file.");
                Err(e)
            }
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
        let file_content = fs::read_to_string(&path)?;
        let mut conf = toml::from_str::<Conf>(&file_content)
            .map_err(|e| ProgramError::ConfFile {
                path: path.to_string_lossy().to_string(),
                details: e.into(),
            })?;
        overwrite!(self, default_flags, conf);
        overwrite!(self, date_time_format, conf);
        overwrite!(self, icon_theme, conf);
        overwrite!(self, syntax_theme, conf);
        overwrite!(self, disable_mouse_capture, conf);
        overwrite!(self, true_colors, conf);
        overwrite!(self, show_selection_mark, conf);
        overwrite!(self, cols_order, conf);
        overwrite!(self, skin, conf);
        overwrite!(self, search_modes, conf);
        self.verbs.append(&mut conf.verbs);
        // the following maps are "additive": we can add entries from several
        // config files and they still make sense
        overwrite_map!(self, special_paths, conf);
        overwrite_map!(self, ext_colors, conf);
        self.files.push(path);
        Ok(())
    }
}

