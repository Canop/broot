//! Definitions of custom errors used in broot

use {
    custom_error::custom_error,
    regex,
    std::io,
};

custom_error! {pub ProgramError
    Io {source: io::Error} = "IO Error : {:?}",
    Crossterm {source: crossterm::ErrorKind} = "Crossterm Error : {:?}",
    Termimad {source: termimad::Error} = "Termimad Error : {:?}",
    Conf {source: ConfError} = "Bad configuration: {}",
    ArgParse {bad: String, valid: String} = "{:?} can't be parsed (valid values: {:?})",
    UnknownVerb {name: String} = "No verb matches {:?}",
    AmbiguousVerbName {name: String} = "Ambiguous name: More than one verb matches {:?}",
    UnmatchingVerbArgs {name: String} = "No matching argument found for verb {:?}",
    TreeBuild {source: TreeBuildError} = "{}",
    LaunchError {program: String, source: io::Error} = "Unable to launch {program}: {source}",
    UnknowShell {shell: String} = "Unknown shell: {shell}",
    InternalError {details: String} = "Internal error: {details}", // should not happen
}

custom_error! {pub TreeBuildError
    NotADirectory { path: String } = "Not a directory: {}",
    FileNotFound { path: String } = "File not found: {}",
}

custom_error! {pub ConfError
    Io {source: io::Error}                          = "unable to read from the file",
    Toml {source: toml::de::Error}                  = "unable to parse TOML",
    MissingField {txt: String}                      = "missing field in conf",
    InvalidVerbInvocation {invocation: String}      = "invalid verb invocation: {}",
    InvalidKey {raw: String}                        = "not a valid key: {}",
    ReservedKey {key: String}                       = "reserved key: {}",
}

// error which can be raised when parsing a regex the
// user typed
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
}

