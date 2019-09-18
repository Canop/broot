use std::cmp::{self, Ordering};
use std::collections::{BinaryHeap, VecDeque};
use std::fs;
use std::path::PathBuf;
use std::result::Result;
use std::time::{Duration, Instant};

use crate::errors::TreeBuildError;
use crate::flat_tree::{LineType, Tree, TreeLine};
use crate::git_ignore::GitIgnoreFilter;
use crate::task_sync::TaskLifetime;
use crate::tree_options::{OptionBool, TreeOptions};

/// like a tree line, but with the info needed during the build
/// This structure isn't usable independantly from the tree builder
struct BLine {
    parent_idx: usize,
    path: PathBuf,
    depth: u16,
    name: String,
    file_type: fs::FileType,
    children: Option<Vec<usize>>, // sorted and filtered (indexes of the children in tree.blines)
    next_child_idx: usize,        // index for iteration, among the children
    has_error: bool,
    has_match: bool,
    score: i32,
    ignore_filter: Option<GitIgnoreFilter>,
    nb_kept_children: i32, // used during the trimming step
}

/// the result of trying to build a bline
enum BLineResult {
    Some(BLine), // the only positive result
    FilteredOutAsHidden,
    FilteredOutByPattern,
    FilteredOutAsNonFolder,
    GitIgnored,
    Invalid,
}

impl BLine {
    /// a special constructor, checking nothing
    fn from_root(path: PathBuf, respect_ignore: OptionBool) -> Result<BLine, TreeBuildError> {
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
        if let Ok(md) = fs::metadata(&path) {
            let file_type = md.file_type();
            Ok(BLine {
                parent_idx: 0,
                path,
                depth: 0,
                name,
                children: None,
                next_child_idx: 0,
                file_type,
                has_error: false,
                has_match: true,
                score: 0,
                ignore_filter,
                nb_kept_children: 0,
            })
        } else {
            Err(TreeBuildError::FileNotFound {
                path: format!("{:?}", path),
            })
        }
    }
    /// return a bline if the direntry directly matches the options and there's no error
    fn from(
        parent_idx: usize,
        e: fs::DirEntry,
        depth: u16,
        options: &TreeOptions,
        parent_ignore_filter: &Option<GitIgnoreFilter>,
    ) -> BLineResult {
        let name = e.file_name();
        let name = match name.to_str() {
            Some(name) => name,
            None => {
                return BLineResult::Invalid;
            }
        };
        if !options.show_hidden && name.starts_with('.') {
            return BLineResult::FilteredOutAsHidden;
        }
        let mut has_match = true;
        let mut score = 10000 - i32::from(depth); // we dope less deep entries
        if options.pattern.is_some() {
            if let Some(m) = options.pattern.find(&name) {
                score += m.score;
            } else {
                has_match = false;
            }
        }
        let file_type = match e.file_type() {
            Ok(ft) => ft,
            Err(_) => {
                return BLineResult::Invalid;
            }
        };
        if file_type.is_file() || file_type.is_symlink() {
            if !has_match {
                return BLineResult::FilteredOutByPattern;
            }
            if options.only_folders {
                return BLineResult::FilteredOutAsNonFolder;
            }
        }
        let path = e.path();
        let mut ignore_filter = None;
        if let Some(gif) = parent_ignore_filter {
            if !gif.accepts(&path, &name, file_type.is_dir()) {
                return BLineResult::GitIgnored;
            }
            if file_type.is_dir() {
                ignore_filter = Some(gif.extended_to(&path));
            }
        }
        BLineResult::Some(BLine {
            parent_idx,
            path,
            depth,
            name: name.to_string(),
            file_type,
            children: None,
            next_child_idx: 0,
            has_error: false,
            has_match,
            score,
            ignore_filter,
            nb_kept_children: 0,
        })
    }

    fn to_tree_line(&self) -> std::io::Result<TreeLine> {
        let mut has_error = self.has_error;
        let line_type = if self.file_type.is_dir() {
            LineType::Dir
        } else if self.file_type.is_symlink() {
            if let Ok(target) = fs::read_link(&self.path) {
                let target = target.to_string_lossy().into_owned();
                let mut target_path = PathBuf::from(&target);
                if target_path.is_relative() {
                    target_path = self.path.parent().unwrap().join(target_path)
                }
                if let Ok(target_metadata) = fs::symlink_metadata(&target_path) {
                    if target_metadata.file_type().is_dir() {
                        LineType::SymLinkToDir(target)
                    } else {
                        LineType::SymLinkToFile(target)
                    }
                } else {
                    has_error = true;
                    LineType::SymLinkToFile(target)
                }
            } else {
                has_error = true;
                LineType::SymLinkToFile(String::from("????"))
            }
        } else {
            LineType::File
        };
        let unlisted = if let Some(children) = &self.children {
            children.len() - self.next_child_idx
        } else {
            0
        };
        let metadata = fs::symlink_metadata(&self.path)?;
        Ok(TreeLine {
            left_branchs: vec![false; self.depth as usize].into_boxed_slice(),
            depth: self.depth,
            name: self.name.to_string(),
            path: self.path.clone(),
            line_type,
            has_error,
            nb_kept_children: self.nb_kept_children as usize,
            unlisted,
            score: self.score,
            size: None,
            metadata,
        })
    }
}

// a structure making it possible to keep bline references
//  sorted in a binary heap with the line with the smallest
//  score at the top
struct SortableBLineIdx {
    idx: usize,
    score: i32,
}
impl Eq for SortableBLineIdx {}
impl PartialEq for SortableBLineIdx {
    fn eq(&self, other: &SortableBLineIdx) -> bool {
        self.score == other.score // unused but required by spec of Ord
    }
}
impl Ord for SortableBLineIdx {
    fn cmp(&self, other: &SortableBLineIdx) -> Ordering {
        if self.score == other.score {
            Ordering::Equal
        } else if self.score < other.score {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    }
}
impl PartialOrd for SortableBLineIdx {
    fn partial_cmp(&self, other: &SortableBLineIdx) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// The TreeBuilder builds a Tree according to options (including an optional search pattern)
/// Instead of the final TreeLine, the builder uses an internal structure: BLine.
/// All BLines used during build are stored in the blines vector and kept until the end.
/// Most operations and temporary data structures just deal with the indexes of lines in
///  the blines vector.
pub struct TreeBuilder {
    blines: Vec<BLine>, // all blines, even the ones not yet "seen" by BFS
    options: TreeOptions,
    targeted_size: usize, // the number of lines we should fill (height of the screen)
    nb_gitignored: u32,   // number of times a gitignore pattern excluded a file
}
impl TreeBuilder {
    pub fn from(
        path: PathBuf,
        options: TreeOptions,
        targeted_size: usize,
    ) -> Result<TreeBuilder, TreeBuildError> {
        let mut blines = Vec::new();
        blines.push(BLine::from_root(path, options.respect_git_ignore)?);
        Ok(TreeBuilder {
            blines,
            options,
            targeted_size,
            nb_gitignored: 0,
        })
    }
    /// stores (move) the bline in the global vec. Returns its index
    fn store(&mut self, bline: BLine) -> usize {
        let idx = self.blines.len();
        self.blines.push(bline);
        idx
    }
    /// returns true when there are direct matches among children
    fn load_children(&mut self, bline_idx: usize) -> bool {
        let mut has_child_match = false;
        match fs::read_dir(&self.blines[bline_idx].path) {
            Ok(entries) => {
                let mut children: Vec<usize> = Vec::new();
                for e in entries {
                    if let Ok(e) = e {
                        let bl = BLine::from(
                            bline_idx,
                            e,
                            self.blines[bline_idx].depth + 1,
                            &self.options,
                            &self.blines[bline_idx].ignore_filter,
                        );
                        match bl {
                            BLineResult::Some(bl) => {
                                if bl.has_match {
                                    // direct match
                                    self.blines[bline_idx].has_match = true;
                                    has_child_match = true;
                                }
                                children.push(self.store(bl));
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
                children.sort_by(|&a, &b| {
                    self.blines[a]
                        .name
                        .to_lowercase()
                        .cmp(&self.blines[b].name.to_lowercase())
                });
                self.blines[bline_idx].children = Some(children);
            }
            Err(_err) => {
                self.blines[bline_idx].has_error = true;
                self.blines[bline_idx].children = Some(Vec::new());
            }
        }
        has_child_match
    }
    // load_children must have been called before on bline_idx
    fn next_child(
        &mut self,
        bline_idx: usize, // the parent
    ) -> Option<usize> {
        let bline = &mut self.blines[bline_idx];
        if let Some(children) = &bline.children {
            if bline.next_child_idx < children.len() {
                let next_child = children[bline.next_child_idx];
                bline.next_child_idx += 1;
                Some(next_child)
            } else {
                Option::None
            }
        } else {
            unreachable!();
        }
    }

    /// first step of the build: we explore the directories and gather lines.
    /// If there's no search pattern we stop when we have enough lines to fill the screen.
    /// If there's a pattern, we try to gather more lines that will be sorted afterwards.
    fn gather_lines(&mut self, task_lifetime: &TaskLifetime) -> Option<Vec<usize>> {
        let start = Instant::now();
        let mut out_blines: Vec<usize> = Vec::new(); // the blines we want to display (indexes into blines)
        let not_long = Duration::from_millis(600);
        let optimal_size = self
            .options
            .pattern
            .optimal_result_number(self.targeted_size);
        out_blines.push(0);
        let mut nb_lines_ok = 1; // in out_blines
        let mut open_dirs: VecDeque<usize> = VecDeque::new();
        let mut next_level_dirs: Vec<usize> = Vec::new();
        self.load_children(0);
        open_dirs.push_back(0);
        loop {
            if (nb_lines_ok > optimal_size)
                || (nb_lines_ok >= self.targeted_size && start.elapsed() > not_long)
            {
                break;
            }
            if let Some(open_dir_idx) = open_dirs.pop_front() {
                if let Some(child_idx) = self.next_child(open_dir_idx) {
                    open_dirs.push_back(open_dir_idx);
                    let child = &self.blines[child_idx];
                    if child.has_match {
                        nb_lines_ok += 1;
                    }
                    if child.file_type.is_dir() {
                        next_level_dirs.push(child_idx);
                    }
                    out_blines.push(child_idx);
                }
            } else {
                // this depth is finished, we must go deeper
                if self.options.show_sizes {
                    // both for technical reasons (bad sort) and ergonomics
                    //  ones (it proved to be hard to read), we don't want
                    //  a deep tree when looking at sizes.
                    break;
                }
                if next_level_dirs.is_empty() {
                    // except there's nothing deeper
                    break;
                }
                for next_level_dir_idx in &next_level_dirs {
                    if task_lifetime.is_expired() {
                        info!("task expired (core build - inner loop)");
                        return None;
                    }
                    let has_child_match = self.load_children(*next_level_dir_idx);
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
                next_level_dirs.clear();
            }
        }
        if self.options.show_sizes || !self.options.trim_root {
            // if the root directory isn't totally read, we finished it even
            // it it goes past the bottom of the screen
            while let Some(child_idx) = self.next_child(0) {
                out_blines.push(child_idx);
            }
        }
        Some(out_blines)
    }

    /// Post search trimming
    /// When there's a pattern, gathering normally brings many more lines than
    ///  strictly necessary to fill the screen.
    /// This function keeps only the best ones while taking care of not
    ///  removing a parent before its children.
    fn trim_excess(&mut self, out_blines: &[usize]) {
        let mut count = 1;
        let trim_root = self.options.trim_root && !self.options.show_sizes;
        for idx in out_blines[1..].iter() {
            if self.blines[*idx].has_match {
                count += 1;
                let parent_idx = self.blines[*idx].parent_idx;
                self.blines[parent_idx].nb_kept_children += 1;
            }
        }
        let mut remove_queue: BinaryHeap<SortableBLineIdx> = BinaryHeap::new();
        for idx in out_blines[1..].iter() {
            let bline = &self.blines[*idx];
            if bline.has_match && bline.nb_kept_children == 0 && (bline.depth > 1 || trim_root)
            // keep the complete first level when showing sizes
            {
                //debug!("in list: {:?} score: {}",  &bline.path, bline.score);
                remove_queue.push(SortableBLineIdx {
                    idx: *idx,
                    score: bline.score,
                });
            }
        }
        debug!(
            "Trimming: we have {} lines for a goal of {}",
            count, self.targeted_size
        );
        while count > self.targeted_size {
            if let Some(sli) = remove_queue.pop() {
                //debug!("removing {:?}", &self.blines[sli.idx].path);
                self.blines[sli.idx].has_match = false;
                let parent_idx = self.blines[sli.idx].parent_idx;
                let mut parent = &mut self.blines[parent_idx];
                parent.nb_kept_children -= 1;
                parent.next_child_idx -= 1; // to fix the number of "unlisted"
                if parent.nb_kept_children == 0 {
                    remove_queue.push(SortableBLineIdx {
                        idx: parent_idx,
                        score: parent.score,
                    });
                }
                count -= 1;
            } else {
                debug!("trimming prematurely interrupted");
                break;
            }
        }
    }

    // makes a tree from the builder's specific structure
    fn take(&mut self, out_blines: &[usize]) -> Tree {
        let mut lines: Vec<TreeLine> = Vec::new();
        for idx in out_blines.iter() {
            if self.blines[*idx].has_match {
                // we need to count the children, so we load them
                if self.blines[*idx].file_type.is_dir() && self.blines[*idx].children.is_none() {
                    self.load_children(*idx);
                }
                if let Ok(tree_line) = self.blines[*idx].to_tree_line() {
                    lines.push(tree_line);
                } else {
                    // I guess the file went missing during tree computation
                    warn!(
                        "Error while builind treeline for {:?}",
                        self.blines[*idx].path,
                    );
                }
            }
        }
        let mut tree = Tree {
            lines: lines.into_boxed_slice(),
            selection: 0,
            options: self.options.clone(),
            scroll: 0,
            nb_gitignored: self.nb_gitignored,
        };
        tree.after_lines_changed();

        if self.options.show_sizes {
            tree.fetch_file_sizes(); // not the dirs, only simple files
        }
        tree
    }

    /// build a tree. Can be called only once per builder.
    ///
    /// Return None if the lifetime expires before end of computation
    /// (usually because the user hit a key)
    pub fn build(mut self, task_lifetime: &TaskLifetime) -> Option<Tree> {
        debug!("start building with pattern {}", self.options.pattern);
        match self.gather_lines(task_lifetime) {
            Some(out_blines) => {
                self.trim_excess(&out_blines);
                Some(self.take(&out_blines))
            }
            None => None, // interrupted
        }
    }
}
