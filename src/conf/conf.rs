//! manage reading the verb shortcuts from the configuration file,
//! initializing if if it doesn't yet exist

use {
    super::{
        default_conf::DEFAULT_CONF_FILE,
        toml::*,
    },
    crate::{
        verb::Verb,
        display::{Col, Cols},
        errors::ConfError,
        pattern::{SearchModeMap, SearchModeMapEntry},
        skin::{ExtColorMap, SkinEntry},
        tree::*,
    },
    crossterm::style::Attribute,
    std::{
        collections::HashMap,
        convert::TryFrom,
        fs, io,
        path::{Path, PathBuf},
    },
    toml::{self, Value},
};

/// The configuration read from conf.toml file(s)
#[derive(Default)]
pub struct Conf {
    pub default_flags: String, // the flags to apply before cli ones
    pub date_time_format: Option<String>,
    pub verbs: Vec<Verb>,
    pub skin: HashMap<String, SkinEntry>,
    pub special_paths: Vec<SpecialPath>,
    pub search_modes: SearchModeMap,
    pub disable_mouse_capture: bool,
    pub cols_order: Option<Cols>,
    pub show_selection_mark: Option<bool>,
    pub ext_colors: ExtColorMap,
    pub syntax_theme: Option<String>,
    pub true_colors: Option<bool>,
}

impl Conf {

    pub fn default_location() -> &'static Path {
        lazy_static! {
            static ref CONF_PATH: PathBuf = super::dir().join("conf.toml");
        }
        &*CONF_PATH
    }

    /// read the configuration file from the default OS specific location.
    /// Create it if it doesn't exist
    pub fn from_default_location() -> Result<Conf, ConfError> {
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
        match conf.read_file(&conf_filepath) {
            Err(e) => {
                println!("Failed to read configuration in {:?}.", &conf_filepath);
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
    /// stderr is supposed to be a valid solution for displaying errors
    /// (i.e. this function is called before or after the terminal alternation)
    pub fn read_file(&mut self, filepath: &Path) -> Result<(), ConfError> {
        let data = fs::read_to_string(filepath)?;
        let root: toml::value::Table = match data.parse::<Value>()? {
            Value::Table(tbl) => tbl,
            _ => {
                return Err(ConfError::Invalid {});
            }
        };
        // reading default flags
        if let Some(s) = string_field(&root, "default_flags") {
            // it's additive because another config file may have
            // been read before and we usually want all the flags
            // (the last ones may reverse the first ones)
            self.default_flags.push_str(&s);
        }
        // date/time format
        self.date_time_format = string_field(&root, "date_time_format");
        // reading the optional theme for syntect
        self.syntax_theme = string_field(&root, "syntax_theme");
        // mouse capture
        if let Some(mouse_capture) = bool_field(&root, "capture_mouse") {
            self.disable_mouse_capture = !mouse_capture;
        }
        // cols order
        if let Some(s) = string_field(&root, "cols_order") {
            // old format, with each char being a col, for example
            // `cols_order = "gbpdscn"`
            self.cols_order = Some(Col::parse_cols_single_str(&s)?);
        } else if let Some(arr) = string_array_field(&root, "cols_order") {
            // new format, where each col is a string, for example
            // `cols_order = ["branch", "size" ..., "name"]`
            self.cols_order = Some(Col::parse_cols(&arr)?);
        }
        // reading verbs
        if let Some(Value::Array(verbs_value)) = &root.get("verbs") {
            for verb_value in verbs_value.iter() {
                match Verb::try_from(verb_value) {
                    Ok(verb) => {
                        self.verbs.push(verb);
                    }
                    Err(e) => {
                        eprintln!("Invalid [[verbs]] entry in configuration: {:?}", e);
                    }
                }
            }
        }
        // reading the skin
        if let Some(Value::Table(entries_tbl)) = &root.get("skin") {
            for (k, v) in entries_tbl.iter() {
                if let Some(s) = v.as_str() {
                    match SkinEntry::parse(s) {
                        Ok(sec) => {
                            self.skin.insert(k.to_string(), sec);
                        }
                        Err(e) => {
                            eprintln!("{}", e);
                        }
                    }
                }
            }
        }
        // reading special paths
        if let Some(Value::Table(paths_tbl)) = &root.get("special-paths") {
            for (k, v) in paths_tbl.iter() {
                if let Some(v) = v.as_str() {
                    match SpecialPath::parse(k, v) {
                        Ok(sp) => {
                            debug!("Adding special path: {:?}", &sp);
                            self.special_paths.push(sp);
                        }
                        Err(e) => {
                            eprintln!("{}", e);
                        }
                    }
                }
            }
        }
        // reading serch modes
        if let Some(Value::Table(search_modes_tbl)) = &root.get("search-modes") {
            for (k, v) in search_modes_tbl.iter() {
                if let Some(v) = v.as_str() {
                    match SearchModeMapEntry::parse(k, v) {
                        Ok(entry) => {
                            debug!("Adding search mode map entry: {:?}", &entry);
                            self.search_modes.set(entry);
                        }
                        Err(e) => {
                            eprintln!("{}", e);
                        }
                    }
                }
            }
        }
        // reading the ext_colors map
        if let Some(Value::Table(ext_colors_tbl)) = &root.get("ext-colors") {
            for (k, v) in ext_colors_tbl.iter() {
                if let Some(v) = v.as_str() {
                    if let Err(e) = self.ext_colors.set(k.to_string(), v) {
                        eprintln!("{}", e);
                    }
                }
            }
        }
        // true_colors ?
        if let Some(b) = bool_field(&root, "true_colors") {
            self.true_colors = Some(b);
        }
        // show selection mark
        if let Some(b) = bool_field(&root, "show_selection_mark") {
            self.show_selection_mark = Some(b);
        }

        Ok(())
    }
}

