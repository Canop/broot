/// this module manages reading and translating
/// the arguments passed on launch of the application.

use {
    crate::{
        errors::{ProgramError, TreeBuildError},
        tree_options::{OptionBool, TreeOptions},
    },
    std::{
        env,
        io,
        path::{Path, PathBuf},
    },
};

/// the parsed program launch arguments
pub struct AppLaunchArgs {
    pub root: PathBuf,                    // what should be the initial root
    pub file_export_path: Option<String>, // where to write the produced path (if required with --out)
    pub cmd_export_path: Option<String>, // where to write the produced command (if required with --outcmd or -oc)
    pub tree_options: TreeOptions,       // initial tree options
    pub commands: Option<String>,        // commands passed as cli argument, still unparsed
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
    Ok(
        if root.is_relative() {
            env::current_dir()?.join(root)
        } else {
            root.to_path_buf()
        }
    )
}

/// return the parsed launch arguments
pub fn read_launch_args() -> Result<AppLaunchArgs, ProgramError> {
    let cli_args = crate::clap::clap_app().get_matches();
    let mut root = match cli_args.value_of("root") {
        Some(path) => {

            /*
            // On windows we unwrap the argument if it's given between
            // double quotes, which notably happens when broot is called
            // because mapped to a right click action in Explorer.
            // Related issue: #72
            // I'm not sure it's the best way. Inputs welcome.
            #[cfg(windows)] {
                if path.starts_with('"') && path.ends_with('"') {
                    path = &path[1..path.len()-1];
                }
            }
            */

            PathBuf::from(path)
        }
        None => env::current_dir()?,
    };
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
    let file_export_path = cli_args
        .value_of("file_export_path")
        .and_then(|s| Some(s.to_owned()));
    let cmd_export_path = cli_args
        .value_of("cmd_export_path")
        .and_then(|s| Some(s.to_owned()));
    let commands = cli_args
        .value_of("commands")
        .and_then(|s| Some(s.to_owned()));
    let no_style = cli_args.is_present("no-style");
    let height = cli_args.value_of("height").and_then(|s| s.parse().ok());
    Ok(AppLaunchArgs {
        root,
        file_export_path,
        cmd_export_path,
        tree_options,
        commands,
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
