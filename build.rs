// This file is executed during broot compilation.
// It builds shell completion scripts.

use {
    clap::Shell,
    std::{
        env,
        fs,
        str::FromStr,
        path::Path,
    },
};

include!("src/clap.rs");
include!("src/conf/default_conf.rs");

/// write the shell completion scripts which will be added to
/// the release archive
fn build_completion_scripts() {
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

/// write the default configuration file, which will be added to
/// the release archive
fn build_default_conf() {
    let out_dir = env::var_os("OUT_DIR").expect("out dir not set");
    let file_path = Path::new(&out_dir).join("default-conf.hjson");
    fs::write(&file_path, DEFAULT_CONF_FILE).expect("it to work :'(");
    println!("default conf written in {:?}", file_path);
}

fn main() {
    build_completion_scripts();
    build_default_conf();
}
