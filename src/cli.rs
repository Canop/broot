/// this module manages reading and translating
/// the arguments passed on launch of the application.
use {
    crate::{
        app::{App, AppContext},
        conf::Conf,
        display::{self, Screen},
        errors::{ProgramError, TreeBuildError},
        launchable::Launchable,
        shell_install::{ShellInstall, ShellInstallState},
        tree::TreeOptions,
        verb::VerbStore,
    },
    clap::{self, ArgMatches},
    crossterm::{
        self, cursor,
        event::{DisableMouseCapture, EnableMouseCapture},
        terminal::{EnterAlternateScreen, LeaveAlternateScreen},
        QueueableCommand,
    },
    std::{
        env,
        io::{self, Write},
        path::{Path, PathBuf},
    },
};

/// launch arguments related to installation
/// (not used by the application after the first step)
struct InstallLaunchArgs {
    install: bool,                                // installation is required
    set_install_state: Option<ShellInstallState>, // the state to set
    print_shell_function: Option<String>,         // shell function to print on stdout
}
impl InstallLaunchArgs {
    fn from(cli_args: &ArgMatches<'_>) -> Result<Self, ProgramError> {
        let install = cli_args.is_present("install");
        let print_shell_function = cli_args
            .value_of("print-shell-function")
            .map(str::to_string);
        let set_install_state = cli_args
            .value_of("set-install-state")
            .map(ShellInstallState::from_str)
            .transpose()?;
        Ok(Self {
            install,
            set_install_state,
            print_shell_function,
        })
    }
}

/// the parsed program launch arguments which are kept for the
/// life of the program
pub struct AppLaunchArgs {
    pub root: PathBuf,                    // what should be the initial root
    pub file_export_path: Option<String>, // where to write the produced path (if required with --out)
    pub cmd_export_path: Option<String>, // where to write the produced command (if required with --outcmd)
    pub tree_options: TreeOptions,       // initial tree options
    pub commands: Option<String>,        // commands passed as cli argument, still unparsed
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

fn get_root_path(cli_args: &ArgMatches<'_>) -> Result<PathBuf, ProgramError> {
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
    Ok(canonicalize_root(&root)?)
}

/// run the application, and maybe return a launchable
/// which must be run after broot
pub fn run() -> Result<Option<Launchable>, ProgramError> {
    let clap_app = crate::clap::clap_app();

    // parse the launch arguments we got from cli
    let cli_matches = clap_app.get_matches();

    // read the install related arguments
    let install_args = InstallLaunchArgs::from(&cli_matches)?;

    // execute installation things required by launch args
    let mut must_quit = false;
    if let Some(state) = install_args.set_install_state {
        state.write_file()?;
        must_quit = true;
    }
    if let Some(shell) = &install_args.print_shell_function {
        ShellInstall::print(shell)?;
        must_quit = true;
    }
    if must_quit {
        return Ok(None);
    }

    // read the list of specific config files
    let specific_conf: Option<Vec<PathBuf>> = cli_matches
        .value_of("conf")
        .map(|s| s.split(';').map(PathBuf::from).collect());

    // if we don't run on a specific config file, we check the
    // configuration
    if specific_conf.is_none() {
        let mut shell_install = ShellInstall::new(install_args.install);
        shell_install.check()?;
        if shell_install.should_quit {
            return Ok(None);
        }
    }

    // read the configuration file(s): either the standard one
    // or the ones required by the launch args
    let config = match &specific_conf {
        Some(conf_paths) => {
            let mut conf = Conf::default();
            for path in conf_paths {
                conf.read_file(path)?;
            }
            conf
        }
        _ => Conf::from_default_location()?,
    };

    // tree options are built from the default_flags
    // found in the config file(s) (if any) then overriden
    // by the cli args
    let mut tree_options = TreeOptions::default();
    if !config.default_flags.is_empty() {
        let clap_app = crate::clap::clap_app().setting(clap::AppSettings::NoBinaryName);
        let flags_args = format!("-{}", &config.default_flags);
        let conf_matches = clap_app.get_matches_from(vec![&flags_args]);
        tree_options.apply(&conf_matches);
    }
    tree_options.apply(&cli_matches);

    // verb store is completed from the config file(s)
    let mut verb_store = VerbStore::new();
    verb_store.init(&config);

    // reading the other arguments
    let file_export_path = cli_matches.value_of("file-export-path").map(str::to_string);
    let cmd_export_path = cli_matches.value_of("cmd-export-path").map(str::to_string);
    let commands = cli_matches.value_of("commands").map(str::to_string);
    let no_style = cli_matches.is_present("no-style");
    let height = cli_matches.value_of("height").and_then(|s| s.parse().ok());

    let root = get_root_path(&cli_matches)?;

    let launch_args = AppLaunchArgs {
        root,
        file_export_path,
        cmd_export_path,
        tree_options,
        commands,
        height,
        no_style,
    };

    let context = AppContext::from(launch_args, verb_store);
    let mut w = display::writer();
    let mut screen = Screen::new(&context, &config)?;
    let app = App::new(&context, &screen)?;
    w.queue(EnterAlternateScreen)?;
    w.queue(cursor::Hide)?;
    w.queue(EnableMouseCapture)?;
    let r = app.run(&mut w, &mut screen, &context, &config);
    w.queue(DisableMouseCapture)?;
    w.queue(cursor::Show)?;
    w.queue(LeaveAlternateScreen)?;
    w.flush()?;
    r
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
