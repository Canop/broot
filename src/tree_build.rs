use std::fs;
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
    ) -> TreeBuilder {
        let mut builder = TreeBuilder {
            lines: Vec::new(),
            child_iterators: Vec::new(),
            options,
            task_lifetime,
        };
        builder.push(path, 0);
        builder
    }
    fn push(&mut self, path: PathBuf, depth: u16) {
        let line = TreeLine::create(path, depth);
        let iterator = ChildIterator::from(&line, &self.options, &self.task_lifetime);
        self.lines.push(line);
        self.child_iterators.push(iterator);
    }
    // build can be called only once per iterator
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
                if self.lines[i].is_dir() {
                    self.lines[i].unlisted = count;
                }
            } else if count > 0 {
                self.lines[index].content = LineType::Pruning;
                self.lines[index].unlisted = count + 1;
            }
        }

        let mut tree = Tree{
            lines: self.lines.into_boxed_slice(),
            selection: 0,
            pattern: self.options.pattern.clone(),
        };
        tree.after_lines_changed();
        Some(tree)
    }
}
