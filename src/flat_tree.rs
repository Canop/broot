//! in the flat_tree structure, every "node" is just a line, there's
//!  no link from a child to its parent or from a parent to its childs.
//! It looks stupid and probably is but makes it easier to deal
//!  with the borrow checker.
//! Tree lines can be designated either by their index (from 0 to the
//!  tree's root to the number of lines of the screen) or by their "key",
//!  a string reproducing the hierarchy of the tree.

use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum LineType {
    File { name: String },
    Dir { name: String, unlisted: usize },
    Pruning { unlisted: usize },
}

#[derive(Debug)]
pub struct TreeLine {
    pub left_branchs: Box<[bool]>,
    pub depth: u16,
    pub key: String,
    pub path: PathBuf,
    pub content: LineType,
}

#[derive(Debug)]
pub struct Tree {
    pub lines: Box<[TreeLine]>,
    pub selection: usize, // there's always a selection (starts with root)
}

fn index_to_char(i: usize) -> char {
    match i {
        1...26 => (96 + i as u8) as char,
        27...36 => (47 - 26 + i as u8) as char,
        37...60 => (64 - 36 + i as u8) as char,
        _ => ' ', // we'll avoid this case
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
            true => LineType::Dir { name, unlisted: 0 },
            false => LineType::File { name },
        };
        Ok(TreeLine {
            left_branchs: left_branchs.into_boxed_slice(),
            key,
            path,
            depth,
            content,
        })
    }
    pub fn is_dir(&self) -> bool {
        match &self.content {
            LineType::Dir {
                name: _,
                unlisted: _,
            } => true,
            _ => false,
        }
    }
    pub fn fill_key(&mut self, v: &Vec<usize>, depth: usize) {
        for i in 0..depth {
            self.key.push(index_to_char(v[i + 1]));
        }
    }
}

impl Tree {
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
    pub fn move_selection(&mut self, dy: i16) {
        loop {
            let l = self.lines.len();
            self.selection = (self.selection + (l as i16 + dy) as usize) % l;
            match &self.lines[self.selection].content {
                LineType::Dir {
                    name: _,
                    unlisted: _,
                } => {
                    break;
                }
                LineType::File { name: _ } => {
                    break;
                }
                _ => {}
            }
        }
    }
    pub fn key(&self) -> String {
        self.lines[self.selection].key.to_owned()
    }
    pub fn root(&self) -> &PathBuf {
        &self.lines[0].path
    }
}
