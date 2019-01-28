//! manage reading the verb shortcuts from the configuration file,
//! initializing if if it doesn't yet exist

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::result::Result;
use toml::{self, Value};

use custom_error::custom_error;
use directories::ProjectDirs;

custom_error! {pub ConfError
    Io{source: io::Error}           = "unable to read from the file",
    Toml{source: toml::de::Error}   = "unable to parse TOML",
    MissingField{txt: String}       = "missing field in conf",
}

/// what's needed to handle a verb
#[derive(Debug)]
pub struct VerbConf {
    pub name: String,
    pub invocation: String,
    pub execution: String,
}

#[derive(Debug)]
pub struct Conf {
    pub verbs: Vec<VerbConf>,
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

impl Conf {
    pub fn default_location() -> PathBuf {
        let dirs = match ProjectDirs::from("org", "dystroy", "broot") {
            Some(dirs) => dirs,
            None => {
                panic!("Unable to find configuration directories");
            }
        };
        dirs.config_dir().join("conf.toml")
    }
    // read the configuration file from the default OS specific location.
    // Create it if it doesn't exist
    pub fn from_default_location() -> Result<Conf, ConfError> {
        let conf_filepath = Conf::default_location();
        if !conf_filepath.exists() {
            Conf::write_sample(&conf_filepath)?;
            println!(
                "{}New Configuration file written in {:?}.{}",
                termion::style::Bold,
                &conf_filepath,
                termion::style::Reset
            );
            println!("You should have a look at it.");
        }
        Ok(Conf::from_file(&conf_filepath)?)
    }
    // assume the file doesn't yet exist
    pub fn write_sample(filepath: &Path) -> Result<(), io::Error> {
        fs::create_dir_all(filepath.parent().unwrap())?;
        fs::write(filepath, DEFAULT_CONF_FILE)?;
        Ok(())
    }
    // read the configuration from a given path. Assume it exists.
    // stderr is supposed to be a valid solution for displaying errors
    // (i.e. this function is called before or after the terminal alternation)
    pub fn from_file(filepath: &Path) -> Result<Conf, ConfError> {
        let data = fs::read_to_string(filepath)?;
        let root: Value = data.parse::<Value>()?;
        let mut verbs: Vec<VerbConf> = vec![];
        if let Value::Array(verbs_value) = &root["verbs"] {
            for verb_value in verbs_value.iter() {
                let invocation = match string_field(verb_value, "invocation") {
                    Some(s) => s,
                    None => {
                        eprintln!("Missing invocation in [[verbs]] entry in configuration");
                        continue;
                    }
                };
                let execution = match string_field(verb_value, "execution") {
                    Some(s) => s,
                    None => {
                        eprintln!("Missing execution in [[verbs]] entry in configuration");
                        continue;
                    }
                };
                let name = match string_field(verb_value, "name") {
                    Some(s) => s,
                    None => {
                        if execution.starts_with(":") {
                            // we'll assume that this entry isn't a new verb definition
                            // but just the addition of a shortcut for a built-in verb
                            "unnamed".to_string()
                        } else {
                            eprintln!("Missing name in [[verbs]] entry in configuration");
                            continue;
                        }
                    }
                };
                verbs.push(VerbConf {
                    name,
                    invocation,
                    execution,
                });
            }
        }
        Ok(Conf { verbs })
    }
}

const DEFAULT_CONF_FILE: &str = r#"
# This configuration file lets you define new commands
# or change the shortcut of built-in verbs.
#
# 'invocation' can be a letter or a word
# 'execution' is either a command, where {file} will be replaced by the selected line,
# 	or one of the built-in commands.
#
# The configuration documentation and complete list of built-in verbs
# can be found in https://github.com/Canop/broot/documentation.md


###############################
# shortcuts for built-in verbs:

[[verbs]]
invocation = "p"
execution = ":parent"

#####################
# user defined verbs:

[[verbs]]
name = "edit"
invocation = "ed"
execution = "/usr/bin/nvim {file}"

[[verbs]]
name = "view"
invocation = "view"
execution = "less {file}"

"#;
