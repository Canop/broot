use std::io;
use std::path::PathBuf;
use std::process::Command;

// description of a possible launch of an external program
// (might be more complex, and a sequence of things to try, in the future)
#[derive(Debug)]
pub struct Launchable {
    exe: String,
    args: Vec<String>,
    pub just_print: bool, // this part of the API will change
}

impl Launchable {
    pub fn opener(path: &PathBuf) -> io::Result<Launchable> {
        Launchable::from(&format!("xdg-open {}", &path.to_string_lossy()))
    }
    pub fn from(launch_string: &str) -> io::Result<Launchable> {
        let mut tokens = launch_string.split_whitespace().map(|t| t.to_string());
        match tokens.next() {
            Some(exe) => Ok(Launchable {
                exe: exe,
                args: tokens.collect(),
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
