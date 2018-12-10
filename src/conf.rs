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

// what's needed to handle a verb
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

fn string_field(value: &Value, field_name: &str) -> Result<String, ConfError> {
    match &value[field_name] {
        Value::String(s) => Ok(s.to_owned()),
        _ => Err(ConfError::MissingField {
            txt: field_name.to_owned(),
        }),
    }
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
    // read the configuration from a given path. Assume it exists
    pub fn from_file(filepath: &Path) -> Result<Conf, ConfError> {
        let data = fs::read_to_string(filepath)?;
        let root: Value = data.parse::<Value>()?;
        let mut verbs: Vec<VerbConf> = vec![];
        if let Value::Array(verbs_value) = &root["verbs"] {
            for verb_value in verbs_value.iter() {
                let name = string_field(verb_value, "name")?;
                let invocation = string_field(verb_value, "invocation")?;
                let execution = string_field(verb_value, "execution")?;
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
# verbs define the commands you can call on files
# 'invocation' can be a letter or a word
# 'execution' is either a command, where {file} will be replaced by the selected line,
# 	or one of the predefined commands:
#   ":back"          : reverts to the previous state, or quit the application if it's the first one (mapped to <esc>)
#   ":print_path"    : outputs the path to stdout
#   ":focus"         : displays the tree of that directory (mapped to <enter> on directories)
#   ":open"          : tries to open the file according to OS settings (e.g. using xdg-open) (mapped to <enter> on files)
#   ":parent"        : moves to the parent directory
#   ":quit"          : quits the application
#   ":toggle_hidden" : toggles showing hidden files

[[verbs]]
name = "cd"
invocation = "c"
execution = ":cd" # doesn't work yet

[[verbs]]
name = "focus"
invocation = "f"
execution = ":focus"

[[verbs]]
name = "edit"
invocation = "e"
execution = "/usr/bin/nvim {file}"

[[verbs]]
name = "toggle hidden"
invocation = "h"
execution = ":toggle_hidden"

[[verbs]]
name = "toggle files"
invocation = "f"
execution = ":toggle_files"

[[verbs]]
name = "open"
invocation = "o"
execution = ":open"

[[verbs]]
# this is an example of a very specific verb
name = "geany"
invocation = "g"
execution = "/usr/bin/geany {file}"

[[verbs]]
name = "parent"
invocation = "p"
execution = ":parent"

[[verbs]]
name = "quit"
invocation = "q"
execution = ":quit"
"#;
