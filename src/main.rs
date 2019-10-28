#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_regex;

mod app;
mod app_context;
mod browser_states;
mod browser_verbs;
mod cli;
mod command_parsing;
mod commands;
mod conf;
mod displayable_tree;
mod elision;
mod errors;
mod external;
mod file_sizes;
mod flat_tree;
mod fuzzy_patterns;
mod git_ignore;
mod help_content;
mod help_states;
mod help_verbs;
mod patterns;
mod permissions;
mod regex_patterns;
mod screens;
mod shell_bash;
mod shell_fish;
mod shell_install;
mod skin;
mod skin_conf;
mod spinner;
mod status;
mod task_sync;
mod tree_build;
mod tree_options;
mod verb_conf;
mod verb_invocation;
mod verb_store;
mod verbs;

use std::{env, fs::File, str::FromStr};

use log::LevelFilter;
use simplelog;

use crate::{
    app::App, app_context::AppContext, conf::Conf, errors::ProgramError, external::Launchable,
    verb_store::VerbStore,
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
    let should_quit = shell_install::init(&launch_args)?;
    if should_quit {
        return Ok(None);
    }
    let mut verb_store = VerbStore::new();
    let config = Conf::from_default_location()?;
    verb_store.init(&config);
    let context = AppContext {
        launch_args,
        verb_store,
    };
    let skin = skin::Skin::create(config.skin);
    App::new().run(&mut std::io::stderr(), &context, skin)
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
