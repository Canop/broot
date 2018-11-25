use std::process::Command;
use std::path::{PathBuf};


pub fn execute(exec: &str) {
    let mut tokens = exec.split_whitespace();
    match tokens.next() {
        Some(exe)   => {
            Command::new(exe).args(tokens).spawn().expect("failed to start external exectutable");
        },
        None        => {
            // FIXME panic?
        },
    }
}

pub fn open_file(path: &PathBuf) {
    Command::new("xdg-open")
        .arg(String::from(path.to_string_lossy()))
        .spawn()
        .expect("xdg-open failed to start");
}

