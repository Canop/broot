//#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;
extern crate custom_error;
extern crate directories;
extern crate regex;
extern crate termion;
extern crate toml;
#[macro_use]
extern crate log;
extern crate simplelog;

mod app;
mod browser_states;
mod commands;
mod conf;
mod external;
mod flat_tree;
mod help_states;
mod input;
mod patterns;
mod screens;
mod status;
mod task_sync;
mod tree_build;
mod tree_options;
mod tree_views;
mod verbs;

use custom_error::custom_error;
use log::LevelFilter;
use std::env;
use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::result::Result;
use std::str::FromStr;

use app::App;
use browser_states::BrowserState;
use conf::Conf;
use external::Launchable;
use task_sync::TaskLifetime;
use tree_options::TreeOptions;
use verbs::VerbStore;

custom_error! {ProgramError
    Io{source: io::Error}           = "IO Error",
    Conf{source: conf::ConfError}   = "Bad configuration",
}

// There's no log unless the BROOT_LOG environment variable is set to
//  a valid log level (trace, debug, info, warn, error, off)
// Example:
//      BROOT_LOG=info broot
// As broot is a terminal application, we only log to a file (dev.log)
fn configure_log() {
    let level = env::var("BROOT_LOG").unwrap_or("off".to_string());
    if level == "off" {
        return;
    }
    if let Ok(level) = LevelFilter::from_str(&level) {
        simplelog::WriteLogger::init(
            level,
            simplelog::Config::default(),
            File::create("dev.log").expect("Log file can't be created"),
        ).expect("log initialization failed");
        info!("Starting B-Root with log level {}", level);
    }
}

fn run() -> Result<Option<Launchable>, ProgramError> {
    configure_log();

    let config = Conf::from_default_location()?;

    let mut verb_store = VerbStore::new();
    verb_store.fill_from_conf(&config);

    let args: Vec<String> = env::args().collect();
    debug!("args: {:?}", args);
    let path = match args.len() >= 2 {
        true => PathBuf::from(&args[1]),
        false => env::current_dir()?,
    };
    let path = path.canonicalize()?;

    Ok(
        match BrowserState::new(path.clone(), TreeOptions::new(), TaskLifetime::unlimited()) {
            Some(bs) => {
                let mut app = App::new();
                app.push(Box::new(bs));
                app.run(&verb_store)?
            }
            _ => None, // should not happen
        },
    )
}

fn main() {
    match run().unwrap() {
        Some(launchable) => {
            info!("launching {:?}", &launchable);
            if let Err(e) = launchable.execute() {
                println!("Failed to launch {:?}", &launchable);
                println!("Error: {:?}", e);
                warn!("Failed to launch {:?}", &launchable);
                warn!("Error: {:?}", e);
            }
        }
        None => {}
    }
    //info!("bye");
}
