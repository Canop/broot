//! this module manages reading and translating
//! the arguments passed on launch of the application.

pub mod clap_args;
mod app_launch_args;
mod install_launch_args;

pub use {
    app_launch_args::*,
    install_launch_args::*,
};

use {
    crate::{
        app::{App, AppContext},
        conf::Conf,
        display,
        errors::{ProgramError, TreeBuildError},
        launchable::Launchable,
        shell_install::ShellInstall,
        tree::TreeOptions,
        verb::VerbStore,
    },
    clap::{self, ArgMatches},
    crossterm::{
        self,
        cursor,
        event::{DisableMouseCapture, EnableMouseCapture},
        terminal::{EnterAlternateScreen, LeaveAlternateScreen},
        QueueableCommand,
        tty::IsTty,
    },
    std::{
        env,
        io::{self, Write, stdout},
        path::{Path, PathBuf},
    },
};

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
        .value_of("ROOT")
        .map_or(env::current_dir()?, PathBuf::from);
    if !root.exists() {
        return Err(TreeBuildError::FileNotFound {
            path: format!("{:?}", &root),
        }.into());
    }
    if !root.is_dir() {
        // we try to open the parent directory if the passed file isn't one
        if let Some(parent) = root.parent() {
            info!("Passed path isn't a directory => opening parent instead");
            root = parent.to_path_buf();
        } else {
            // let's give up
            return Err(TreeBuildError::NotADirectory {
                path: format!("{:?}", &root),
            }.into());
        }
    }
    Ok(canonicalize_root(&root)?)
}

fn is_output_piped() -> bool {
    !stdout().is_tty()
}

fn is_no_style(cli_matches: &ArgMatches) -> bool {
    if cli_matches.is_present("no-style") {
        return true;
    }
    match cli_matches.value_of("color") {
        Some("yes") => false,
        Some("no") => true,
        _  => is_output_piped(),
    }
}

/// run the application, and maybe return a launchable
/// which must be run after broot
pub fn run() -> Result<Option<Launchable>, ProgramError> {
    let clap_app = clap_args::clap_app();

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
    if specific_conf.is_none() && install_args.install != Some(false) {
        let mut shell_install = ShellInstall::new(install_args.install == Some(true));
        shell_install.check()?;
        if shell_install.should_quit {
            return Ok(None);
        }
    }

    // read the configuration file(s): either the standard one
    // or the ones required by the launch args
    let mut config = match &specific_conf {
        Some(conf_paths) => {
            let mut conf = Conf::default();
            for path in conf_paths {
                conf.read_file(path.to_path_buf())?;
            }
            conf
        }
        _ => time!(Conf::from_default_location())?,
    };
    debug!("config: {:#?}", &config);

    // tree options are built from the default_flags
    // found in the config file(s) (if any) then overriden
    // by the cli args
    let mut tree_options = TreeOptions::default();
    tree_options.apply_config(&config)?;
    tree_options.apply_launch_args(&cli_matches);

    // verb store is completed from the config file(s)
    let mut verb_store = VerbStore::default();
    verb_store.init(&mut config)?;

    // reading the other arguments
    let file_export_path = cli_matches.value_of("file-export-path").map(str::to_string);
    let cmd_export_path = cli_matches.value_of("cmd-export-path").map(str::to_string);
    let commands = cli_matches.value_of("commands").map(str::to_string);
    let no_style = is_no_style(&cli_matches);
    let height = cli_matches.value_of("height").and_then(|s| s.parse().ok());

    let root = get_root_path(&cli_matches)?;

    #[cfg(feature = "client-server")]
    if let Some(server_name) = cli_matches.value_of("send") {
        use crate::{
            command::Sequence,
            net::{Client, Message},
        };
        let client = Client::new(server_name);
        if let Some(seq) = &commands {
            let message = Message::Sequence(Sequence::new_local(seq.to_string()));
            client.send(&message)?;
        } else if !cli_matches.is_present("get-root") {
            let message = Message::Command(format!(":focus {}", root.to_string_lossy()));
            client.send(&message)?;
        };
        if cli_matches.is_present("get-root") {
            client.send(&Message::GetRoot)?;
        }
        return Ok(None);
    }

    let launch_args = AppLaunchArgs {
        root,
        file_export_path,
        cmd_export_path,
        tree_options,
        commands,
        height,
        no_style,

        #[cfg(feature = "client-server")]
        listen: cli_matches.value_of("listen").map(str::to_string),
    };

    let context = AppContext::from(launch_args, verb_store, &config)?;
    let mut w = display::writer();
    let app = App::new(&context)?;
    w.queue(EnterAlternateScreen)?;
    w.queue(cursor::Hide)?;
    let capture_mouse = config.disable_mouse_capture != Some(true);
    if capture_mouse {
        w.queue(EnableMouseCapture)?;
    }
    let r = app.run(&mut w, &context, &config);
    if capture_mouse {
        w.queue(DisableMouseCapture)?;
    }
    w.queue(cursor::Show)?;
    w.queue(LeaveAlternateScreen)?;
    w.flush()?;
    r
}

/// wait for user input, return `true` if they didn't answer 'n'
pub fn ask_authorization() -> Result<bool, ProgramError> {
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    let answer = answer.trim();
    Ok(!matches!(answer, "n" | "N"))
}
