//! Definitions of custom errors used in broot

use {
    custom_error::custom_error,
    image::error::ImageError,
    lazy_regex::regex,
    std::io,
};

custom_error! {pub ProgramError
    AmbiguousVerbName {name: String} = "Ambiguous name: More than one verb matches {name:?}",
    ArgParse {bad: String, valid: String} = "{bad:?} can't be parsed (valid values: {valid:?})",
    Conf {source: ConfError} = "Bad configuration: {source}",
    ConfFile {path:String, details: ConfError} = "Bad configuration file {path:?} : {details}",
    ImageError {source: ImageError } = "{source}",
    InternalError {details: String} = "Internal error: {details}", // should not happen
    InvalidGlobError {pattern: String} = "Invalid glob: {pattern}",
    Io {source: io::Error} = "IO Error : {source}",
    LaunchError {program: String, source: io::Error} = "Unable to launch {program}: {source}",
    Lfs {details: String} = "Failed to fetch mounts: {details}",
    NetError {source: NetError} = "{source}",
    OpenError { source: opener::OpenError } = "Open error: {source}",
    ShelInstall { source: ShellInstallError } = "{source}",
    Svg {source: SvgError} = "SVG error: {source}",
    SyntectCrashed { details: String } = "Syntect crashed on {details:?}",
    Termimad {source: termimad::Error} = "Termimad Error : {source}",
    TreeBuild {source: TreeBuildError} = "{source}",
    UnknowShell {shell: String} = "Unknown shell: {shell}",
    UnknownVerb {name: String} = "No verb matches {name:?}",
    UnmappableFile = "File can't be mapped",
    UnmatchingVerbArgs {name: String} = "No matching argument found for verb {name:?}",
    UnprintableFile = "File can't be printed", // has characters that can't be printed without escaping
    Unrecognized {token: String} = "Unrecognized: {token}",
    ZeroLenFile = "File seems empty",
}

custom_error!{pub ShellInstallError
    Io {source: io::Error, when: String} = "IO Error {source} on {when}",
}
impl ShellInstallError {
    pub fn is_permission_denied(&self) -> bool {
        match self {
            Self::Io { source, .. } => {
                if source.kind() == io::ErrorKind::PermissionDenied {
                    true
                } else { cfg!(windows) && source.raw_os_error().unwrap_or(0) == 1314 }
            }
        }
    }
}
pub trait IoToShellInstallError<Ok> {
    fn context(self, f: &dyn Fn() -> String) -> Result<Ok, ShellInstallError>;
}
impl<Ok> IoToShellInstallError<Ok> for Result<Ok, io::Error> {
    fn context(self, f: &dyn Fn() -> String) -> Result<Ok, ShellInstallError> {
        self.map_err(|source| ShellInstallError::Io {
            source, when: f()
        })
    }
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
    InvalidGlobPattern { pattern: String }          = "invalid glob pattern: {pattern:?}",
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
    InvalidColor {source: termimad::ParseColorError}  = "invalid color: {source}",
    InvalidAttribute {raw : String}  = "'{raw}' is not a valid style attribute",
    InvalidGreyLevel {level: u8} = "grey level must be between 0 and 23 (got {level})",
    InvalidStyle {style: String}   = "Invalid skin style : {style}",
    InvalidStyleToken {source: termimad::ParseStyleTokenError} = "{source}",
}

custom_error! {pub NetError
    SocketNotAvailable { path : String } = "Can't open socket: {path} already exists - consider removing it",
    Io {source: io::Error}               = "error on the socket: {source}",
    InvalidMessage                       = "invalid message received",
}

custom_error! {pub SvgError
    Io {source: io::Error} = "IO Error : {source}",
    Internal { message: &'static str } = "Internal error : {message}",
    Svg {source: resvg::usvg::Error} = "SVG Error: {source}",
}
