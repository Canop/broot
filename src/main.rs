
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
mod tree_views;
mod verbs;

use std::env;
use std::path::{PathBuf};
use std::io;
use directories::{ProjectDirs};
use std::{thread, time};
use conf::Conf;

use app::App;
use tree_build::{TreeBuilder};
use verbs::VerbStore;

const SHOW_APP: bool = false;

fn main() -> io::Result<()> {
    if let Some(proj_dirs) = ProjectDirs::from("org", "dystroy",  "btree") {
        let conf_filename = format!("{}/conf.toml", proj_dirs.config_dir().to_string_lossy());
        let _ = Conf::from_file(&conf_filename).unwrap();
    }

    let args: Vec<String> = env::args().collect();
    let path = match args.len() >= 2 {
        true    => PathBuf::from(&args[1]),
        false   => env::current_dir()?,
    };
    if SHOW_APP {
        let mut app = App::new()?;
        let mut verb_store = VerbStore::new();
        verb_store.set_defaults();
        app.push(path)?;
        match app.run(&verb_store)? {
            Some(launchable)    => {
                launchable.execute()?;
            },
            None                => {
            },
        }
    } else {
        let tree = TreeBuilder::from(path)?.build(80)?;
        //println!("{:?}", tree);
    }
    Ok(())
}
