#[cfg(unix)]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

mod app;
mod app_context;
mod browser_states;
mod browser_verbs;
mod cli;
mod commands;
mod command_parsing;
mod conf;
mod displayable_tree;
mod errors;
mod external;
mod file_sizes;
mod flat_tree;
mod fuzzy_patterns;
mod git_ignore;
mod help_content;
mod help_states;
mod help_verbs;
mod input;
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
mod verbs;
mod verb_invocation;
mod verb_store;

use log::LevelFilter;
use simplelog;
use std::env;
use std::fs::File;
use std::result::Result;
use std::str::FromStr;

use crate::app::App;
use crate::app_context::AppContext;
use crate::conf::Conf;
use crate::errors::ProgramError;
use crate::external::Launchable;
use crate::verb_store::VerbStore;

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
            "Starting B-Root v{} with log level {}",
            env!("CARGO_PKG_VERSION"),
            level
        );
    }
}

/// run the application, and maybe return a launchable
/// which must be run after broot
fn run() -> Result<Option<Launchable>, ProgramError> {
    configure_log();
    let launch_args = cli::read_lauch_args()?;
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
    App::new().run(&context, skin)
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
        info!("launching {:?}", &launchable);
        if let Err(e) = launchable.execute() {
            warn!("Failed to launch {:?}", &launchable);
            warn!("Error: {:?}", e);
            eprintln!("{}", e);
        }
    }
    info!("bye");
}
