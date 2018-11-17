
use std::{io, env};
use std::fs::{self, ReadDir};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum LineType {
    File,
    Dir,
}

#[derive(Debug)]
pub struct TreeLine {
    pub name: String,
    pub depth: u16,
    pub path: PathBuf,
    pub content: LineType,
}

// FIXME Copy ?

impl TreeLine {
    fn create(path: PathBuf, depth: u16) -> io::Result<TreeLine> {
        let name = match path.file_name() {
            Some(s) => s.to_string_lossy().into_owned(),
            None => String::from("???"),
        };
        let metadata = fs::metadata(&path)?;
        let content = match metadata.is_dir() {
            true    => LineType::Dir,
            false   => LineType::File,
        };
        Ok(TreeLine { name, path, depth, content })
    }
    pub fn is_dir(&self) -> bool {
        match &self.content {
            LineType::Dir   => true,
            _               => false,
        }
    }

    pub fn read_dir(&self) -> io::Result<Option<ReadDir>> {
        Ok(match &self.is_dir() {
            true     => Some(fs::read_dir(&self.path)?),
            false    => None,
        })
    }
}

#[derive(Debug)]
pub struct Tree {
    pub lines: Vec<TreeLine>,
}

pub struct TreeBuilder {
    lines: Vec<TreeLine>,
    read_dirs: Vec<Option<ReadDir>>,
}

impl TreeBuilder {
    pub fn from(path: PathBuf) -> io::Result<TreeBuilder> {
        let mut builder = TreeBuilder {
            lines: Vec::new(),
            read_dirs: Vec::new(),
        };
        builder.push(path, 0);
        Ok(builder)
    }
    fn push(&mut self, path: PathBuf, depth: u16) -> io::Result<()> {
        let line = TreeLine::create(path, depth)?;
        let read_dir = line.read_dir()?;
        self.lines.push(line);
        self.read_dirs.push(read_dir);
        Ok(())
    }
    fn next_child(&mut self, i: usize) -> io::Result<Option<PathBuf>> {
        Ok(match self.read_dirs[i] {
            Some(ref mut read_dir) => {
                match read_dir.next() {
                    Some(entry) => Some(entry?.path()),
                    None => Option::None,
                }
            },
            None => Option::None
        })
    }
    pub fn build(mut self, nb_lines_max: u16) -> io::Result<Tree> {
        let nb_lines_max = nb_lines_max as usize;
        println!(" nb_lines_max={:?}", nb_lines_max);
        let mut current_depth = 0;
        let mut max_depth = 0;
        loop {
            let n = self.lines.len();
            if n >= nb_lines_max {
                break;
            }
            let mut has_open_dirs = false;
            for i in 0..n {
                if self.lines[i].depth != current_depth {
                    continue;
                }
                if let Some(child) = self.next_child(i)? {
                    has_open_dirs = true;
                    max_depth = current_depth + 1;
                    self.push(child, max_depth)?;
                }
            }
            if !has_open_dirs {
                if max_depth > current_depth {
                    current_depth = current_depth + 1;
                } else {
                    break;
                }
            }
        }
        Ok(Tree{
            lines: self.lines
        })
    }
}
