// This file is executed during broot compilation.
// It builds shell completion scripts.

use {
    clap::Shell,
    std::{env, str::FromStr},
};

include!("src/clap.rs");

fn main() {
    // out_dir should be defined, see
    //  https://doc.rust-lang.org/cargo/reference/environment-variables.html
    let out_dir = env::var_os("OUT_DIR").expect("out dir not set");
    let mut app = clap_app();
    for variant in &Shell::variants() {
        let variant = Shell::from_str(variant).unwrap();
        app.gen_completions("broot", variant, &out_dir);
        app.gen_completions("br", variant, &out_dir);
    }
    println!("completion scripts generated in {:?}", out_dir);
}
