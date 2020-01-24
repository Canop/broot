#[macro_use]
extern crate log;

use {
    broot::{
        app::App, app_context::AppContext, cli, conf::Conf, errors::ProgramError,
        external::Launchable, io, shell_install::ShellInstall, skin, verb_store::VerbStore,
    },
    log::LevelFilter,
    simplelog,
    std::{env, fs::File, str::FromStr},
};

/// configure the application log according to env variable.
///
/// There's no log unless the BROOT_LOG environment variable is set to
///  a valid log level (trace, debug, info, warn, error, off)
/// Example:
///      BROOT_LOG=info broot
/// As broot is a terminal application, we only log to a file (dev.log)
fn configure_log() {
    let level = env::var("BROOT_LOG").unwrap_or_else(|_| "off".to_string());
    if level == "off" {
        return;
    }
    if let Ok(level) = LevelFilter::from_str(&level) {
        simplelog::WriteLogger::init(
            level,
            simplelog::Config::default(),
            File::create("dev.log").expect("Log file can't be created"),
        )
        .expect("log initialization failed");
        info!(
            "Starting Broot v{} with log level {}",
            env!("CARGO_PKG_VERSION"),
            level
        );
    }
}

/// run the application, and maybe return a launchable
/// which must be run after broot
fn run() -> Result<Option<Launchable>, ProgramError> {
    configure_log();
    let launch_args = cli::read_launch_args()?;
    let mut must_quit = false;
    if let Some(state) = launch_args.set_install_state {
        state.write_file()?;
        must_quit = true;
    }
    if let Some(shell) = &launch_args.print_shell_function {
        ShellInstall::print(shell)?;
        must_quit = true;
    }
    if must_quit {
        return Ok(None);
    }
    let mut verb_store = VerbStore::new();
    let config = match &launch_args.specific_conf {
        Some(conf_paths) => {
            let mut conf = Conf::default();
            for path in conf_paths {
                conf.read_file(path)?;
            }
            conf
        }
        _ => {
            let mut shell_install = ShellInstall::new(&launch_args);
            shell_install.check()?;
            if shell_install.should_quit {
                return Ok(None);
            }
            Conf::from_default_location()?
        }
    };
    verb_store.init(&config);
    let context = AppContext::from(launch_args, verb_store);
    let skin = skin::Skin::create(config.skin);
    App::new().run(io::writer(), &context, skin)
}

fn main() {
    match run() {
        Ok(Some(launchable)) => {
            if let Err(e) = launchable.execute() {
                warn!("Failed to launch {:?}", &launchable);
                warn!("Error: {:?}", e);
                eprintln!("{}", e);
            }
        }
        Ok(None) => {}
        Err(e) => {
            // this usually happens when the passed path isn't of a directory
            warn!("Error: {}", e);
            eprintln!("{}", e);
        }
    };
    info!("bye");
}
