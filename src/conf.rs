//! manage reading the verb shortcuts from the configuration file,
//! initializing if if it doesn't yet exist

use crossterm::{Attribute};
use directories::ProjectDirs;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::result::Result;
use toml::{self, Value};
use crossterm::ObjectStyle;

use crate::errors::ConfError;
use crate::skin_conf;

/// what's needed to handle a verb
#[derive(Debug)]
pub struct VerbConf {
    pub shortcut: Option<String>,
    pub invocation: String,
    pub execution: String,
    pub description: Option<String>,
    pub from_shell: Option<bool>,
    pub leave_broot: Option<bool>,
    pub confirm: Option<bool>,
}

pub struct Conf {
    pub verbs: Vec<VerbConf>,
    pub skin: HashMap<String, ObjectStyle>,
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

/// return the path to the config directory, based on XDG
pub fn dir() -> PathBuf {
    if let Some(dirs) = ProjectDirs::from("org", "dystroy", "broot") {
        dirs.config_dir().to_path_buf()
    } else {
        panic!("Unable to find configuration directories");
    }
}

impl Conf {

    pub fn default_location() -> PathBuf {
        dir().join("conf.toml")
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
        Ok(Conf::from_file(&conf_filepath)?)
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
    pub fn from_file(filepath: &Path) -> Result<Conf, ConfError> {
        let data = fs::read_to_string(filepath)?;
        let root: Value = data.parse::<Value>()?;
        // reading verbs
        let mut verbs: Vec<VerbConf> = vec![];
        if let Some(Value::Array(verbs_value)) = &root.get("verbs") {
            for verb_value in verbs_value.iter() {
                let invocation = match string_field(verb_value, "invocation") {
                    Some(s) => s,
                    None => {
                        eprintln!("Invalid [[verbs]] entry in configuration");
                        eprintln!("Missing invocation");
                        continue;
                    }
                };
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
                        eprintln!("You can't simultaneously have leave_broot=false and from_shell=true");
                        continue;
                }
                verbs.push(VerbConf {
                    invocation,
                    execution,
                    shortcut: string_field(verb_value, "shortcut"),
                    description: string_field(verb_value, "description"),
                    from_shell,
                    leave_broot,
                    confirm: bool_field(verb_value, "confirm"),
                });
            }
        }
        // reading the skin
        let mut skin = HashMap::new();
        if let Some(Value::Table(entries_tbl)) = &root.get("skin") {
            for (k, v) in entries_tbl.iter() {
                if let Some(s) = v.as_str() {
                    match skin_conf::parse_object_style(s) {
                        Ok(ske) => { skin.insert(k.to_string(), ske); },
                        Err(e) => { eprintln!("{}", e); }
                    }
                }
            }
        }

        Ok(Conf {
            verbs,
            skin,
        })
    }
}

const DEFAULT_CONF_FILE: &str = r#"
# This configuration file lets you define new commands
# or change the shortcut of built-in verbs.
# You can change the colors of broot too.
#
# Configuration documentation is available at https://dystroy.org/broot
#

###############################
# shortcuts for built-in verbs:

[[verbs]]
invocation = "p"
execution = ":parent"

#####################
# user defined verbs:

# If $EDITOR isn't set on your computer, you should either set it
#  or just replace it with your editor of choice in the 'execution'
#  pattern.
# Example:
#  execution = "/usr/bin/nvim {file}"
[[verbs]]
invocation = "edit"
shortcut = "e"
execution = "$EDITOR {file}"

[[verbs]]
invocation = "create {subpath}"
execution = "$EDITOR {directory}/{subpath}"

[[verbs]]
name = "view"
invocation = "view"
execution = "less {file}"

"#;
