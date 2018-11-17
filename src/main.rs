extern crate termion;

mod app;
mod status;
mod input;
mod flat_tree;
mod tree_views;

use app::App;
use std::env;
use std::io::{self, Write, stdout, stdin};
use flat_tree::{TreeBuilder, Tree};


fn run() -> io::Result<()> {
    let tree = TreeBuilder::from(env::current_dir()?)?.build(10)?;
    println!("{:?}", tree);
    Ok(())
}

fn main() {
    //run().unwrap();
    let app = App::new().unwrap();
    app.run().unwrap();
}
