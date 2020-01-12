/// this module manages reading and translating
/// the arguments passed on launch of the application.
use {
    crate::{
        errors::{ProgramError, TreeBuildError},
        shell_install::ShellInstallState,
        tree_options::{OptionBool, TreeOptions},
    },
    std::{
        env, io,
        path::{Path, PathBuf},
    },
};

/// the parsed program launch arguments
pub struct AppLaunchArgs {
    pub root: PathBuf,                                // what should be the initial root
    pub file_export_path: Option<String>, // where to write the produced path (if required with --out)
    pub cmd_export_path: Option<String>, // where to write the produced command (if required with --outcmd)
    pub print_shell_function: Option<String>, // shell function to print on stdout
    pub set_install_state: Option<ShellInstallState>, // the state to set
    pub tree_options: TreeOptions,       // initial tree options
    pub commands: Option<String>,        // commands passed as cli argument, still unparsed
    pub command_separator: Option<String>, // The separator to use for commands. Defaults to ;
    pub install: bool,                   // installation is required
    pub height: Option<u16>,             // an optional height to replace the screen's one
    pub no_style: bool,                  // whether to remove all styles (including colors)
}

#[cfg(not(windows))]
fn canonicalize_root(root: &Path) -> io::Result<PathBuf> {
    root.canonicalize()
}

#[cfg(windows)]
fn canonicalize_root(root: &Path) -> io::Result<PathBuf> {
    Ok(if root.is_relative() {
        env::current_dir()?.join(root)
    } else {
        root.to_path_buf()
    })
}

/// return the parsed launch arguments
pub fn read_launch_args() -> Result<AppLaunchArgs, ProgramError> {
    let cli_args = crate::clap::clap_app().get_matches();
    let mut root = cli_args
        .value_of("root")
        .map_or(env::current_dir()?, PathBuf::from);
    if !root.exists() {
        Err(TreeBuildError::FileNotFound {
            path: format!("{:?}", &root),
        })?;
    }
    if !root.is_dir() {
        // we try to open the parent directory if the passed file isn't one
        if let Some(parent) = root.parent() {
            info!("Passed path isn't a directory => opening parent instead");
            root = parent.to_path_buf();
        } else {
            // let's give up
            Err(TreeBuildError::NotADirectory {
                path: format!("{:?}", &root),
            })?;
        }
    }

    let root = canonicalize_root(&root)?;

    let mut tree_options = TreeOptions::default();
    tree_options.show_sizes = cli_args.is_present("sizes");
    if tree_options.show_sizes {
        // by default, if we're asked to show the size, we show all files
        tree_options.show_hidden = true;
        tree_options.respect_git_ignore = OptionBool::No;
    }
    tree_options.only_folders = cli_args.is_present("only-folders");
    tree_options.show_hidden = cli_args.is_present("hidden");
    tree_options.show_dates = cli_args.is_present("dates");
    tree_options.show_permissions = cli_args.is_present("permissions");
    if let Some(respect_ignore) = cli_args.value_of("gitignore") {
        tree_options.respect_git_ignore = respect_ignore.parse()?;
    }
    let install = cli_args.is_present("install");
    let file_export_path = cli_args.value_of("file_export_path").map(str::to_string);
    let cmd_export_path = cli_args.value_of("cmd_export_path").map(str::to_string);
    let commands = cli_args.value_of("commands").map(str::to_string);
    let command_separator = cli_args.value_of("command_separator").map(str::to_string);
    let no_style = cli_args.is_present("no-style");
    let height = cli_args.value_of("height").and_then(|s| s.parse().ok());
    let print_shell_function = cli_args
        .value_of("print-shell-function")
        .map(str::to_string);
    let set_install_state = cli_args
        .value_of("set-install-state")
        .map(ShellInstallState::from_str)
        .transpose()?;
    Ok(AppLaunchArgs {
        root,
        file_export_path,
        cmd_export_path,
        print_shell_function,
        set_install_state,
        tree_options,
        commands,
        command_separator,
        install,
        height,
        no_style,
    })
}

/// wait for user input, return `true` if she
/// didn't answer 'n'
pub fn ask_authorization() -> Result<bool, ProgramError> {
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    let answer = answer.trim();
    Ok(match answer.as_ref() {
        "n" | "N" => false,
        _ => true,
    })
}
