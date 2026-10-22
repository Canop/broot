#[cfg(any(unix, windows))]
use crate::net::NetError;
use {
    crate::{
        conf::ConfError,
        image::SvgError,
        shell_install::ShellInstallError,
        tree_build::TreeBuildError,
    },
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
// Note: the error messages are markdown, rendered with termimad by [`ProgramError::report`] or in the status line
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
    #[cfg(any(unix, windows))]
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
