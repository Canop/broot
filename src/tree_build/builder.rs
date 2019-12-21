use {
    crate::{
        errors::TreeBuildError,
        flat_tree::{Tree, TreeLine},
        task_sync::TaskLifetime,
        tree_options::{TreeOptions},
    },
    id_arena::Arena,
    std::{
        collections::{BinaryHeap, VecDeque},
        fs,
        path::PathBuf,
        result::Result,
        time::{Duration, Instant},
    },
    super::{
        bline::BLine,
        bid::{BId, SortableBId},
    },
};

/// If a search found enough results to fill the screen but didn't scan
/// everything, we search a little more in case we find better matches
/// but not after the NOT_LONG duration.
static NOT_LONG: Duration = Duration::from_millis(900);

/// the result of trying to build a bline
enum BLineResult {
    Some(BId), // the only positive result
    FilteredOutAsHidden,
    FilteredOutByPattern,
    FilteredOutAsNonFolder,
    GitIgnored,
    Invalid,
}

/// The TreeBuilder builds a Tree according to options (including an optional search pattern)
/// Instead of the final TreeLine, the builder uses an internal structure: BLine.
/// All BLines used during build are stored in the blines vector and kept until the end.
/// Most operations and temporary data structures just deal with the indexes of lines in
///  the blines vector.
pub struct TreeBuilder {
    options: TreeOptions,
    targeted_size: usize, // the number of lines we should fill (height of the screen)
    nb_gitignored: u32,   // number of times a gitignore pattern excluded a file
    blines: Arena<BLine>,
    root_id: BId,
}
impl TreeBuilder {
    pub fn from(
        path: PathBuf,
        options: TreeOptions,
        targeted_size: usize,
    ) -> Result<TreeBuilder, TreeBuildError> {
        let mut blines = Arena::new();
        let root_id = BLine::from_root(&mut blines, path, options.respect_git_ignore)?;
        Ok(TreeBuilder {
            options,
            targeted_size,
            nb_gitignored: 0,
            blines,
            root_id,
        })
    }
    /// return a bline if the direntry directly matches the options and there's no error
    fn make_line(&mut self, parent_id: BId, e: fs::DirEntry, depth: u16) -> BLineResult {
        let name = e.file_name();
        let name = match name.to_str() {
            Some(name) => name,
            None => {
                return BLineResult::Invalid;
            }
        };
        if !self.options.show_hidden && name.starts_with('.') {
            return BLineResult::FilteredOutAsHidden;
        }
        let mut has_match = true;
        let mut score = 10000 - i32::from(depth); // we dope less deep entries
        if self.options.pattern.is_some() {
            if let Some(pattern_score) = self.options.pattern.score_of(&name) {
                score += pattern_score;
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
            if self.options.only_folders {
                return BLineResult::FilteredOutAsNonFolder;
            }
        }
        let path = e.path();
        let mut ignore_filter = None;
        if let Some(gif) = &self.blines[parent_id].ignore_filter {
            if !gif.accepts(&path, &name, file_type.is_dir()) {
                return BLineResult::GitIgnored;
            }
            if file_type.is_dir() {
                ignore_filter = Some(gif.extended_to(&path));
            }
        }
        BLineResult::Some(self.blines.alloc(BLine {
            parent_id: Some(parent_id),
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
        }))
    }

    /// returns true when there are direct matches among children
    fn load_children(&mut self, bid: BId) -> bool {
        let mut has_child_match = false;
        match fs::read_dir(&self.blines[bid].path) {
            Ok(entries) => {
                let mut children: Vec<BId> = Vec::new();
                let child_depth = self.blines[bid].depth + 1;
                for e in entries {
                    if let Ok(e) = e {
                        let bl = self.make_line(bid, e, child_depth);
                        match bl {
                            BLineResult::Some(child_id) => {
                                if self.blines[child_id].has_match {
                                    // direct match
                                    self.blines[bid].has_match = true;
                                    has_child_match = true;
                                }
                                children.push(child_id);
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
                self.blines[bid].children = Some(children);
            }
            Err(_err) => {
                self.blines[bid].has_error = true;
                self.blines[bid].children = Some(Vec::new());
            }
        }
        has_child_match
    }

    /// return the next child.
    /// load_children must have been called before on parent_id
    fn next_child(&mut self, parent_id: BId) -> Option<BId> {
        let bline = &mut self.blines[parent_id];
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
    fn gather_lines(&mut self, task_lifetime: &TaskLifetime) -> Option<Vec<BId>> {
        let start = Instant::now();
        let mut out_blines: Vec<BId> = Vec::new(); // the blines we want to display
        let optimal_size = self
            .options
            .pattern
            .optimal_result_number(self.targeted_size);
        out_blines.push(self.root_id);
        let mut nb_lines_ok = 1; // in out_blines
        let mut open_dirs: VecDeque<BId> = VecDeque::new();
        let mut next_level_dirs: Vec<BId> = Vec::new();
        self.load_children(self.root_id);
        open_dirs.push_back(self.root_id);
        loop {
            if (nb_lines_ok > optimal_size)
                || (nb_lines_ok >= self.targeted_size && start.elapsed() > NOT_LONG)
            {
                break;
            }
            if let Some(open_dir_id) = open_dirs.pop_front() {
                if let Some(child_id) = self.next_child(open_dir_id) {
                    open_dirs.push_back(open_dir_id);
                    let child = &self.blines[child_id];
                    if child.has_match {
                        nb_lines_ok += 1;
                    }
                    if child.file_type.is_dir() {
                        next_level_dirs.push(child_id);
                    }
                    out_blines.push(child_id);
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
                for next_level_dir_id in &next_level_dirs {
                    if task_lifetime.is_expired() {
                        info!("task expired (core build - inner loop)");
                        return None;
                    }
                    let has_child_match = self.load_children(*next_level_dir_id);
                    if has_child_match {
                        // we must ensure the ancestors are made Ok
                        let mut id = *next_level_dir_id;
                        loop {
                            let mut bline = &mut self.blines[id];
                            if !bline.has_match {
                                bline.has_match = true;
                                nb_lines_ok += 1;
                            }
                            if let Some(pid) = bline.parent_id {
                                id = pid;
                            } else {
                                break;
                            }
                        }
                    }
                    open_dirs.push_back(*next_level_dir_id);
                }
                next_level_dirs.clear();
            }
        }
        if self.options.show_sizes || !self.options.trim_root {
            // if the root directory isn't totally read, we finished it even
            // it it goes past the bottom of the screen
            while let Some(child_id) = self.next_child(self.root_id) {
                out_blines.push(child_id);
            }
        }
        Some(out_blines)
    }

    /// Post search trimming
    /// When there's a pattern, gathering normally brings many more lines than
    ///  strictly necessary to fill the screen.
    /// This function keeps only the best ones while taking care of not
    ///  removing a parent before its children.
    fn trim_excess(&mut self, out_blines: &[BId]) {
        let mut count = 1;
        let trim_root = self.options.trim_root && !self.options.show_sizes;
        for id in out_blines[1..].iter() {
            if self.blines[*id].has_match {
                //debug!("bline before trimming: {:?}", &self.blines[*idx].path);
                count += 1;
                let parent_id = self.blines[*id].parent_id.unwrap();
                // (we can unwrap because only the root can have a None parent)
                self.blines[parent_id].nb_kept_children += 1;
            }
        }
        let mut remove_queue: BinaryHeap<SortableBId> = BinaryHeap::new();
        for id in out_blines[1..].iter() {
            let bline = &self.blines[*id];
            if bline.has_match && bline.nb_kept_children == 0 && (bline.depth > 1 || trim_root)
            // keep the complete first level when showing sizes
            {
                //debug!("in list: {:?} score: {}",  &bline.path, bline.score);
                remove_queue.push(SortableBId {
                    id: *id,
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
                self.blines[sli.id].has_match = false;
                let parent_id = self.blines[sli.id].parent_id.unwrap();
                let mut parent = &mut self.blines[parent_id];
                parent.nb_kept_children -= 1;
                parent.next_child_idx -= 1; // to fix the number of "unlisted"
                if parent.nb_kept_children == 0 {
                    remove_queue.push(SortableBId {
                        id: parent_id,
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

    /// makes a tree from the builder's specific structure
    fn take(mut self, out_blines: &[BId]) -> Tree {
        let mut lines: Vec<TreeLine> = Vec::new();
        for id in out_blines.iter() {
            if self.blines[*id].has_match {
                // we need to count the children, so we load them
                if self.blines[*id].file_type.is_dir() && self.blines[*id].children.is_none() {
                    self.load_children(*id);
                }
                if let Ok(tree_line) = self.blines[*id].to_tree_line() {
                    lines.push(tree_line);
                } else {
                    // I guess the file went missing during tree computation
                    warn!(
                        "Error while builind treeline for {:?}",
                        self.blines[*id].path,
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
