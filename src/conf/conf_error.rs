use {
    crate::{
        skin::InvalidSkinError,
        verb::ExecArgFlag,
    },
    std::{
        io,
        path::PathBuf,
    },
};

#[derive(thiserror::Error, Debug)]
// Note: the error messages are markdown, rendered with termimad by [`ProgramError::report`] or in the status line
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
        "the `{flag}` flag can't be used on `{name}`, which is provided by broot: it only applies to an argument defined in the invocation pattern"
    )]
    PathFlagOnStandardArg { name: String, flag: ExecArgFlag },
    #[error(
        "unknown argument `{name}` in `terminal_title`: only the arguments provided by broot can be used"
    )]
    UnknownTerminalTitleArg { name: String },
    #[error("invalid panel reference: `{raw}`")]
    InvalidPanelReference { raw: String },
}
