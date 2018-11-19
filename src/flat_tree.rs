
use std::{io, env};
use std::iter::Iterator;
use std::fs::{self, ReadDir};
use std::path::{Path, PathBuf};

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
    pub path: PathBuf,
    pub content: LineType,
}

#[derive(Debug)]
pub struct Tree {
    pub lines: Vec<TreeLine>,
}

impl TreeLine {
    fn create(path: PathBuf, depth: u16) -> io::Result<TreeLine> {
        let left_branchs = vec![false; depth as usize];
        let name = match path.file_name() {
            Some(s) => s.to_string_lossy().into_owned(),
            None => String::from("???"),
        };
        let metadata = fs::metadata(&path)?;
        let content = match metadata.is_dir() {
            true    => LineType::Dir(name),
            false   => LineType::File(name),
        };
        Ok(TreeLine { left_branchs, path, depth, content })
    }
    pub fn is_dir(&self) -> bool {
        match &self.content {
            LineType::Dir(_)    => true,
            _                   => false,
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
}

// a child iterator makes it possible to iter over sorted childs
//  (a standard ReadDir is unsorted). It also keeps a "pointer" over
//  the last generated line in the parent tree builder
struct ChildIterator {
    sorted_childs: Option<Vec<PathBuf>>,
    index_next_child: usize, // index for iteration
    index_last_line: usize, // 0 if none, index of line in tree if any
}
impl ChildIterator {
    fn from(line: &TreeLine) -> io::Result<ChildIterator> {
        let sorted_childs = match line.is_dir() {
            true    => {
                //let paths = fs::read_dir(&line.path)?.map(|e| e?.path()).collect();
                let mut paths: Vec<PathBuf> = fs::read_dir(&line.path)?.map(|e| e.unwrap().path()).collect();
                paths.sort();
                Some(paths)
            },
            false   => None,
        };
        Ok(ChildIterator {
            sorted_childs,
            index_next_child: 0,
            index_last_line: 0,
        })
    }
    fn next_child(&mut self) -> Option<PathBuf> {
        match &self.sorted_childs {
            Some(v) => match self.index_next_child<v.len() {
                true    => {
                    let next_child = &v[self.index_next_child];
                    self.index_next_child += 1;
                    Some(next_child.to_path_buf())
                },
                false   => Option::None,
            },
            None => Option::None
        }
    }
    fn nb_unlisted(&self) -> usize {
        match &self.sorted_childs {
            Some(v) => v.len() - self.index_next_child,
            None => 0,
        }
    }
}

pub struct TreeBuilder {
    lines: Vec<TreeLine>,
    child_iterators: Vec<ChildIterator>,
}
impl TreeBuilder {
    pub fn from(path: PathBuf) -> io::Result<TreeBuilder> {
        let mut builder = TreeBuilder {
            lines: Vec::new(),
            child_iterators: Vec::new(),
        };
        builder.push(path, 0);
        Ok(builder)
    }
    fn push(&mut self, path: PathBuf, depth: u16) -> io::Result<()> {
        let line = TreeLine::create(path, depth)?;
        let iterator = ChildIterator::from(&line)?;
        self.lines.push(line);
        self.child_iterators.push(iterator);
        Ok(())
    }
    pub fn build(mut self, nb_lines_max: u16) -> io::Result<Tree> {
        // first step: we grow the lines, not exceding nb_lines_max
        let nb_lines_max = nb_lines_max as usize;
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
                if let Some(child) = self.child_iterators[i].next_child() {
                    has_open_dirs = true;
                    max_depth = current_depth + 1;
                    self.child_iterators[i].index_last_line = self.lines.len();
                    self.push(child, max_depth)?;
                }
                if self.lines.len() >= nb_lines_max {
                    break;
                }
            }
            if !has_open_dirs {
                if max_depth > current_depth {
                    current_depth += 1;
                } else {
                    break;
                }
            }
        }

        // we replace the last childs by Pruning marks if there are
        //  some unlisted files behind
        for i in 0..self.lines.len() {
            let index = self.child_iterators[i].index_last_line;
            if index == 0 {
                continue;
            }
            let count = self.child_iterators[i].nb_unlisted();
            if count == 0 {
                continue;
            }
            self.lines[index].content = LineType::Pruning(count+1);
        }

        // second step: we sort the lines
        self.lines.sort_by(|a,b| a.path.cmp(&b.path));

        // then we discover the branches (for the drawing)
        for end_index in 1..self.lines.len() {
            let depth = (self.lines[end_index].depth - 1) as usize;
            let start_index = {
                let parent_path = &self.lines[end_index].path.parent();
                let start_index = match parent_path {
                    Some(parent_path)   => {
                        let parent_path = parent_path.to_path_buf();
                        let mut index = end_index;
                        while index >= 0 {
                            if self.lines[index].path == parent_path {
                                break;
                            }
                            index -= 1;
                        }
                        index
                    },
                    None    => end_index, // Should not happen
                };
                start_index + 1
            };
            for i in start_index..end_index+1 {
                self.lines[i].left_branchs[depth] = true;
            }
        }

        let tree = Tree{
            lines: self.lines,
        };

        Ok(tree)
    }
}

