
use std::{io};
use std::fs::{self};
use std::path::{PathBuf};

#[derive(Debug)]
pub enum LineType {
    File(String), // name
    Dir(String), // name
    Pruning(usize), // unlisted files
}

#[derive(Debug)]
pub struct TreeLine {
    pub left_branchs: Vec<bool>, // len: depth (possible to use an array ? boxed ?)
    pub depth: u16,
    pub key: String,
    pub path: PathBuf,
    pub content: LineType,
}

#[derive(Debug)]
pub struct Tree {
    pub lines: Vec<TreeLine>,
    pub selection: usize, // there's always a selection (starts with root)
}

fn index_to_char(i: usize) -> char {
    match i {
        1...26  => (96 + i as u8) as char,
        27...35 => (48 - 26 + i as u8) as char,
        _       => ' ', // we'll avoid this case
    }
}

impl TreeLine {
    pub fn create(path: PathBuf, depth: u16) -> io::Result<TreeLine> {
        let left_branchs = vec![false; depth as usize];
        let name = match path.file_name() {
            Some(s) => s.to_string_lossy().into_owned(),
            None => String::from("???"),
        };
        let key = String::from("");
        let metadata = fs::metadata(&path)?;
        let content = match metadata.is_dir() {
            true    => LineType::Dir(name),
            false   => LineType::File(name),
        };
        Ok(TreeLine { left_branchs, key, path, depth, content })
    }
    pub fn is_dir(&self) -> bool {
        match &self.content {
            LineType::Dir(_)    => true,
            _                   => false,
        }
    }
    pub fn fill_key(&mut self, v: &Vec<usize>, depth: usize) {
        for i in 0..depth {
            self.key.push(index_to_char(v[i+1]));
        }
    }
}


impl Tree {
    pub fn index_of(&self, path: &PathBuf) -> Option<usize> {
        for i in 0..self.lines.len() {
            if path == &self.lines[i].path {
                return Some(i);
            }
        }
        None
    }
    pub fn has_branch(&self, line_index: usize, depth: usize) -> bool {
        if line_index >= self.lines.len() {
            return false;
        }
        let line = &self.lines[line_index];
        if depth >= line.depth as usize {
            return false;
        }
        return line.left_branchs[depth];
    }
    // if a line matches the key, it is selected and true is returned
    // if none matches, return false and changes nothing
    pub fn try_select(&mut self, key: &str) -> bool {
        for i in 0..self.lines.len() {
            if key == self.lines[i].key {
                self.selection = i;
                return true;
            }
        }
        return false;
    }
}

