//! Definitions of custom errors used in broot

use {
    custom_error::custom_error,
    image::error::ImageError,
    regex,
    std::io,
};

custom_error! {pub ProgramError
    Io {source: io::Error} = "IO Error : {source}",
    Termimad {source: termimad::Error} = "Termimad Error : {source}",
    Conf {source: ConfError} = "Bad configuration: {source}",
    ConfFile {path:String, details: ConfError} = "Bad configuration file {path:?} : {details}",
    ArgParse {bad: String, valid: String} = "{bad:?} can't be parsed (valid values: {valid:?})",
    UnknownVerb {name: String} = "No verb matches {name:?}",
    AmbiguousVerbName {name: String} = "Ambiguous name: More than one verb matches {name:?}",
    UnmatchingVerbArgs {name: String} = "No matching argument found for verb {name:?}",
    TreeBuild {source: TreeBuildError} = "{source}",
    LaunchError {program: String, source: io::Error} = "Unable to launch {program}: {source}",
    UnknowShell {shell: String} = "Unknown shell: {shell}",
    InternalError {details: String} = "Internal error: {details}", // should not happen
    InvalidGlobError {pattern: String} = "Invalid glob: {pattern}",
    Unrecognized {token: String} = "Unrecognized: {token}",
    NetError {source: NetError} = "{source}",
    ImageError {source: ImageError } = "{source}",
    Lfs {details: String} = "Failed to fetch mounts: {details}",
    ZeroLenFile = "File seems empty",
    UnmappableFile = "File can't be mapped",
    UnprintableFile = "File can't be printed", // has characters that can't be printed without escaping
    SyntectCrashed { details: String } = "Syntect crashed on {details:?}",
    OpenError { source: opener::OpenError } = "Open error: {source}",
}

custom_error! {pub TreeBuildError
    NotADirectory { path: String } = "Not a directory: {path}",
    FileNotFound { path: String } = "File not found: {path}",
    Interrupted = "Task Interrupted",
    TooManyMatches { max: usize } = "Too many matches (max allowed: {max})",
}

custom_error! {pub ConfError
    Io {source: io::Error}                          = "unable to read from the file: {source}",
    ImportNotFound {path: String}                   = "import file not found: {path:?}",
    UnknownFileExtension { path: String}            = "unexpected file extension in {path:?}",
    Toml {source: toml::de::Error}                  = "unable to parse TOML: {source}",
    Hjson {source: deser_hjson::Error}              = "unable to parse Hjson: {source}",
    Invalid                                         = "unexpected conf structure", // not expected
    MissingField {txt: String}                      = "missing field in conf",
    InvalidVerbInvocation {invocation: String}      = "invalid verb invocation: {invocation}",
    InvalidVerbConf {details: String}               = "invalid verb conf: {details}",
    UnknownInternal {verb: String}                  = "not a known internal: {verb}",
    InvalidSearchMode {details: String}             = "invalid search mode: {details}",
    InvalidKey {raw: String}                        = "not a valid key: {raw}",
    ParseKey {source: crokey::ParseKeyError}        = "{source}",
    ReservedKey {key: String}                       = "reserved key: {key}",
    UnexpectedInternalArg {invocation: String}      = "unexpected argument for internal: {invocation}",
    InvalidCols {details: String}                   = "invalid cols definition: {details}",
    InvalidSkin {source: InvalidSkinError}          = "invalid skin: {source}",
    InvalidThreadsCount { count: usize }            = "invalid threads count: {count}",
    InvalidDefaultFlags { flags: String }           = "invalid default flags: {flags:?}",
    InvalidSyntaxTheme { name: String }             = "invalid syntax theme: {name:?}",
}

// error which can be raised when parsing a pattern the user typed
custom_error! {pub PatternError
    InvalidMode { mode: String } = "Invalid search mode: {mode:?}",
    InvalidRegex {source: regex::Error} = @{
        format!("Invalid Regular Expression: {}", source.to_string().lines().last().unwrap_or(""))
    },
    UnknownRegexFlag {bad: char} = "Unknown regular expression flag: {bad:?}",
}

custom_error! {pub InvalidSkinError
    InvalidColor { raw : String }  = "'{raw}' is not a valid color",
    InvalidAttribute { raw : String }  = "'{raw}' is not a valid style attribute",
    InvalidGreyLevel { level: u8 } = "grey level must be between 0 and 23 (got {level})",
    InvalidStyle {style: String}   = "Invalid skin style : {style}",
}

custom_error! {pub NetError
    SocketNotAvailable { path : String } = "Can't open socket: {path} already exists - consider removing it",
    Io {source: io::Error}               = "error on the socket: {source}",
    InvalidMessage                       = "invalid message received",
}

