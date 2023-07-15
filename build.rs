//! This file is executed during broot compilation.
//! It builds shell completion scripts and the man page
//!
//! Note: to see the eprintln messages, run cargo with
//!     cargo -vv build --release
use {
    clap::CommandFactory,
    clap_complete::{Generator, Shell},
    std::{
        env,
        ffi::OsStr,
    },
};

include!("src/cli/args.rs");

/// The man page built by clap-mangen is too rough to be used as is. It's only
/// used as part of a manual process to update the one in /man/page
/// so this generation is usually not needed
pub const BUILD_MAN_PAGE: bool = false;

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

/// generate the man page from the Clap configuration
fn build_man_page() -> std::io::Result<()> {
    let out_dir = env::var_os("OUT_DIR").expect("out dir not set");
    let out_dir = PathBuf::from(out_dir);
    let cmd = Args::command();
    let man = clap_mangen::Man::new(cmd);
    let mut buffer: Vec<u8> = Default::default();
    man.render(&mut buffer)?;
    let file_path = out_dir.join("broot.1");
    std::fs::write(&file_path, buffer)?;
    eprintln!("map page generated in {file_path:?}");
    Ok(())
}

fn main() -> std::io::Result<()> {
    build_completion_scripts();
    if BUILD_MAN_PAGE {
        build_man_page()?;
    }
    Ok(())
}
