//! Definitions of custom errors used in broot
use custom_error::custom_error;
use opener;
use regex;
use std::io;

custom_error! {pub TreeBuildError
    NotADirectory { path: String } = "Not a directory: {}",
    FileNotFound { path: String } = "File not found: {}",
}

custom_error! {pub ProgramError
    Io {source: io::Error} = "IO Error : {:?}",
    Conf {source: ConfError} = "Bad configuration: {}",
    ArgParse {bad: String, valid: String} = "{:?} can't be parsed (valid values: {:?})",
    UnknownVerb {key: String} = "No verb matches {:?}",
    AmbiguousVerbKey {key: String} = "Ambiguous key: More than one verb matches {:?}",
    UnmatchingVerbArgs {key: String} = "No matching argument found for verb {:?}",
    TreeBuild {source: TreeBuildError} = "{}",
    OpenError {err: opener::OpenError} = "{}",
    LaunchError {program: String, source: io::Error} = "Unable to launch {program}: {source}",
}

custom_error! {pub RegexError
    Parsing {source: regex::Error} = @{
        format!("Invalid Regular Expression: {}", source.to_string().lines().last().unwrap_or(""))
    },
    UnknownFlag {bad: char} = "Unknown regular expression flag: {:?}",
}

custom_error! {pub InvalidSkinError
    InvalidColor { raw : String }  = "'{}' is not a valid color",
    InvalidAttribute { raw : String }  = "'{}' is not a valid style attribute",
    InvalidGreyLevel { level: u8 } = "grey level must be between 0 and 23 (got {})",
    InvalidStyle {style: String}   = "Invalid skin style : {}",
    //BadKey                         = "not a valid skin configuration key",
}

custom_error! {pub ConfError
    Io {source: io::Error}                          = "unable to read from the file",
    Toml {source: toml::de::Error}                  = "unable to parse TOML",
    MissingField {txt: String}                      = "missing field in conf",
    InvalidVerbInvocation {invocation: String}      = "invalid verb invocation: {}",
}
