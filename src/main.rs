
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate termion;

mod app;
mod commands;
mod flat_tree;
mod tree_build;
mod input;
mod status;
mod tree_views;

use app::App;
use std::env;
use std::path::{PathBuf};
use std::io;
use tree_build::{TreeBuilder};

const SHOW_APP: bool = true;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let path = match args.len() >= 2 {
        true    => PathBuf::from(&args[1]),
        false   => env::current_dir()?,
    };
    if SHOW_APP {
        let mut app = App::new()?;
        app.push(path)?;
        app.run()?;
    } else {
        let tree = TreeBuilder::from(path)?.build(80)?;
        println!("{:?}", tree);
    }
    Ok(())
}
