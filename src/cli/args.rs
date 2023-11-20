// Warning: this module can't import broot's stuf due to its use in build.rs
use {
    clap::{Parser, ValueEnum},
    std::{
        path::PathBuf,
        str::FromStr,
    },
};

/// Launch arguments
#[derive(Debug, Parser)]
#[command(author, about, version, disable_version_flag = true, disable_help_flag = true)]
pub struct Args {

    /// Print help information
    #[arg(long)]
    pub help: bool,

    /// print the version
    #[arg(long)]
    pub version: bool,

    /// Semicolon separated paths to specific config files
    #[arg(long, value_name = "paths")]
    pub conf: Option<String>,

    /// Show the last modified date of files and directories
    #[arg(short, long)]
    pub dates: bool,

    /// Don't show the last modified date
    #[arg(short='D', long)]
    pub no_dates: bool,

    #[arg(short='f', long)]
    /// Only show folders
    pub only_folders: bool,

    /// Show folders and files alike
    #[arg(short='F', long)]
    pub no_only_folders: bool,

    /// Show filesystem info on top
    #[arg(long)]
    pub show_root_fs: bool,

    /// Show git statuses on files and stats on repo
    #[arg(short='g', long)]
    pub show_git_info: bool,

    /// Don't show git statuses on files and stats on repo
    #[arg(short='G', long)]
    pub no_show_git_info: bool,

    #[arg(long)]
    /// Only show files having an interesting git status, including hidden ones
    pub git_status: bool,

    #[arg(short='h', long)]
    /// Show hidden files
    pub hidden: bool,

    #[arg(short='H', long)]
    /// Don't show hidden files
    pub no_hidden: bool,

    #[arg(short='i', long)]
    /// Show git ignored files
    pub git_ignored: bool,

    #[arg(short='I', long)]
    /// Don't show git ignored files
    pub no_git_ignored: bool,

    #[arg(short='p', long)]
    /// Show permissions
    pub permissions: bool,

    #[arg(short='P', long)]
    /// Don't show permissions
    pub no_permissions: bool,

    #[arg(short='s', long)]
    /// Show the size of files and directories
    pub sizes: bool,

    #[arg(short='S', long)]
    /// Don't show sizes
    pub no_sizes: bool,

    #[arg(long)]
    /// Sort by count (only show one level of the tree)
    pub sort_by_count: bool,

    #[arg(long)]
    /// Sort by date (only show one level of the tree)
    pub sort_by_date: bool,

    #[arg(long)]
    /// Sort by size (only show one level of the tree)
    pub sort_by_size: bool,

    #[arg(long)]
    /// Same as sort-by-type-dirs-first
    pub sort_by_type: bool,

    #[arg(long)]
    /// Do not show the tree, even if allowed by sorting mode.
    pub no_tree: bool,

    #[arg(long)]
    /// Show the tree, when allowed by sorting mode.
    pub tree: bool,

    #[arg(long)]
    /// Sort by type, directories first (only show one level of the tree)
    pub sort_by_type_dirs_first: bool,

    #[arg(long)]
    /// Sort by type, directories last (only show one level of the tree)
    pub sort_by_type_dirs_last: bool,

    /// Don't sort
    #[arg(long)]
    pub no_sort: bool,

    /// Sort by size, show ignored and hidden files
    #[arg(short, long)]
    pub whale_spotting: bool,

    /// Trim the root too and don't show a scrollbar
    #[arg(short='t', long)]
    pub trim_root: bool,

    /// Don't trim the root level, show a scrollbar
    #[arg(short='T', long)]
    pub no_trim_root: bool,

    /// Where to write the produced cmd (if any)
    #[arg(long, value_name = "path")]
    pub outcmd: Option<PathBuf>,

    /// Semicolon separated commands to execute
    #[arg(short, long, value_name = "cmd")]
    pub cmd: Option<String>,

    /// Whether to have styles and colors (default is usually OK)
    #[arg(long, default_value="auto", value_name = "color")]
    pub color: TriBool,

    /// Height (if you don't want to fill the screen or for file export)
    #[arg(long, value_name = "height")]
    pub height: Option<u16>,

    /// Install or reinstall the br shell function
    #[arg(long)]
    pub install: bool,

    /// Where to write the produced cmd (if any)
    #[arg(long, value_name = "state")]
    pub set_install_state: Option<CliShellInstallState>,

    /// Print to stdout the br function for a given shell
    #[arg(long, value_name = "shell")]
    pub print_shell_function: Option<String>,

    /// A socket to listen to for commands
    #[cfg(unix)]
    #[arg(long, value_name = "socket")]
    pub listen: Option<String>,

    /// Ask for the current root of the remote broot
    #[cfg(unix)]
    #[arg(long)]
    pub get_root: bool,

    /// Write default conf files in given directory
    #[arg(long, value_name = "path")]
    pub write_default_conf: Option<PathBuf>,

    /// A socket to send commands to
    #[cfg(unix)]
    #[arg(long, value_name = "socket")]
    pub send: Option<String>,

    /// Root Directory
    pub root: Option<PathBuf>,
}

/// This is an Option<bool> but I didn't find any way to configure
/// clap to parse an Option<T> as I want
#[derive(ValueEnum)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriBool {
    Auto,
    Yes,
    No,
}
impl TriBool {
    pub fn unwrap_or_else<F>(self, f: F) -> bool
    where
        F: FnOnce() -> bool
    {
        match self {
            Self::Auto => f(),
            Self::Yes => true,
            Self::No => false,
        }
    }
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum CliShellInstallState {
    Undefined, // before any install, this is the initial state
    Refused,
    Installed,
}
impl FromStr for CliShellInstallState {
    type Err = String;
    fn from_str(state: &str) -> Result<Self, Self::Err> {
        match state {
            "undefined" => Ok(Self::Undefined),
            "refused" => Ok(Self::Refused),
            "installed" => Ok(Self::Installed),
            _ => Err(
                format!("unexpected install state: {state:?}")
            ),
        }
    }
}
