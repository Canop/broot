
#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate termion;
extern crate directories;
extern crate toml;
extern crate custom_error;

mod app;
mod commands;
mod conf;
mod external;
mod flat_tree;
mod tree_build;
mod input;
mod status;
mod tree_options;
mod tree_views;
mod verbs;

use custom_error::custom_error;
use std::env;
use std::path::{PathBuf};
use std::io;
use std::result::Result;

use app::App;
use conf::{Conf};
use external::Launchable;
use tree_options::TreeOptions;
use verbs::VerbStore;

const SHOW_APP: bool = true;

custom_error! {ProgramError
    Io{source: io::Error}           = "IO Error",
    Conf{source: conf::ConfError}   = "Bad configuration",
}

fn run(with_gui: bool) -> Result<Option<Launchable>, ProgramError> {
    let config = Conf::from_default_location()?;

    let mut verb_store = VerbStore::new();
    verb_store.fill_from_conf(&config);

    let args: Vec<String> = env::args().collect();
    let path = match args.len() >= 2 {
        true    => PathBuf::from(&args[1]),
        false   => env::current_dir()?,
    };
    Ok(match with_gui {
        true    => {
            let mut app = App::new()?;
            app.push(path, TreeOptions::new())?;
            app.run(&verb_store)?
        },
        false   => {
            None
        },
    })
}

fn main() {
    match run(SHOW_APP).unwrap() {
        Some(launchable)    => {
            launchable.execute().unwrap();
        },
        None                => {},
    }
}
