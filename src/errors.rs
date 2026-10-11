//! Definitions of custom errors used in broot
//!
//! Error messages are markdown: they're rendered with termimad, either in the status
//! line or by [`ProgramError::report`], so interpolated values should be in backticks.

use {
    lazy_regex::regex,
    std::{
        io::{
            self,
            IsTerminal,
            Write,
        },
        path::PathBuf,
        process::ExitCode,
    },
    termimad::{
        CompoundStyle,
        MadSkin,
        crossterm::style::{
            Attribute,
            Color,
        },
    },
};

#[derive(thiserror::Error, Debug)]
pub enum ProgramError {
    #[error("Ambiguous name: More than one verb matches `{name}`")]
    AmbiguousVerbName { name: String },
    #[error("`{bad}` can't be parsed (valid values: `{valid}`)")]
    ArgParse { bad: String, valid: String },
    #[error("**Bad configuration file** `{}`: {details}", .path.display())]
    ConfFile {
        path: PathBuf,
        #[source]
        details: ConfError,
    },
    #[error("**Bad configuration:** {source}")]
    Conf {
        #[from]
        source: ConfError,
    },
    #[error("Git error: {details}")]
    Git { details: String },
    #[error("Image error: {details}")]
    ImageError { details: String },
    #[error("Internal error: {details}")]
    Internal { details: String }, // should not happen
    #[error("IO Error : {source}")]
    Io {
        #[from]
        source: io::Error,
    },
    #[error("Unable to launch `{program}`: {source}")]
    LaunchError { program: String, source: io::Error },
    #[error("Failed to fetch mounts: {details}")]
    Lfs { details: String },
    #[error(transparent)]
    NetError {
        #[from]
        source: NetError,
    },
    #[error("Open error: {source}")]
    OpenError {
        #[from]
        source: opener::OpenError,
    },
    #[error(transparent)]
    ShelInstall {
        #[from]
        source: ShellInstallError,
    },
    #[error("SVG error: {source}")]
    Svg {
        #[from]
        source: SvgError,
    },
    #[error("Syntect crashed on `{details}`")]
    SyntectCrashed { details: String },
    #[error("Termimad Error : {source}")]
    Termimad {
        #[from]
        source: termimad::Error,
    },
    #[error("Trash error: {message}")]
    Trash { message: String },
    #[error(transparent)]
    TreeBuild {
        #[from]
        source: TreeBuildError,
    },
    #[error("Unknown shell: `{shell}`")]
    UnknownShell { shell: String },
    #[error("No verb matches `{name}`")]
    UnknownVerb { name: String },
    #[error("File can't be mapped")]
    UnmappableFile,
    #[error("No matching argument found for verb `{name}`")]
    UnmatchingVerbArgs { name: String },
    #[error("File can't be printed")]
    UnprintableFile, // has characters that can't be printed without escaping
    #[error("Unrecognized: `{token}`")]
    Unrecognized { token: String },
    #[error("File seems empty")]
    ZeroLenFile,
    #[error("Notify error: {source}")]
    Notify {
        #[from]
        source: notify::Error,
    },
}

impl ProgramError {
    /// Whether the error comes from writing to a closed pipe
    pub fn is_broken_pipe(&self) -> bool {
        match self {
            Self::Io { source } | Self::Termimad { source: termimad::Error::IO(source) } => {
                source.kind() == io::ErrorKind::BrokenPipe
            }
            _ => false,
        }
    }
    /// Print the error on stderr (unless it's a broken pipe)
    /// and return the matching exit code
    pub fn report(&self) -> ExitCode {
        if self.is_broken_pipe() {
            return ExitCode::from(141);
        }
        let mut stderr = io::stderr();
        let skin = if stderr.is_terminal() {
            MadSkin {
                bold: CompoundStyle::new(Some(Color::Red), None, Attribute::Bold.into()),
                inline_code: CompoundStyle::with_fg(Color::Yellow),
                ..Default::default()
            }
        } else {
            MadSkin::no_style()
        };
        for line in self.to_string().lines() {
            let _ = writeln!(stderr, "{}", skin.inline(line));
        }
        ExitCode::FAILURE
    }
}

impl From<image::ImageError> for ProgramError {
    fn from(err: image::ImageError) -> Self {
        ProgramError::ImageError {
            details: err.to_string(),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ShellInstallError {
    #[error("IO Error {source} on {when}")]
    Io { source: io::Error, when: String },
}
impl ShellInstallError {
    pub fn is_permission_denied(&self) -> bool {
        match self {
            Self::Io { source, .. } => {
                if source.kind() == io::ErrorKind::PermissionDenied {
                    true
                } else {
                    cfg!(windows) && source.raw_os_error().unwrap_or(0) == 1314
                }
            }
        }
    }
}
pub trait IoToShellInstallError<Ok> {
    fn context(
        self,
        f: &dyn Fn() -> String,
    ) -> Result<Ok, ShellInstallError>;
}
impl<Ok> IoToShellInstallError<Ok> for Result<Ok, io::Error> {
    fn context(
        self,
        f: &dyn Fn() -> String,
    ) -> Result<Ok, ShellInstallError> {
        self.map_err(|source| ShellInstallError::Io { source, when: f() })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum TreeBuildError {
    #[error("File not found: `{}`", .path.display())]
    FileNotFound { path: PathBuf },
    #[error("Task Interrupted")]
    Interrupted,
    #[error("Invalid UTF-8 in `{}`", .path.display())]
    InvalidUtf8 { path: PathBuf },
    #[error("Not a directory: `{}`", .path.display())]
    NotADirectory { path: PathBuf },
    #[error("Not a descendant of the root: `{}`", .path.display())]
    NotARootDescendant { path: PathBuf },
    #[error("Too many matches (max allowed: {max})")]
    TooManyMatches { max: usize },
    #[error("Inconsistent data: {message}")]
    InconsistentData { message: String }, // maybe refresh ?
}

#[derive(thiserror::Error, Debug)]
pub enum ConfError {
    #[error("unable to read from the file: {source}")]
    Io {
        #[from]
        source: io::Error,
    },
    #[error("import file not found: `{path}`")]
    ImportNotFound { path: String },
    #[error("unexpected file extension in `{}`", .path.display())]
    UnknownFileExtension { path: PathBuf },
    #[error("unable to parse TOML: {source}")]
    Toml {
        #[from]
        source: toml::de::Error,
    },
    #[error("unable to parse Hjson: {source}")]
    Hjson {
        #[from]
        source: deser_hjson::Error,
    },
    #[error("unexpected conf structure")]
    Invalid, // not expected
    #[error("missing field in conf")]
    MissingField { txt: String },
    #[error("invalid verb invocation: `{invocation}`")]
    InvalidVerbInvocation { invocation: String },
    #[error("invalid verb conf: {details}")]
    InvalidVerbConf { details: String },
    #[error("not a known internal: `{verb}`")]
    UnknownInternal { verb: String },
    #[error("invalid search mode: {details}")]
    InvalidSearchMode { details: String },
    #[error("not a valid key: `{raw}`")]
    InvalidKey { raw: String },
    #[error(transparent)]
    ParseKey {
        #[from]
        source: crokey::ParseKeyError,
    },
    #[error("reserved key: `{key}`")]
    ReservedKey { key: String },
    #[error("unexpected argument for internal: `{invocation}`")]
    UnexpectedInternalArg { invocation: String },
    #[error("invalid cols definition: {details}")]
    InvalidCols { details: String },
    #[error("invalid skin: {source}")]
    InvalidSkin {
        #[from]
        source: InvalidSkinError,
    },
    #[error("invalid threads count: {count}")]
    InvalidThreadsCount { count: usize },
    #[error("invalid default flags: `{flags}`")]
    InvalidDefaultFlags { flags: String },
    #[error("invalid syntax theme: `{name}`")]
    InvalidSyntaxTheme { name: String },
    #[error("invalid glob pattern: `{pattern}`")]
    InvalidGlobPattern { pattern: String },
    #[error(
        "invalid verb name: `{name}` (must either not start with a special character or be only made of special characters)"
    )]
    InvalidVerbName { name: String },
    #[error("unknown verb argument flag: `{name}`")]
    UnknownVerbArgFlag { name: String },
    #[error(
        "unknown flag in invocation argument: `{name}` (expected `theme` or the name of a standard argument)"
    )]
    UnknownInvocationArgFlag { name: String },
    #[error(
        "invalid verb in `{}`:\n{details}\nFaulty verb configuration:\n{verb_conf}",
        .file.as_deref().map_or_else(|| "?".to_string(), |path| path.display().to_string()),
    )]
    InvalidVerb {
        file: Option<PathBuf>,
        #[source]
        details: Box<ConfError>,
        verb_conf: String,
    },
    #[error(
        "unknown argument `{name}`: it's neither provided by broot nor defined in the invocation pattern"
    )]
    UnknownVerbArg { name: String },
    #[error(
        "unknown argument `{name}` in `terminal_title`: only the arguments provided by broot can be used"
    )]
    UnknownTerminalTitleArg { name: String },
    #[error("invalid panel reference: `{raw}`")]
    InvalidPanelReference { raw: String },
}

/// Error which can be raised when parsing a pattern the user typed
#[derive(thiserror::Error, Debug)]
pub enum PatternError {
    #[error("Invalid search mode: `{mode}`")]
    InvalidMode { mode: String },
    #[error(
        "Invalid Regular Expression: {}",
        .source.to_string().lines().last().unwrap_or(""),
    )]
    InvalidRegex {
        #[from]
        source: regex::Error,
    },
    #[error("Unknown regular expression flag: `{bad}`")]
    UnknownRegexFlag { bad: char },
}

#[derive(thiserror::Error, Debug)]
pub enum InvalidSkinError {
    #[error("invalid color: {source}")]
    InvalidColor {
        #[from]
        source: termimad::ParseColorError,
    },
    #[error("`{raw}` is not a valid style attribute")]
    InvalidAttribute { raw: String },
    #[error("grey level must be between 0 and 23 (got {level})")]
    InvalidGreyLevel { level: u8 },
    #[error("Invalid skin style: `{style}`")]
    InvalidStyle { style: String },
    #[error(transparent)]
    InvalidStyleToken {
        #[from]
        source: termimad::ParseStyleTokenError,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum NetError {
    #[error("Can't open socket: `{path}` already exists - consider removing it")]
    SocketNotAvailable { path: String },
    #[error("a broot server with the name `{name}` is already running")]
    DuplicateServerName { name: String },
    #[error("error on the socket: {source}")]
    Io {
        #[from]
        source: io::Error,
    },
    #[error("invalid message received")]
    InvalidMessage,
}

#[derive(thiserror::Error, Debug)]
pub enum SvgError {
    #[error("IO Error : {source}")]
    Io {
        #[from]
        source: io::Error,
    },
    #[error("Internal error : {message}")]
    Internal { message: &'static str },
    #[error("SVG Error: {source}")]
    Svg {
        #[from]
        source: resvg::usvg::Error,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum PreviewTransformerError {
    #[error("Invalid input")]
    InvalidInput,
    #[error("No output")]
    NoOutput,
    #[error("IO Error : {source}")]
    Io {
        #[from]
        source: io::Error,
    },
    #[error("Process interrupted")]
    ProcessInterrupted,
    #[error("Execution failed with code {code}")]
    ProcessFailed { code: i32 },
}
