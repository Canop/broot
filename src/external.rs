use regex::Regex;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::app::AppStateCmdResult;
use crate::app_context::AppContext;
use crate::errors::LaunchError;

/// description of a possible launch of an external program
/// (might be more complex, and a sequence of things to try, in the future).
/// A launchable can only be executed on end of life of broot.
#[derive(Debug)]
pub struct Launchable {
    exe: String,
    args: Vec<String>,
    pub just_print: bool, // this part of the API will change
}

impl Launchable {
    pub fn opener(path: &PathBuf) -> io::Result<Launchable> {
        Launchable::from(vec![
            "xdg-open".to_string(),
            path.to_string_lossy().to_string(),
        ])
    }
    pub fn from(mut parts: Vec<String>) -> io::Result<Launchable> {
        let mut parts = parts.drain(0..);
        match parts.next() {
            Some(exe) => Ok(Launchable {
                exe,
                args: parts.collect(),
                just_print: false,
            }),
            None => Err(io::Error::new(io::ErrorKind::Other, "Empty launch string")),
        }
    }
    pub fn execute(&self) -> Result<(), LaunchError> {
        if self.just_print {
            print!("{}", &self.exe);
            for arg in &self.args {
                print!(" {}", &arg);
            }
            println!();
        } else {
            Command::new(&self.exe)
                .args(self.args.iter())
                .spawn()
                .and_then(|mut p| p.wait())
                .map_err(|source| LaunchError {
                    program: self.exe.clone().into(),
                    source,
                })?;
        }
        Ok(())
    }
}

// from a path, build a string usable in a shell command, wrapping
//  it in quotes if necessary (and then escaping internal quotes).
// Don't do unnecessary transformation, so that the produced string
//  is prettier on screen.
pub fn escape_for_shell(path: &Path) -> String {
    lazy_static! {
        static ref SIMPLE_PATH: Regex = Regex::new(r"^[\w/.-]*$").unwrap();
    }
    let path = path.to_string_lossy();
    if SIMPLE_PATH.is_match(&path) {
        path.to_string()
    } else {
        format!("'{}'", &path.replace('\'', r"'\''"))
    }
}

pub fn print_path(path: &Path, con: &AppContext) -> io::Result<AppStateCmdResult> {
    let path = path.to_string_lossy().to_string();
    Ok(
        if let Some(ref output_path) = con.launch_args.file_export_path {
            // an output path was provided, we write to it
            let f = OpenOptions::new()
                .create(true)
                .append(true)
                .open(output_path)?;
            writeln!(&f, "{}", path)?;
            AppStateCmdResult::Quit
        } else {
            // no output path provided. We write on stdout, but we must
            // do it after app closing to have the normal terminal
            let mut launchable = Launchable::from(vec![path])?;
            launchable.just_print = true;
            AppStateCmdResult::Launch(launchable)
        },
    )
}
