use std::process::Command;
use std::path::{PathBuf};

pub fn open_file(path: &PathBuf) {
    Command::new("xdg-open")
        .arg(String::from(path.to_string_lossy()))
        .spawn()
        .expect("xdg-open failed to start");
}

