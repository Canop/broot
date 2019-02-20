use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use regex::{Regex, NoExpand};

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
    pub fn execute(&self) -> io::Result<()> {
        if self.just_print {
            print!("{}", &self.exe);
            for arg in &self.args {
                print!(" {}", &arg);
            }
            println!();
        } else {
            Command::new(&self.exe)
                .args(self.args.iter())
                .spawn()?
                .wait()?;
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
        static ref SIMPLE_PATH: Regex = Regex::new(r"^[\w/.]+$").unwrap();
        static ref REPLACER: Regex = Regex::new(r"'").unwrap();
    }
    let path = path.to_string_lossy();
    if SIMPLE_PATH.is_match(&path) {
        path.to_string()
    } else {
        format!("'{}'", REPLACER.replace_all(&path, NoExpand(r"'\''")))
    }
}
