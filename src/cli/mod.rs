//! this module manages reading and translating
//! the arguments passed on launch of the application.

//mod app_launch_args;
mod args;
mod install_launch_args;

pub use {
    //app_launch_args::*,
    args::*,
    install_launch_args::*,
};

use {
    crate::{
        app::{App, AppContext},
        conf::{Conf, write_default_conf_in},
        display,
        errors::ProgramError,
        launchable::Launchable,
        shell_install::{ShellInstall, write_state},
        verb::VerbStore,
    },
    clap::Parser,
    crossterm::{
        self,
        cursor,
        event::{DisableMouseCapture, EnableMouseCapture},
        terminal::{EnterAlternateScreen, LeaveAlternateScreen},
        QueueableCommand,
    },
    std::{
        io::{self, Write},
        path::PathBuf,
    },
};

/// run the application, and maybe return a launchable
/// which must be run after broot
pub fn run() -> Result<Option<Launchable>, ProgramError> {

    // parse the launch arguments we got from cli
    let args = Args::parse();
    let mut must_quit = false;

    if let Some(dir) = &args.write_default_conf {
        write_default_conf_in(dir)?;
        must_quit = true;
    }

    // read the install related arguments
    let install_args = InstallLaunchArgs::from(&args)?;

    // execute installation things required by launch args
    if let Some(state) = install_args.set_install_state {
        write_state(state)?;
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
    let specific_conf: Option<Vec<PathBuf>> = args.conf
        .as_ref()
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

    // verb store is completed from the config file(s)
    let verb_store = VerbStore::new(&mut config)?;

    let mut context = AppContext::from(args, verb_store, &config)?;

    #[cfg(unix)]
    if let Some(server_name) = &context.launch_args.send {
        use crate::{
            command::Sequence,
            net::{Client, Message},
        };
        let client = Client::new(server_name);
        if let Some(seq) = &context.launch_args.cmd {
            let message = Message::Sequence(Sequence::new_local(seq.to_string()));
            client.send(&message)?;
        } else if !context.launch_args.get_root {
            let message = Message::Command(
                format!(":focus {}", context.initial_root.to_string_lossy())
            );
            client.send(&message)?;
        };
        if context.launch_args.get_root {
            client.send(&Message::GetRoot)?;
        }
        return Ok(None);
    }

    let mut w = display::writer();
    let app = App::new(&context)?;
    w.queue(EnterAlternateScreen)?;
    w.queue(cursor::Hide)?;
    if context.capture_mouse {
        w.queue(EnableMouseCapture)?;
    }
    let r = app.run(&mut w, &mut context, &config);
    if context.capture_mouse {
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
