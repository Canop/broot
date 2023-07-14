// Warning: this module can't import broot's stuf
use {
    clap::Parser,
    std::{
        path::PathBuf,
        str::FromStr,
    },
};

#[derive(Debug, Parser)]
/// A tree explorer and a customizable launcher
///
/// Complete documentation lives at https://dystroy.org/broot"
#[command(author, version, about, disable_help_flag = true)]
pub struct Args {

    /// Show the last modified date of files and directories"
    #[arg(short, long, action)]
    pub dates: bool,

    /// Don't show the last modified date"
    #[arg(short='D', long, action)]
    pub no_dates: bool,

    #[arg(short='f', long, action)]
    /// Only show folders
    pub only_folders: bool,

    /// Show folders and files alike
    #[arg(short='F', long, action)]
    pub no_only_folders: bool,

    /// Show filesystem info on top
    #[arg(long, action)]
    pub show_root_fs: bool,

    /// Show git statuses on files and stats on repo
    #[arg(short='g', long, action)]
    pub show_git_info: bool,

    /// Don't show git statuses on files and stats on repo
    #[arg(short='G', long, action)]
    pub no_show_git_info: bool,

    #[arg(long, action)]
    /// Only show files having an interesting git status, including hidden ones
    pub git_status: bool,

    /// Print help information
    #[arg(long)]
    pub help: bool,

    #[arg(short='h', long, action)]
    /// Show hidden files
    pub hidden: bool,

    #[arg(short='H', long, action)]
    /// Don't show hidden files
    pub no_hidden: bool,

    #[arg(short='i', long, action)]
    /// Show git ignored files
    pub git_ignored: bool,

    #[arg(short='I', long, action)]
    /// Don't show git ignored files
    pub no_git_ignored: bool,

    #[arg(short='p', long, action)]
    /// Show permissions
    pub permissions: bool,

    #[arg(short='P', long, action)]
    /// Don't show permissions
    pub no_permissions: bool,

    #[arg(short='s', long, action)]
    /// Show the size of files and directories
    pub sizes: bool,

    #[arg(short='S', long, action)]
    /// Don't show sizes
    pub no_sizes: bool,

    #[arg(long, action)]
    /// Sort by count (only show one level of the tree)
    pub sort_by_count: bool,

    #[arg(long, action)]
    /// Sort by date (only show one level of the tree)
    pub sort_by_date: bool,

    #[arg(long, action)]
    /// Sort by size (only show one level of the tree)
    pub sort_by_size: bool,

    #[arg(long, action)]
    /// Same as sort-by-type-dirs-first
    pub sort_by_type: bool,

    #[arg(long, action)]
    /// Sort by type, directories first (only show one level of the tree)
    pub sort_by_type_dirs_first: bool,

    #[arg(long, action)]
    /// Sort by type, directories last (only show one level of the tree)
    pub sort_by_type_dirs_last: bool,

    /// Sort by size, show ignored and hidden files
    #[arg(short, long, action)]
    pub whale_spotting: bool,

    /// Don't sort
    #[arg(long, action)]
    pub no_sort: bool,

    /// Trim the root too and don't show a scrollbar
    #[arg(short='t', long, action)]
    pub trim_root: bool,

    /// Don't trim the root level, show a scrollbar
    #[arg(short='T', long, action)]
    pub no_trim_root: bool,

    /// Where to write the produced cmd (if any)
    #[arg(long)]
    pub outcmd: Option<PathBuf>,

    /// Semicolon separated commands to execute
    #[arg(short, long)]
    pub cmd: Option<String>,

    /// Whether to have styles and colors (auto is default and usually OK)
    #[arg(long, default_value="auto")]
    pub color: TriBool,

    /// Semicolon separated paths to specific config files"),
    #[arg(long)]
    pub conf: Option<String>,

    /// Height (if you don't want to fill the screen or for file export)
    #[arg(long)]
    pub height: Option<u16>,

    /// Install or reinstall the br shell function
    #[arg(long, action)]
    pub install: bool,

    /// Where to write the produced cmd (if any)
    #[arg(long)]
    pub set_install_state: Option<CliShellInstallState>,

    /// Print to stdout the br function for a given shell
    #[arg(long)]
    pub print_shell_function: Option<String>,

    /// A socket to listen to for commands
    #[cfg(unix)]
    #[arg(long)]
    pub listen: Option<String>,

    /// Ask for the current root of the remote broot
    #[cfg(unix)]
    #[arg(long, action)]
    pub get_root: bool,

    /// Write default conf files in given directory
    #[arg(long)]
    pub write_default_conf: Option<PathBuf>,

    /// A socket that broot sends commands to before quitting
    #[cfg(unix)]
    #[arg(long)]
    pub send: Option<String>,

    /// Root Directory
    #[arg(value_parser, value_name="FILE")]
    pub root: Option<PathBuf>,
}

/// This is an Option<bool> but I didn't find any way to configure
/// clap to parse an Option<T> as I want
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
impl FromStr for TriBool {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" | "a" => Ok(Self::Auto),
            "yes" | "y" => Ok(Self::Yes),
            "no" | "n" => Ok(Self::No),
            _ => Err(
                format!("unexpected value: should be 'auto', 'yes', or 'no'")
            ),
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

