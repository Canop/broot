#![warn(clippy::all)]

use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::os::unix::fs::MetadataExt;

use crate::flat_tree::{LineType, Tree, TreeLine};
use crate::git_ignore::GitIgnoreFilter;
use crate::patterns::Pattern;
use crate::task_sync::TaskLifetime;
use crate::tree_options::{OptionBool, TreeOptions};

// like a tree line, but with the info needed during the build
// This structure isn't usable independantly from the tree builder
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
    has_match: bool,
    score: i32,
    ignore_filter: Option<GitIgnoreFilter>,
}

// the result of trying to build a bline
enum BLineResult {
    Some(BLine), // the only positive result
    FilteredOutAsHidden,
    FilteredOutByPattern,
    FilteredOutAsNonFolder,
    GitIgnored,
    Invalid,
}

impl BLine {
    // a special constructor, checking nothing
    fn from_root(path: PathBuf, respect_ignore: OptionBool) -> BLine {
        let name = match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => String::from("???"), // should not happen
        };
        let ignore_filter = if respect_ignore == OptionBool::No {
            None
        } else {
            let gif = GitIgnoreFilter::applicable_to(&path);
            // if auto, we don't look for other gif if we're not in a git dir
            if respect_ignore == OptionBool::Auto && gif.files.is_empty() {
                None
            } else {
                Some(gif)
            }
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
            has_match: true,
            score: 0,
            ignore_filter,
        }
    }
    // return a bline if the path directly matches the conditions
    // (pattern, no_hidden, only_folders)
    fn from(
        parent_idx: usize,
        path: PathBuf,
        depth: u16,
        no_hidden: bool,
        only_folders: bool,
        pattern: &Option<Pattern>,
        parent_ignore_filter: &Option<GitIgnoreFilter>,
    ) -> BLineResult {
        let mut has_match = true;
        let mut score = 0;
        let name = {
            let name = match &path.file_name() {
                Some(name) => name.to_string_lossy(),
                None => {
                    return BLineResult::Invalid;
                }
            };
            if no_hidden && name.starts_with('.') {
                return BLineResult::FilteredOutAsHidden;
            }
            if let Some(pattern) = pattern {
                if let Some(m) = pattern.test(&name) {
                    score = m.score;
                } else {
                    has_match = false;
                }
            }
            name.to_string()
        };
        let mut has_error = false;
        let mut is_dir = false;
        let line_type = match fs::symlink_metadata(&path) {
            Ok(metadata) => {
                let ft = metadata.file_type();
                if ft.is_dir() {
                    is_dir = true;
                    LineType::Dir
                } else if ft.is_symlink() {
                    if !has_match {
                        return BLineResult::FilteredOutByPattern;
                    }
                    if only_folders {
                        return BLineResult::FilteredOutAsNonFolder;
                    }
                    LineType::SymLink(match fs::read_link(&path) {
                        Ok(target) => target.to_string_lossy().into_owned(),
                        Err(_) => String::from("???"),
                    })
                } else {
                    if !has_match {
                        return BLineResult::FilteredOutByPattern;
                    }
                    if only_folders {
                        return BLineResult::FilteredOutAsNonFolder;
                    }
                    LineType::File
                }
            }
            Err(err) => {
                debug!("Error while fetching metadata: {:?}", err);
                has_error = true;
                if !has_match {
                    return BLineResult::FilteredOutByPattern;
                }
                LineType::File
            }
        };
        let mut ignore_filter = None;
        if let Some(gif) = parent_ignore_filter {
            if !gif.accepts(&path, &name, is_dir) {
                return BLineResult::GitIgnored;
            }
            if is_dir {
                ignore_filter = Some(gif.extended_to(&path));
            }
        }
        BLineResult::Some(BLine {
            parent_idx,
            path,
            depth,
            name: name.to_string(),
            childs_loaded: false,
            childs: Vec::new(),
            next_child_idx: 0,
            line_type,
            has_error,
            has_match,
            score,
            ignore_filter,
        })
    }
    fn is_file(&self) -> bool {
        match &self.line_type {
            LineType::File => true,
            _ => false,
        }
    }
    fn to_tree_line(&self) -> TreeLine {
        let mut mode = 0;
        let mut uid = 0;
        let mut gid = 0;
        if let Ok(metadata) = fs::symlink_metadata(&self.path) {
            mode = metadata.mode();
            uid = metadata.uid();
            gid = metadata.gid();
        }
        TreeLine {
            left_branchs: vec![false; self.depth as usize].into_boxed_slice(),
            depth: self.depth,
            name: self.name.to_string(),
            path: self.path.clone(),
            line_type: self.line_type.clone(),
            has_error: self.has_error,
            unlisted: self.childs.len() - self.next_child_idx,
            score: self.score,
            mode,
            uid,
            gid,
            size: None,
        }
    }
}

pub struct TreeBuilder {
    blines: Vec<BLine>, // all blines, even the ones not yet "seen" by BFS
    options: TreeOptions,
    nb_gitignored: u32, // number of times a gitignore pattern excluded a file
}
impl TreeBuilder {
    pub fn from(path: PathBuf, options: TreeOptions) -> TreeBuilder {
        let mut blines = Vec::new();
        blines.push(BLine::from_root(path, options.respect_git_ignore));
        TreeBuilder {
            blines,
            options,
            nb_gitignored: 0,
        }
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
                            &self.blines[bline_idx].ignore_filter,
                        );
                        match bl {
                            BLineResult::Some(bl) => {
                                if bl.has_match {
                                    // direct match
                                    self.blines[bline_idx].has_match = true;
                                    has_child_match = true;
                                }
                                childs.push(self.store(bl));
                            }
                            BLineResult::GitIgnored => {
                                self.nb_gitignored += 1;
                            }
                            _ => {
                                // other reason, we don't care
                            }
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
        if bline.next_child_idx < bline.childs.len() {
            let next_child = bline.childs[bline.next_child_idx];
            bline.next_child_idx += 1;
            Some(next_child)
        } else {
            None
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
                    if child.has_match {
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
                            if !bline.has_match {
                                bline.has_match = true;
                                nb_lines_ok += 1;
                            }
                            idx = bline.parent_idx;
                            if idx == 0 {
                                break;
                            }
                        }
                    }
                    open_dirs.push_back(*next_level_dir_idx);
                }
                if nb_lines_ok >= nb_lines_max {
                    break;
                }
                next_level_dirs.clear();
            }
        }

        if self.options.show_sizes {
            // if the root directory isn't totally read, we finished it even
            // it it goes past the bottom of the screen
            while let Some(child_idx) = self.next_child(0) {
                let child = &self.blines[child_idx];
                if child.has_match {
                    nb_lines_ok += 1;
                }
                out_blines.push(child_idx);
            }
        } else if self.options.pattern.is_some() {
            // At this point we usually have more lines than really needed.
            // We'll select the best ones
            // To start with, we get a better count of what we have:
            let mut count = 0;
            for idx in out_blines.iter() {
                if self.blines[*idx].has_match {
                    //debug!(" {} {:?}", self.blines[*idx].score, self.blines[*idx].path);
                    count += 1;
                }
            }
            while count > nb_lines_max {
                // we'll try to remove the less interesting line:
                //  the one with the worst score at the greatest depth
                let mut worst_index: usize = 0;
                let mut depth: u16 = 0;
                for &out_index in out_blines.iter().skip(1) {
                    let bline = &self.blines[out_index];
                    if bline.has_match && bline.depth > depth {
                        depth = bline.depth;
                    }
                }
                let mut score: i32 = std::i32::MAX;
                for &out_index in out_blines.iter().skip(1) {
                    let bline = &self.blines[out_index];
                    if !bline.has_match {
                        continue;
                    }
                    if (bline.depth == depth || bline.is_file()) && bline.score < score {
                        score = bline.score;
                        worst_index = out_index;
                    }
                }
                if worst_index > 0 {
                    // we set the has_match to 0 so the line won't be kept
                    //debug!("removing {} {:?}", self.blines[worst_index].score, self.blines[worst_index].path);
                    self.blines[worst_index].has_match = false;
                    count -= 1;
                } else {
                    break;
                }
            }
        }

        let mut lines: Vec<TreeLine> = Vec::new();
        for idx in out_blines.iter() {
            if self.blines[*idx].has_match {
                // we need to count the childs, so we load them
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

        debug!("nb gitignored files: {}", self.nb_gitignored);
        let mut tree = Tree {
            lines: lines.into_boxed_slice(),
            selection: 0,
            options: self.options.clone(),
            scroll: 0,
            nb_gitignored: self.nb_gitignored,
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
