use std::fs;
use std::io;
use std::path::PathBuf;

use flat_tree::{LineType, Tree, TreeLine};
use task_sync::TaskLifetime;
use tree_options::TreeOptions;

const SEARCH_DEPTH: usize = 2;

// a child iterator makes it possible to iter over sorted childs
//  (a standard ReadDir is unsorted). It also keeps a "pointer" over
//  the last generated line in the parent tree builder
struct ChildIterator {
    sorted_childs: Option<Vec<PathBuf>>,
    index_next_child: usize, // index for iteration
    index_last_line: usize,  // 0 if none, index of line in tree if any
}
impl ChildIterator {
    fn from(line: &TreeLine, options: &TreeOptions, task_lifetime: &TaskLifetime) -> ChildIterator {
        let sorted_childs = match line.is_dir() {
            true => {
                let mut paths: Vec<PathBuf> = Vec::new();
                match fs::read_dir(&line.path) {
                    Ok(entries) => {
                        for e in entries {
                            if task_lifetime.is_expired() {
                                info!("task expired (child iterator)");
                                return ChildIterator {
                                    sorted_childs: None,
                                    index_next_child: 0,
                                    index_last_line: 0,
                                };
                            }
                            match e {
                                Ok(e) => {
                                    let path = e.path();
                                    if options.accepts(&path, SEARCH_DEPTH, task_lifetime) {
                                        paths.push(e.path());
                                    }
                                }
                                Err(err) => {
                                    debug!("Error while listing {:?} : {:?}", &line.path, err);
                                    // TODO store the error and display it next to the dir
                                }
                            }
                        }
                    }
                    Err(err) => {
                        debug!("Error while listing {:?} : {:?}", &line.path, err);
                        // TODO store the error and display it next to the dir
                    }
                }
                paths.sort();
                Some(paths)
            }
            false => None,
        };
        ChildIterator {
            sorted_childs,
            index_next_child: 0,
            index_last_line: 0,
        }
    }
    fn next_child(&mut self) -> Option<PathBuf> {
        match &self.sorted_childs {
            Some(v) => match self.index_next_child < v.len() {
                true => {
                    let next_child = &v[self.index_next_child];
                    self.index_next_child += 1;
                    Some(next_child.to_path_buf())
                }
                false => Option::None,
            },
            None => Option::None,
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
    options: TreeOptions,
    task_lifetime: TaskLifetime,
}
impl TreeBuilder {
    pub fn from(
        path: PathBuf,
        options: TreeOptions,
        task_lifetime: TaskLifetime,
    ) -> io::Result<TreeBuilder> {
        let path = path.canonicalize()?;
        let mut builder = TreeBuilder {
            lines: Vec::new(),
            child_iterators: Vec::new(),
            options,
            task_lifetime,
        };
        builder.push(path, 0);
        Ok(builder)
    }
    fn push(&mut self, path: PathBuf, depth: u16) {
        let line = TreeLine::create(path, depth);
        let iterator = ChildIterator::from(&line, &self.options, &self.task_lifetime);
        self.lines.push(line);
        self.child_iterators.push(iterator);
    }
    pub fn build(mut self, nb_lines_max: u16) -> Option<Tree> {
        if self.task_lifetime.is_expired() {
            info!("task expired (core build)");
            return None;
        }
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
                if self.task_lifetime.is_expired() {
                    info!("task expired (core build)");
                    return None;
                }
                if let Some(child) = self.child_iterators[i].next_child() {
                    has_open_dirs = true;
                    max_depth = current_depth + 1;
                    self.child_iterators[i].index_last_line = self.lines.len();
                    self.push(child, max_depth);
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
            let count = self.child_iterators[i].nb_unlisted();
            if index == 0 {
                if let LineType::Dir {
                    name: _,
                    ref mut unlisted,
                } = self.lines[i].content
                {
                    *unlisted = count;
                }
            } else if count > 0 {
                self.lines[index].content = LineType::Pruning {
                    unlisted: count + 1,
                };
            }
        }

        // second step: we sort the lines
        self.lines.sort_by(|a, b| a.path.cmp(&b.path));

        // we can now give every file and directory a key
        let mut d: usize = 0;
        let mut counts: Vec<usize> = vec![0; 1]; // first cell not used
        for i in 1..self.lines.len() {
            let line_depth = self.lines[i].depth as usize;
            if line_depth > d {
                if counts.len() <= line_depth {
                    counts.push(0);
                } else {
                    counts[line_depth] = 0;
                }
            }
            d = line_depth;
            counts[d] += 1;
            self.lines[i].fill_key(&counts, d);
        }

        // then we discover the branches (for the drawing)
        for end_index in 1..self.lines.len() {
            let depth = (self.lines[end_index].depth - 1) as usize;
            let start_index = {
                let parent_path = &self.lines[end_index].path.parent();
                let start_index = match parent_path {
                    Some(parent_path) => {
                        let parent_path = parent_path.to_path_buf();
                        let mut index = end_index;
                        loop {
                            if self.lines[index].path == parent_path {
                                break;
                            }
                            if index == 0 {
                                break;
                            }
                            index -= 1;
                        }
                        index
                    }
                    None => end_index, // Should not happen
                };
                start_index + 1
            };
            for i in start_index..end_index + 1 {
                self.lines[i].left_branchs[depth] = true;
            }
        }

        Some(Tree {
            lines: self.lines.into_boxed_slice(),
            selection: 0,
        })
    }
}
