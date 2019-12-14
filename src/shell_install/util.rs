
use std::{
    fs::self,
    io::{self, BufRead, BufReader},
    path::Path,
};

pub fn file_contains_line(path: &Path, searched_line: &str) -> io::Result<bool> {
    for line in BufReader::new(fs::File::open(path)?).lines() {
        if line? == searched_line {
            return Ok(true);
        }
    }
    Ok(false)
}

