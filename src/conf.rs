//! manage reading the verb shortcuts from the configuration file,
//! initializing if if it doesn't yet exist

use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
    result::Result,
};

use crossterm::style::Attribute;
use termimad::CompoundStyle;
use directories::ProjectDirs;
use toml::{self, Value};

use crate::{
    errors::ConfError,
    skin_conf,
    verb_conf::{self, VerbConf},
};

pub struct Conf {
    pub verbs: Vec<VerbConf>,
    pub skin: HashMap<String, CompoundStyle>,
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
        match Conf::from_file(&conf_filepath) {
            Ok(conf) => Ok(conf),
            Err(e) => {
                println!("Failed to read configuration in {:?}.", &conf_filepath);
                println!("Please delete or fix this file.");
                Err(e)
            }
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
                let key = string_field(verb_value, "key")
                    .map(|s| verb_conf::parse_key(&s))
                    .transpose()?;
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
                verbs.push(VerbConf {
                    invocation,
                    execution,
                    key,
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
                        Ok(ske) => {
                            skin.insert(k.to_string(), ske);
                        }
                        Err(e) => {
                            eprintln!("{}", e);
                        }
                    }
                }
            }
        }

        Ok(Conf { verbs, skin })
    }
}

const DEFAULT_CONF_FILE: &str = r#"
# This configuration file lets you define new commands
# or change the shortcut or triggering keys of built-in verbs.
# You can change the colors of broot too.
#
# Configuration documentation is available at https://dystroy.org/broot
#

#####################
# user defined verbs:

# If $EDITOR isn't set on your computer, you should either set it
#  or just replace it with your editor of choice in the 'execution'
#  pattern.
# Example:
#  execution = "/usr/bin/nvim {file}"
[[verbs]]
invocation = "edit"
key = "F2"
shortcut = "e"
execution = "$EDITOR {file}"

[[verbs]]
invocation = "create {subpath}"
execution = "$EDITOR {directory}/{subpath}"

# If $PAGER isn't set on your computer, you should either set it
#  or just replace it with your viewer of choice in the 'execution'
#  pattern.
# Example:
#  execution = "less {file}"
[[verbs]]
name = "view"
invocation = "view"
execution = "$PAGER {file}"

#####################
# Skin

# If you want to change the colors of broot,
# uncomment the following bloc and start messing
# with the various values
# Note that some of those colors might not correcly
# render on terminals with low capabilities
#
# [skin]
# default = "gray(20) gray(1)"
# tree = "rgb(89, 73, 101) none"
# file = "gray(21) none"
# directory = "rgb(255, 152, 0) none bold"
# exe = "rgb(17, 164, 181) none"
# link = "Magenta none"
# pruning = "rgb(89, 73, 101) none Italic"
# permissions = "gray(12) none "
# selected_line = "none gray(3)"
# char_match = "yellow none"
# file_error = "Red none"
# flag_label = "gray(16) none"
# flag_value = "rgb(255, 152, 0) none bold"
# input = "White none"
# status_error = "Red gray(2)"
# status_job = "ansi(220) gray(5)"
# status_normal = "gray(20) gray(3)"
# status_italic = "rgb(255, 152, 0) None"
# status_bold = "rgb(255, 152, 0) None bold"
# status_code = "ansi(229) gray(5)"
# status_ellipsis = "gray(19) gray(1)"
# scrollbar_track = "rgb(80, 50, 0) none"
# scrollbar_thumb = "rgb(255, 187, 0) none"
# help_paragraph = "gray(20) none"
# help_bold = "rgb(255, 187, 0) none bold"
# help_italic = "Magenta rgb(30, 30, 40) italic"
# help_code = "gray(21) gray(3)"
# help_headers = "rgb(255, 187, 0) none"

# You may find other skins on
#  https://dystroy.org/broot/documentation/configuration/#colors
# for example a skin suitable for white backgrounds

"#;
