//! manage reading the verb shortcuts from the configuration file,
//! initializing if if it doesn't yet exist

use {
    crate::{
        errors::ConfError,
        keys,
        skin,
        verb_conf::VerbConf,
    },
    crossterm::style::Attribute,
    directories::ProjectDirs,
    std::{
        collections::HashMap,
        fs, io,
        path::{Path, PathBuf},
        result::Result,
    },
    termimad::CompoundStyle,
    toml::{self, Value},
};

#[derive(Default)]
pub struct Conf {
    pub default_flags: String, // the flags to apply before cli ones
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

/// return the instance of ProjectDirs holding broot's specific paths
pub fn app_dirs() -> ProjectDirs {
    ProjectDirs::from("org", "dystroy", "broot")
        .expect("Unable to find configuration directories")
}

/// return the path to the config directory, based on XDG
pub fn dir() -> PathBuf {
    app_dirs().config_dir().to_path_buf()
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
            self.default_flags.push_str(&s);
        }
        // reading verbs
        if let Some(Value::Array(verbs_value)) = &root.get("verbs") {
            for verb_value in verbs_value.iter() {
                let invocation = string_field(verb_value, "invocation")
                    .unwrap_or("".to_string());
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
                self.verbs.push(VerbConf {
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
        if let Some(Value::Table(entries_tbl)) = &root.get("skin") {
            for (k, v) in entries_tbl.iter() {
                if let Some(s) = v.as_str() {
                    match skin::parse_object_style(s) {
                        Ok(ske) => {
                            self.skin.insert(k.to_string(), ske);
                        }
                        Err(e) => {
                            eprintln!("{}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

const DEFAULT_CONF_FILE: &str = r#"
###############################################################
# This configuration file lets you
# - define new commands
# - change the shortcut or triggering keys of built-in verbs
# - change the colors
# - set default values for flags
#
# Configuration documentation is available at
#     https://dystroy.org/broot
###############################################################

###############################################################
# Default flags
# You can set up flags you want broot to start with by
# default, for example `default_flags="ihp"` if you usually want
# to see hidden and gitignored files and the permissions (then
# if you don't want the hidden files you can launch `br -H`)
# A popular flag is the `g` one which displays git related info.
#
default_flags = ""

###############################################################
# Verbs and shortcuts
# You can define your own commands which would be applied to
# the selection.
#
# Exemple 1: launching `tail -n` on the selected file (leaving broot)
# [[verbs]]
# name = "tail_lines"
# invocation = "tl {lines_count}"
# execution = "tail -f -n {lines_count} {file}"
#
# Exemple 2: creating a new file without leaving broot
# [[verbs]]
# name = "touch"
# invocation = "touch {new_file}"
# execution = "touch {directory}/{new_file}"
# leave_broot = false

# If $EDITOR isn't set on your computer, you should either set it using
#  something similar to
#   export EDITOR=/usr/bin/nvim
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
key = "ctrl-c"
execution = ":quit"

[[verbs]]
invocation = "create {subpath}"
execution = "$EDITOR {directory}/{subpath}"

[[verbs]]
invocation = "git_diff"
shortcut = "gd"
execution = "git diff {file}"

# If $PAGER isn't set on your computer, you should either set it
#  or just replace it with your viewer of choice in the 'execution'
#  pattern.
# Example:
#  execution = "less {file}"
[[verbs]]
name = "view"
invocation = "view"
execution = "$PAGER {file}"

# A popular set of shorctuts for going up and down:
#
# [[verbs]]
# key = "ctrl-j"
# execution = ":line_down"
#
# [[verbs]]
# key = "ctrl-k"
# execution = ":line_up"
#
# [[verbs]]
# key = "ctrl-d"
# execution = ":page_down"
#
# [[verbs]]
# key = "ctrl-u"
# execution = ":page_up"

# If you develop using git, you might like to often switch
# to the "git status" filter:
# [[verbs]]
# key = "ctrl-g"
# execution = ":toggle_git_status"

###############################################################
# Skin
# If you want to change the colors of broot,
# uncomment the following bloc and start messing
# with the various values.
# Note that some of those colors might not correcly
# render on terminals with low capabilities.
#

# [skin]
# default = "gray(20) gray(1)"
# tree = "rgb(89, 73, 101) none"
# file = "gray(21) none"
# directory = "rgb(255, 152, 0) none bold"
# exe = "rgb(17, 164, 181) none"
# link = "Magenta none"
# pruning = "rgb(89, 73, 101) none Italic"
# perm__ = "gray(5) None"
# perm_r = "ansi(94) None"
# perm_w = "ansi(132) None"
# perm_x = "ansi(65) None"
# owner = "gray(12) none"
# group = "gray(12) none"
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

