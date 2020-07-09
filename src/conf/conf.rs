//! manage reading the verb shortcuts from the configuration file,
//! initializing if if it doesn't yet exist

use {
    super::{
        default_conf::DEFAULT_CONF_FILE,
    },
    crate::{
        display::{Col, Cols},
        errors::ConfError,
        keys,
        pattern::{SearchModeMap, SearchModeMapEntry},
        selection_type::SelectionType,
        skin::{ExtColorMap, SkinEntry},
        tree::*,
        verb::VerbConf,
    },
    crossterm::style::Attribute,
    std::{
        collections::HashMap,
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
    pub verbs: Vec<VerbConf>,
    pub skin: HashMap<String, SkinEntry>,
    pub special_paths: Vec<SpecialPath>,
    pub search_modes: SearchModeMap,
    pub disable_mouse_capture: bool,
    pub cols_order: Option<Cols>,
    pub ext_colors: ExtColorMap,
}

fn string_field(value: &Value, field_name: &str) -> Option<String> {
    if let Value::Table(tbl) = value {
        if let Some(fv) = tbl.get(field_name) {
            if let Some(s) = fv.as_str() {
                return Some(s.to_string());
            }
        }
    }
    None
}

fn bool_field(value: &Value, field_name: &str) -> Option<bool> {
    if let Value::Table(tbl) = value {
        if let Some(Value::Boolean(b)) = tbl.get(field_name) {
            return Some(*b);
        }
    }
    None
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
        let root: Value = data.parse::<Value>()?;
        // reading default flags
        if let Some(s) = string_field(&root, "default_flags") {
            // it's additive because another config file may have
            // been read before and we usually want all the flags
            // (the last ones may reverse the first ones)
            self.default_flags.push_str(&s);
        }
        // date/time format
        self.date_time_format = string_field(&root, "date_time_format");
        // mouse capture
        if let Some(mouse_capture) = bool_field(&root, "capture_mouse") {
            self.disable_mouse_capture = !mouse_capture;
        }
        // cols order
        self.cols_order = string_field(&root, "cols_order")
            .map(|s| Col::parse_cols(&s))
            .transpose()?;
        // reading verbs
        if let Some(Value::Array(verbs_value)) = &root.get("verbs") {
            for verb_value in verbs_value.iter() {
                let invocation = string_field(verb_value, "invocation");
                let key = string_field(verb_value, "key")
                    .map(|s| keys::parse_key(&s))
                    .transpose()?;
                if let Some(key) = key {
                    if keys::is_reserved(key) {
                        return Err(ConfError::ReservedKey {
                            key: keys::key_event_desc(key),
                        });
                    }
                }
                let execution = match string_field(verb_value, "execution") {
                    Some(s) => s,
                    None => {
                        eprintln!("Invalid [[verbs]] entry in configuration");
                        eprintln!("Missing execution");
                        continue;
                    }
                };
                let from_shell = bool_field(verb_value, "from_shell");
                let leave_broot = bool_field(verb_value, "leave_broot");
                if leave_broot == Some(false) && from_shell == Some(true) {
                    eprintln!("Invalid [[verbs]] entry in configuration");
                    eprintln!(
                        "You can't simultaneously have leave_broot=false and from_shell=true"
                    );
                    continue;
                }
                let selection_condition = match string_field(verb_value, "apply_to").as_deref() {
                    Some("file") => SelectionType::File,
                    Some("directory") => SelectionType::Directory,
                    Some("any") => SelectionType::Any,
                    None => SelectionType::Any,
                    Some(s) => {
                        eprintln!("Invalid [[verbs]] entry in configuration");
                        eprintln!("{:?} isn't a valid value of apply_to", s);
                        continue;
                    }
                };
                let verb_conf = VerbConf {
                    invocation,
                    execution,
                    key,
                    shortcut: string_field(verb_value, "shortcut"),
                    description: string_field(verb_value, "description"),
                    from_shell,
                    leave_broot,
                    selection_condition,
                };

                self.verbs.push(verb_conf);
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

        Ok(())
    }
}

