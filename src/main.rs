#[macro_use]
extern crate log;

use {
    std::{
        env,
        fs::File,
        str::FromStr,
    },
    log::LevelFilter,
    simplelog,
    broot::{
        app::App,
        app_context::AppContext,
        cli,
        conf::Conf,
        errors::ProgramError,
        external::Launchable,
        io,
        shell_install::ShellInstall,
        skin,
        verb_store::VerbStore,
    },
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
    let mut shell_install = ShellInstall::new(&launch_args);
    shell_install.check()?;
    if shell_install.should_quit {
        return Ok(None);
    }
    let mut verb_store = VerbStore::new();
    let config = Conf::from_default_location()?;
    verb_store.init(&config);
    let context = AppContext::from(launch_args, verb_store);
    let skin = skin::Skin::create(config.skin);
    App::new().run(&mut io::writer(), &context, skin)
}

fn main() {
    let res = match run() {
        Ok(res) => res,
        Err(e) => {
            // this usually happens when the passed path isn't of a directory
            warn!("Error: {}", e);
            eprintln!("{}", e);
            return;
        }
    };
    if let Some(launchable) = res {
        if let Err(e) = launchable.execute() {
            warn!("Failed to launch {:?}", &launchable);
            warn!("Error: {:?}", e);
            eprintln!("{}", e);
        }
    }
    info!("bye");
}
