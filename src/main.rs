#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

mod app;
mod app_context;
mod browser_states;
mod commands;
mod conf;
mod errors;
mod external;
mod file_sizes;
mod flat_tree;
mod git_ignore;
mod help_states;
mod input;
mod patterns;
mod screens;
mod screen_text;
mod spinner;
mod status;
mod task_sync;
mod tree_build;
mod tree_options;
mod tree_views;
mod verbs;

use clap;
use log::LevelFilter;
use simplelog;
use std::env;
use std::fs::File;
use std::path::PathBuf;
use std::result::Result;
use std::str::FromStr;

use crate::app::App;
use crate::app_context::AppContext;
use crate::browser_states::BrowserState;
use crate::commands::Command;
use crate::conf::Conf;
use crate::errors::ProgramError;
use crate::external::Launchable;
use crate::task_sync::TaskLifetime;
use crate::tree_options::TreeOptions;
use crate::verbs::VerbStore;

const VERSION: &str = "0.4.7";

// declare the possible CLI arguments, and gets the values
fn get_cli_args<'a>() -> clap::ArgMatches<'a> {
    clap::App::new("broot")
        .version(VERSION)
        .author("dystroy <denys.seguret@gmail.com>")
        .about("Balanced tree view + fuzzy search + BFS + customizable launcher")
        .arg(
            clap::Arg::with_name("root")
            .help("sets the root directory")
        )
        .arg(
            clap::Arg::with_name("commands")
                .short("c")
                .long("cmd")
                .takes_value(true)
                .help("commands to execute (space separated, experimental)"),
        )
        .arg(
            clap::Arg::with_name("only-folders")
                .short("f")
                .long("only-folders")
                .help("only show folders"),
        )
        .arg(
            clap::Arg::with_name("hidden")
                .short("h")
                .long("hidden")
                .help("show hidden files"),
        )
        .arg(
            clap::Arg::with_name("sizes")
                .short("s")
                .long("sizes")
                .help("show the size of files and directories"),
        )
        .arg(
            clap::Arg::with_name("permissions")
                .short("p")
                .long("permissions")
                .help("show permissions, with owner and group"),
        )
        .arg(
            clap::Arg::with_name("output_path")
                .short("o")
                .long("out")
                .takes_value(true)
                .help("where to write the outputted path (if any)"),
        )
        .arg(
            clap::Arg::with_name("gitignore")
                .short("g")
                .long("gitignore")
                .takes_value(true)
                .help("respect .gitignore rules (yes, no, auto)"),
        )
        .get_matches()
}

// There's no log unless the BROOT_LOG environment variable is set to
//  a valid log level (trace, debug, info, warn, error, off)
// Example:
//      BROOT_LOG=info broot
// As broot is a terminal application, we only log to a file (dev.log)
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
        info!("Starting B-Root v{} / jemalloc with log level {}", VERSION, level);
    }
}

// run the application, and maybe return a launchable
// which must be run after broot
fn run() -> Result<Option<Launchable>, ProgramError> {
    configure_log();

    let config = Conf::from_default_location()?;

    let mut verb_store = VerbStore::new();
    verb_store.init(&config);

    let cli_args = get_cli_args();
    let path = match cli_args.value_of("root") {
        Some(path) => PathBuf::from(path),
        None => env::current_dir()?,
    };
    let path = path.canonicalize()?;
    let mut tree_options = TreeOptions::new();
    if cli_args.is_present("only-folders") {
        debug!("show only folders arg set");
        tree_options.only_folders = true;
    }
    if cli_args.is_present("hidden") {
        debug!("show hidden files arg set");
        tree_options.show_hidden = true;
    }
    if cli_args.is_present("sizes") {
        debug!("show sizes arg set");
        tree_options.show_sizes = true;
    }
    if cli_args.is_present("permissions") {
        debug!("show permissions arg set");
        tree_options.show_permissions = true;
    }
    if let Some(respect_ignore) = cli_args.value_of("gitignore") {
        tree_options.respect_git_ignore = respect_ignore.parse()?;
        debug!("respect_git_itnore = {:?}", tree_options.respect_git_ignore);
    }

    let con = AppContext {
        verb_store,
        output_path: cli_args
            .value_of("output_path")
            .and_then(|s| Some(s.to_owned())),
    };
    debug!("output path: {:?}", &con.output_path);

    let input_commands: Vec<Command> = match cli_args.value_of("commands") {
        Some(str) => str.split(' ').map(|s| Command::from(s.to_string())).collect(),
        None => Vec::new(),
    };

    Ok(
        match BrowserState::new(path.clone(), tree_options, &TaskLifetime::unlimited()) {
            Some(bs) => {
                let mut app = App::new();
                app.push(Box::new(bs));
                app.run(&con, input_commands)?
            }
            _ => None, // should not happen, as the lifetime is "unlimited"
        },
    )
}

fn main() {
    let res = run().unwrap();
    if let Some(launchable) = res {
        info!("launching {:?}", &launchable);
        if let Err(e) = launchable.execute() {
            warn!("Failed to launch {:?}", &launchable);
            warn!("Error: {:?}", e);
            println!("Failed to lauch executable: {:?}", e);
        }
    }
    info!("bye");
}
