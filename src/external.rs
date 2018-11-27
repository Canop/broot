use std::io;
use std::process::Command;
use std::path::{PathBuf};

// description of a possible launch of an external program
// (might be more complex, and a sequence of things to try, in the future)
#[derive(Debug)]
pub struct Launchable {
    exe: String,
    args: Vec<String>,
}

impl Launchable {
    pub fn opener(path: &PathBuf) -> io::Result<Launchable> {
        Launchable::from(&format!("xdg-open {}", &path.to_string_lossy()))
    }
    pub fn from(launch_string: &str) -> io::Result<Launchable> {
        let mut tokens = launch_string
            .split_whitespace()
            .map(|t| t.to_string());
        match tokens.next() {
            Some(exe)   => {
                Ok(Launchable{
                    exe: exe,
                    args: tokens.collect()
                })
            },
            None        => {
                Err(io::Error::new(io::ErrorKind::Other, "Invalid launch string")) // can this really happen?
            },
        }
    }
    // execute the external program
    // WARNING: this may kill the current program. Caller must
    // ensure everything's clean before
    pub fn execute(&self) -> io::Result<()> {
        Command::new(&self.exe)
            .args(self.args.iter())
            .spawn()?
            .wait()?;
        Ok(())
    }
}

