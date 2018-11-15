extern crate termion;

mod app;
mod status;
mod input;
mod nodes;
mod tree_views;

use app::App;

fn main() {
    let app = App::new().unwrap();
    app.run().unwrap();
}
