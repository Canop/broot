//! Definitions of custom errors used in broot
use custom_error::custom_error;
use std::io;

use crate::conf;

custom_error! {pub TreeBuildError
    NotADirectory { path: String } = "Not a directory: {}",
    FileNotFound { path: String } = "File not found: {}",
}

custom_error! {pub ProgramError
    Io {source: io::Error} = "IO Error : {:?}",
    Conf {source: ConfError} = "Bad configuration",
    ArgParse {bad: String, valid: String} = "{:?} can't be parsed (valid values: {:?})",
    TreeBuild {source: TreeBuildError} = "{}",
}

custom_error! {pub RegexError
    Parsing {source: regex::Error} = "Invalid Regular Expression",
    UnknownFlag {bad: char} = "Unknown regular expression flag: {:?}",
}

custom_error! {pub ConfError
    Io{source: io::Error}               = "unable to read from the file",
    Toml{source: toml::de::Error}       = "unable to parse TOML",
    MissingField{txt: String}           = "missing field in conf",
    InvalidSkinEntry{reason: String}    = "{}",
}
