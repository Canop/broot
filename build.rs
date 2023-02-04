// This file is executed during broot compilation.
// It builds shell completion scripts.

use {
    clap::CommandFactory,
    clap_complete::{Generator, Shell},
    std::{
        env,
        ffi::OsStr,
    },
};

include!("src/cli/args.rs");

fn write_completions_file<G: Generator + Copy, P: AsRef<OsStr>>(generator: G, out_dir: P) {
    let mut args = Args::command();
    for name in &["broot", "br"] {
        clap_complete::generate_to(
            generator,
            &mut args,
            name.to_string(),
            &out_dir,
        ).expect("clap complete generation failed");
    }
}

/// write the shell completion scripts which will be added to
/// the release archive
fn build_completion_scripts() {
    let out_dir = env::var_os("OUT_DIR").expect("out dir not set");
    write_completions_file(Shell::Bash, &out_dir);
    write_completions_file(Shell::Elvish, &out_dir);
    write_completions_file(Shell::Fish, &out_dir);
    write_completions_file(Shell::PowerShell, &out_dir);
    write_completions_file(Shell::Zsh, &out_dir);
    eprintln!("completion scripts generated in {out_dir:?}");
}

fn main() {
    build_completion_scripts();
}
