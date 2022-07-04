use {
    std::{
        path::PathBuf,
        str::FromStr,
    },
};

#[derive(Debug, clap::Parser)]
/// A tree explorer and a customizable launcher
///
/// Complete documentation lives at https://dystroy.org/broot"
#[clap(author, version, about)]
pub struct Args {

    /// Show the last modified date of files and directories"
    #[clap(short, long, action)]
    pub dates: bool,

    /// Don't show the last modified date"
    #[clap(short='D', long, action)]
    pub no_dates: bool,

    #[clap(short='f', long, action)]
    /// Only show folders
    pub only_folders: bool,

    /// Show folders and files alike
    #[clap(short='F', long, action)]
    pub no_only_folders: bool,

    /// Show filesystem info on top
    #[clap(long, action)]
    pub show_root_fs: bool,

    /// Show git statuses on files and stats on repo
    #[clap(short='g', long, action)]
    pub show_git_info: bool,

    /// Don't show git statuses on files and stats on repo
    #[clap(short='G', long, action)]
    pub no_show_git_info: bool,

    #[clap(long, action)]
    /// Only show files having an interesting git status, including hidden ones
    pub git_status: bool,

    #[clap(short='h', long, action)]
    /// Show hidden files
    pub hidden: bool,

    #[clap(short='H', long, action)]
    /// Don't show hidden files
    pub no_hidden: bool,

    #[clap(short='i', long, action)]
    /// Show git ignored files
    pub git_ignored: bool,

    #[clap(short='I', long, action)]
    /// Don't show git ignored files
    pub no_git_ignored: bool,

    #[clap(short='p', long, action)]
    /// Show permissions
    pub permissions: bool,

    #[clap(short='P', long, action)]
    /// Don't show permissions
    pub no_permissions: bool,

    #[clap(short='s', long, action)]
    /// Show the size of files and directories
    pub sizes: bool,

    #[clap(short='S', long, action)]
    /// Don't show sizes
    pub no_sizes: bool,

    #[clap(long, action)]
    /// Sort by count (only show one level of the tree)
    pub sort_by_count: bool,

    #[clap(long, action)]
    /// Sort by date (only show one level of the tree)
    pub sort_by_date: bool,

    #[clap(long, action)]
    /// Sort by size (only show one level of the tree)
    pub sort_by_size: bool,

    /// Sort by size, show ignored and hidden files
    #[clap(short, long, action)]
    pub whale_spotting: bool,

    /// Don't sort
    #[clap(long, action)]
    pub no_sort: bool,

    /// Trim the root too and don't show a scrollbar
    #[clap(short='t', long, action)]
    pub trim_root: bool,

    /// Don't trim the root level, show a scrollbar
    #[clap(short='T', long, action)]
    pub no_trim_root: bool,

    /// Where to write the produced cmd (if any)
    #[clap(long, value_parser)]
    pub outcmd: Option<PathBuf>,

    /// Semicolon separated commands to execute
    #[clap(short, long, value_parser)]
    pub commands: Option<String>,

    /// Whether to have styles and colors (auto is default and usually OK)
    #[clap(long, arg_enum, value_parser, default_value="auto")]
    pub color: TriBool,

    /// Semicolon separated paths to specific config files"),
    #[clap(long, value_parser)]
    pub conf: Option<String>,

    /// Height (if you don't want to fill the screen or for file export)
    #[clap(long, value_parser)]
    pub height: Option<u16>,

    /// Install or reinstall the br shell function
    #[clap(long, action)]
    pub install: bool,

    /// Where to write the produced cmd (if any)
    #[clap(long, value_parser)]
    pub set_install_state: Option<ShellInstallState>,

    /// Print to stdout the br function for a given shell
    #[clap(long, value_parser)]
    pub print_shell_function: Option<String>,

    /// A socket to listen to for commands
    #[cfg(unix)]
    #[clap(long, value_parser)]
    pub listen: Option<String>,

    /// Ask for the current root of the remote broot
    #[cfg(unix)]
    #[clap(long, action)]
    pub get_root: bool,

    /// Write default conf files in given directory
    #[clap(long, value_parser)]
    pub write_default_conf: Option<PathBuf>,

    /// A socket that broot sends commands to before quitting
    #[cfg(unix)]
    #[clap(long, value_parser)]
    pub send: Option<String>,

    /// Root Directory
    #[clap(value_parser, value_name="FILE")]
    pub root: Option<PathBuf>,
}

/// This is an Option<bool> but I didn't find any way to configure
/// clap to parse an Option<T> as I want
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ArgEnum)]
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
pub enum ShellInstallState {
    Undefined, // before any install, this is the initial state
    Refused,
    Installed,
}
impl FromStr for ShellInstallState {
    type Err = String;
    fn from_str(state: &str) -> Result<Self, Self::Err> {
        match state {
            "undefined" => Ok(Self::Undefined),
            "refused" => Ok(Self::Refused),
            "installed" => Ok(Self::Installed),
            _ => Err(
                // not supposed to happen because claps check the values
                format!("unexpected install state: {:?}", state)
            ),
        }
    }
}

