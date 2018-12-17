use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;

use crate::flat_tree::{LineType, Tree, TreeLine};
use crate::patterns::Pattern;
use crate::task_sync::TaskLifetime;
use crate::tree_options::TreeOptions;

// like a tree line, but with the info needed during the build
// This structure isn't usable independantly from the tree builder
#[derive(Debug, Clone)]
struct BLine {
    parent_idx: usize,
    path: PathBuf,
    depth: u16,
    name: String,
    childs_loaded: bool,   // true when load_childs has been called already
    childs: Vec<usize>,    // sorted and filtered (indexes of the childs in tree.blines)
    next_child_idx: usize, // index for iteration, among the childs
    line_type: LineType,
    has_error: bool,
    nb_matches: usize, // can be temporarly 0 for a folder until we check the content
    score: i32,
}
impl BLine {
    // a special constructor, checking nothing
    fn from_root(path: PathBuf) -> BLine {
        let name = match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => String::from("???"), // should not happen
        };
        BLine {
            parent_idx: 0,
            path,
            depth: 0,
            name,
            childs_loaded: false,
            childs: Vec::new(),
            next_child_idx: 0,
            line_type: LineType::Dir, // it should have been checked before
            has_error: false,         // well... let's hope
            nb_matches: 1,
            score: 0,
        }
    }
    // return a bline if the path directly matches the pattern and no_hidden conditions
    fn from(
        parent_idx: usize,
        path: PathBuf,
        depth: u16,
        no_hidden: bool,
        only_folders: bool,
        pattern: &Option<Pattern>,
    ) -> Option<BLine> {
        let mut nb_matches = 1;
        let mut score = 0;
        let name = {
            let name = match &path.file_name() {
                Some(name) => name.to_string_lossy(),
                None => {
                    return None;
                }
            };
            if no_hidden && name.starts_with('.') {
                return None;
            }
            if let Some(pattern) = pattern {
                if let Some(m) = pattern.test(&name) {
                    score = m.score;
                } else {
                    nb_matches = 0;
                }
            }
            name.to_string()
        };
        let mut has_error = false;
        let line_type = match fs::symlink_metadata(&path) {
            Ok(metadata) => {
                let ft = metadata.file_type();
                if ft.is_dir() {
                    LineType::Dir
                } else if ft.is_symlink() {
                    if nb_matches == 0 || only_folders {
                        return None;
                    }
                    LineType::SymLink(match fs::read_link(&path) {
                        Ok(target) => target.to_string_lossy().into_owned(),
                        Err(_) => String::from("???"),
                    })
                } else {
                    if nb_matches == 0 || only_folders {
                        return None;
                    }
                    LineType::File
                }
            }
            Err(_) => {
                if nb_matches == 0 {
                    return None;
                }
                has_error = true;
                LineType::File
            }
        };
        Some(BLine {
            parent_idx,
            path,
            depth,
            name: name.to_string(),
            childs_loaded: false,
            childs: Vec::new(),
            next_child_idx: 0,
            line_type,
            has_error,
            nb_matches,
            score,
        })
    }
    fn to_tree_line(&self) -> TreeLine {
        TreeLine {
            left_branchs: vec![false; self.depth as usize].into_boxed_slice(),
            depth: self.depth,
            name: self.name.to_string(),
            path: self.path.clone(),
            content: self.line_type.clone(),
            has_error: self.has_error,
            unlisted: self.childs.len() - self.next_child_idx,
            score: self.score,
            size: None,
        }
    }
}

pub struct TreeBuilder {
    blines: Vec<BLine>, // all blines, even the ones not yet "seen" by BFS
    options: TreeOptions,
}
impl TreeBuilder {
    pub fn from(path: PathBuf, options: TreeOptions) -> TreeBuilder {
        let mut blines = Vec::new();
        blines.push(BLine::from_root(path));
        TreeBuilder { blines, options }
    }
    // stores (move) the bline in the global vec. Returns its index
    fn store(&mut self, bline: BLine) -> usize {
        let idx = self.blines.len();
        self.blines.push(bline);
        idx
    }
    // returns true when there are direct matches among childs
    fn load_childs(&mut self, bline_idx: usize) -> bool {
        let mut has_child_match = false;
        self.blines[bline_idx].childs_loaded = true;
        match fs::read_dir(&self.blines[bline_idx].path) {
            Ok(entries) => {
                let mut childs: Vec<usize> = Vec::new();
                for e in entries {
                    if let Ok(e) = e {
                        let bl = BLine::from(
                            bline_idx,
                            e.path(),
                            self.blines[bline_idx].depth + 1,
                            !self.options.show_hidden,
                            self.options.only_folders,
                            &self.options.pattern,
                        );
                        if let Some(bl) = bl {
                            if bl.nb_matches > 0 {
                                // direct match
                                self.blines[bline_idx].nb_matches += bl.nb_matches;
                                has_child_match = true;
                            }
                            childs.push(self.store(bl));
                        }
                    }
                }
                childs.sort_by(|&a, &b| {
                    self.blines[a]
                        .name
                        .to_lowercase()
                        .cmp(&self.blines[b].name.to_lowercase())
                });
                self.blines[bline_idx].childs.append(&mut childs);
            }
            Err(err) => {
                debug!(
                    "Error while listing {:?} : {:?}",
                    self.blines[bline_idx].path, err
                );
                self.blines[bline_idx].has_error = true;
            }
        }
        has_child_match
    }
    // load_childs must have been called before on bline_idx
    fn next_child(
        &mut self,
        bline_idx: usize, // the parent
    ) -> Option<usize> {
        let bline = &mut self.blines[bline_idx];
        match bline.next_child_idx < bline.childs.len() {
            true => {
                let next_child = bline.childs[bline.next_child_idx];
                bline.next_child_idx += 1;
                Some(next_child)
            }
            false => Option::None,
        }
    }
    // build can be called only once per builder
    pub fn build(mut self, nb_lines_max: usize, task_lifetime: &TaskLifetime) -> Option<Tree> {
        let mut out_blines: Vec<usize> = Vec::new(); // the blines we want to display
        out_blines.push(0);
        debug!("start building with pattern {:?}", self.options.pattern);
        let mut nb_lines_ok = 1; // in out_blines
        let mut open_dirs: VecDeque<usize> = VecDeque::new();
        let mut next_level_dirs: Vec<usize> = Vec::new();
        self.load_childs(0);
        open_dirs.push_back(0);
        loop {
            if nb_lines_ok >= nb_lines_max {
                break;
            }
            if task_lifetime.is_expired() {
                info!("task expired (core build)");
                return None;
            }
            if let Some(open_dir_idx) = open_dirs.pop_front() {
                if let Some(child_idx) = self.next_child(open_dir_idx) {
                    open_dirs.push_back(open_dir_idx);
                    let child = &self.blines[child_idx];
                    if child.nb_matches > 0 {
                        nb_lines_ok += 1;
                    }
                    if child.line_type == LineType::Dir {
                        next_level_dirs.push(child_idx);
                    }
                    out_blines.push(child_idx);
                }
            } else {
                // this depth is finished, we must go deeper
                if next_level_dirs.is_empty() {
                    break;
                }
                for next_level_dir_idx in &next_level_dirs {
                    let has_child_match = self.load_childs(*next_level_dir_idx);
                    if has_child_match {
                        // we must ensure the ancestors are made Ok
                        let mut idx = *next_level_dir_idx;
                        loop {
                            let mut bline = &mut self.blines[idx];
                            if bline.nb_matches == 0 {
                                bline.nb_matches = 1; // TODO care about the exact count ?
                                nb_lines_ok += 1;
                            }
                            idx = bline.parent_idx;
                            if idx == 0 {
                                break;
                            }
                        }
                        if nb_lines_ok >= nb_lines_max {
                            break;
                        }
                    }
                    open_dirs.push_back(*next_level_dir_idx);
                }
                next_level_dirs.clear();
            }
        }

        if self.options.show_sizes {
            // if the root directory isn't totally read, we finished it even
            // it it goes past the bottom of the screen
            while let Some(child_idx) = self.next_child(0) {
                let child = &self.blines[child_idx];
                if child.nb_matches > 0 {
                    nb_lines_ok += 1;
                }
                out_blines.push(child_idx);
            }
        }

        let mut lines: Vec<TreeLine> = Vec::new();
        for idx in out_blines.iter() {
            if self.blines[*idx].nb_matches > 0 {
                if !self.blines[*idx].childs_loaded {
                    if let LineType::Dir = self.blines[*idx].line_type {
                        self.load_childs(*idx);
                    }
                }
                if !self.options.show_sizes && lines.len() >= nb_lines_max {
                    break; // we can have a little too many lines due to ancestor additions
                }
                lines.push(self.blines[*idx].to_tree_line());
            }
        }

        let mut tree = Tree {
            lines: lines.into_boxed_slice(),
            selection: 0,
            options: self.options.clone(),
            scroll: 0,
        };
        tree.after_lines_changed();

        if self.options.show_sizes {
            tree.fetch_file_sizes();
        }
        debug!(
            "new tree: {} lines, nb_lines_max={}",
            tree.lines.len(),
            nb_lines_max
        );

        Some(tree)
    }
}
