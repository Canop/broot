use {
    super::{
        bid::{BId, SortableBId},
        BuildReport,
        bline::BLine,
    },
    crate::{
        app::AppContext,
        errors::TreeBuildError,
        git::{GitIgnoreChain, GitIgnorer, LineStatusComputer},
        pattern::Candidate,
        path::Directive,
        task_sync::ComputationResult,
        task_sync::Dam,
        tree::*,
    },
    git2::Repository,
    id_arena::Arena,
    std::{
        collections::{BinaryHeap, VecDeque},
        fs,
        path::PathBuf,
        result::Result,
        time::{Duration, Instant},
    },
};

#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;

#[cfg(target_os = "windows")]
use std::ffi::OsStr;

#[cfg(target_os = "windows")]
trait OsStrWin {
    fn as_bytes(&self) -> &[u8];
}

#[cfg(target_os = "windows")]
impl OsStrWin for OsStr {
    fn as_bytes(&self) -> &[u8] {
        static INVALID_UTF8: &[u8] = b"invalid utf8";
        self.to_str().map(|s| s.as_bytes()).unwrap_or(INVALID_UTF8)
    }
}
/// If a search found enough results to fill the screen but didn't scan
/// everything, we search a little more in case we find better matches
/// but not after the NOT_LONG duration.
static NOT_LONG: Duration = Duration::from_millis(900);

/// The TreeBuilder builds a Tree according to options (including an optional search pattern)
/// Instead of the final TreeLine, the builder uses an internal structure: BLine.
/// All BLines used during build are stored in the blines arena and kept until the end.
/// Most operations and temporary data structures just deal with the ids of lines
///  the blines arena.
pub struct TreeBuilder<'c> {
    pub options: TreeOptions,
    targeted_size: usize, // the number of lines we should fill (height of the screen)
    blines: Arena<BLine>,
    root_id: BId,
    total_search: bool,
    git_ignorer: GitIgnorer,
    line_status_computer: Option<LineStatusComputer>,
    con: &'c AppContext,
    pub matches_max: Option<usize>, // optional hard limit
    trim_root: bool,
    pub deep: bool,
    report: BuildReport,
}
impl<'c> TreeBuilder<'c> {

    pub fn from(
        path: PathBuf,
        options: TreeOptions,
        targeted_size: usize,
        con: &'c AppContext,
    ) -> Result<TreeBuilder<'c>, TreeBuildError> {
        let mut blines = Arena::new();
        let mut git_ignorer = time!(GitIgnorer::default());
        let root_ignore_chain = git_ignorer.root_chain(&path);
        let line_status_computer = if options.filter_by_git_status || options.show_git_file_info {
            time!(
                "init line_status_computer",
                Repository::discover(&path)
                    .ok()
                    .and_then(LineStatusComputer::from),
            )
        } else {
            None
        };
        let root_id = BLine::from_root(&mut blines, path, root_ignore_chain, &options)?;
        let trim_root = match (options.trim_root, options.pattern.is_some(), options.sort.prevent_deep_display()) {
            // we never want to trim the root if there's a sort
            (_, _, true) => false,
            // if the user don't want root trimming, we don't trim
            (false, _, _) => false,
            // if there's a pattern, we try to show at least root matches
            (_, true, _) => false,
            // in other cases, as the user wants trimming, we trim
            _ => true,
        };
        Ok(TreeBuilder {
            options,
            targeted_size,
            blines,
            root_id,
            total_search: true, // we'll set it to false if we don't look at all children
            git_ignorer,
            line_status_computer,
            con,
            trim_root,
            matches_max: None,
            deep: true,
            report: BuildReport::default(),
        })
    }

    /// Return a bline if the dir_entry directly matches the options and there's no error
    fn make_line(
        &mut self,
        parent_id: BId,
        e: &fs::DirEntry,
        depth: u16,
    ) -> Option<BLine> {
        let name = e.file_name();
        if name.is_empty() {
            self.report.error_count += 1;
            return None;
        }
        let path = e.path();
        let special_handling = self.con.special_paths.find(&path);
        if special_handling.show == Directive::Never {
            return None;
        }
        if !self.options.show_hidden
            && name.as_bytes()[0] == b'.'
            && special_handling.show != Directive::Always
        {
            self.report.hidden_count += 1;
            return None;
        }
        let name = name.to_string_lossy();
        let mut has_match = true;
        let mut score = 10000 - i32::from(depth); // we dope less deep entries
        let file_type = match e.file_type() {
            Ok(ft) => ft,
            Err(_) => {
                self.report.error_count += 1;
                return None;
            }
        };
        let parent_subpath = &self.blines[parent_id].subpath;
        let subpath = if !parent_subpath.is_empty() {
            format!("{}/{}", parent_subpath, &name)
        } else {
            name.to_string()
        };
        let candidate = Candidate {
            name: &name,
            subpath: &subpath,
            path: &path,
            regular_file: file_type.is_file(),
        };
        let direct_match = if let Some(pattern_score) = self.options.pattern.pattern.score_of(candidate) {
            // we dope direct matches to compensate for depth doping of parent folders
            score += pattern_score + 10;
            true
        } else {
            has_match = false;
            false
        };
        let name = name.to_string();
        if has_match && self.options.filter_by_git_status {
            if let Some(line_status_computer) = &self.line_status_computer {
                if !line_status_computer.is_interesting(&path) {
                    has_match = false;
                }
            }
        }
        if file_type.is_file() {
            if !has_match {
                return None;
            }
        }
        if self.options.only_folders && !file_type.is_dir() {
            if !file_type.is_symlink() {
                return None;
            }
            let Ok(target_metadata) = fs::metadata(&path) else {
                return None;
            };
            if !target_metadata.is_dir() {
                return None;
            }
        }
        if self.options.respect_git_ignore {
            let parent_chain = &self.blines[parent_id].git_ignore_chain;
            if !self
                .git_ignorer
                .accepts(parent_chain, &path, &name, file_type.is_dir())
            {
                if special_handling.show != Directive::Always {
                    return None;
                }
            }
        };
        Some(BLine {
            parent_id: Some(parent_id),
            path,
            depth,
            subpath,
            name,
            file_type,
            children: None,
            next_child_idx: 0,
            has_error: false,
            has_match,
            direct_match,
            score,
            nb_kept_children: 0,
            git_ignore_chain: GitIgnoreChain::default(),
            special_handling,
        })
    }

    /// Return true when there are direct matches among children
    fn load_children(&mut self, bid: BId) -> bool {
        let mut has_child_match = false;
        match self.blines[bid].read_dir() {
            Ok(entries) => {
                let mut children: Vec<BId> = Vec::new();
                let child_depth = self.blines[bid].depth + 1;
                let mut lines = Vec::new();
                for e in entries.flatten() {
                    if let Some(line) = self.make_line(bid, &e, child_depth) {
                        lines.push(line);
                    }
                }
                for mut bl in lines {
                    if self.options.respect_git_ignore {
                        let parent_chain = &self.blines[bid].git_ignore_chain;
                        bl.git_ignore_chain = if bl.file_type.is_dir() {
                            self.git_ignorer.deeper_chain(parent_chain, &bl.path)
                        } else {
                            parent_chain.clone()
                        };
                    }
                    if bl.has_match {
                        self.blines[bid].has_match = true;
                        has_child_match = true;
                    }
                    let child_id = self.blines.alloc(bl);
                    children.push(child_id);
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
    fn gather_lines(&mut self, total_search: bool, dam: &Dam) -> Result<Vec<BId>, TreeBuildError> {
        let start = Instant::now();
        let mut out_blines: Vec<BId> = Vec::new(); // the blines we want to display
        let optimal_size = if self.options.pattern.pattern.has_real_scores() {
            10 * self.targeted_size
        } else {
            self.targeted_size
        };
        out_blines.push(self.root_id);
        let mut nb_lines_ok = 1; // in out_blines
        let mut open_dirs: VecDeque<BId> = VecDeque::new();
        let mut next_level_dirs: Vec<BId> = Vec::new();
        self.load_children(self.root_id);
        open_dirs.push_back(self.root_id);
        let deep = self.deep && self.options.show_tree && !self.options.sort.prevent_deep_display();
        loop {
            if !total_search && (
                (nb_lines_ok > optimal_size)
                || (nb_lines_ok >= self.targeted_size && start.elapsed() > NOT_LONG)
            ) {
                self.total_search = false;
                break;
            }
            if let Some(max) = self.matches_max {
                if nb_lines_ok > max {
                    return Err(TreeBuildError::TooManyMatches{max});
                }
            }
            if let Some(open_dir_id) = open_dirs.pop_front() {
                if let Some(child_id) = self.next_child(open_dir_id) {
                    open_dirs.push_back(open_dir_id);
                    let child = &self.blines[child_id];
                    if child.has_match {
                        nb_lines_ok += 1;
                    }
                    if child.can_enter() {
                        next_level_dirs.push(child_id);
                    }
                    out_blines.push(child_id);
                }
            } else {
                // this depth is finished, we must go deeper
                if !deep {
                    break;
                }
                if next_level_dirs.is_empty() {
                    // except there's nothing deeper
                    break;
                }
                for next_level_dir_id in &next_level_dirs {
                    if dam.has_event() {
                        info!("task expired (core build - inner loop)");
                        return Err(TreeBuildError::Interrupted);
                    }
                    let has_child_match = self.load_children(*next_level_dir_id);
                    if has_child_match {
                        // we must ensure the ancestors are made Ok
                        let mut id = *next_level_dir_id;
                        loop {
                            let bline = &mut self.blines[id];
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
        if let Some(max) = self.matches_max {
            if nb_lines_ok > max {
                return Err(TreeBuildError::TooManyMatches{max});
            }
        }
        if !self.trim_root {
            // if the root directory isn't totally read, we finished it even
            // it it goes past the bottom of the screen
            while let Some(child_id) = self.next_child(self.root_id) {
                out_blines.push(child_id);
            }
        }
        Ok(out_blines)
    }

    /// Post search trimming
    /// When there's a pattern, gathering normally brings many more lines than
    ///  strictly necessary to fill the screen.
    /// This function keeps only the best ones while taking care of not
    ///  removing a parent before its children.
    fn trim_excess(&mut self, out_blines: &[BId]) {
        let mut count = 1;
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
            if bline.has_match && bline.nb_kept_children == 0 && (bline.depth > 1 || self.trim_root)
            {
                //debug!("in list: {:?} score: {}",  &bline.path, bline.score);
                remove_queue.push(SortableBId {
                    id: *id,
                    score: bline.score,
                });
            }
        }
        while count > self.targeted_size {
            if let Some(sli) = remove_queue.pop() {
                self.blines[sli.id].has_match = false;
                let parent_id = self.blines[sli.id].parent_id.unwrap();
                let parent = &mut self.blines[parent_id];
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

    /// make a tree from the builder's specific structure
    fn take_as_tree(mut self, out_blines: &[BId]) -> Tree {
        let mut lines: Vec<TreeLine> = Vec::new();
        for id in out_blines.iter() {
            if self.blines[*id].has_match {
                // we need to count the children, so we load them
                if self.blines[*id].can_enter() && self.blines[*id].children.is_none() {
                    self.load_children(*id);
                }
                if let Ok(tree_line) = self.blines[*id].to_tree_line(*id, self.con) {
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
            total_search: self.total_search,
            git_status: ComputationResult::None,
            build_report: self.report,
        };
        tree.after_lines_changed();
        if let Some(computer) = self.line_status_computer {
            // tree git status is slow to compute, we just mark it should be
            // done (later on)
            tree.git_status = ComputationResult::NotComputed;
            // it would make no sense to keep only files having a git status and
            // not display that type
            for line in tree.lines.iter_mut() {
                line.git_status = computer.line_status(&line.path);
            }
        }
        tree
    }

    /// build a tree. Can be called only once per builder.
    ///
    /// Return None if the lifetime expires before end of computation
    /// (usually because the user hit a key)
    pub fn build_tree(mut self, total_search: bool, dam: &Dam) -> Result<Tree, TreeBuildError> {
        self.gather_lines(total_search, dam)
            .map(|blines_ids| {
                debug!("blines before trimming: {}", blines_ids.len());
                if !self.total_search {
                    self.trim_excess(&blines_ids);
                }
                self.take_as_tree(&blines_ids)
            })
    }

    ///
    pub fn build_paths<F>(
        mut self,
        total_search: bool,
        dam: &Dam,
        filter: F
    ) -> Result<Vec<PathBuf>, TreeBuildError>
    where F: Fn(&BLine) -> bool
    {
        self.gather_lines(total_search, dam)
            .map(|mut blines_ids| {
                blines_ids
                    .drain(..)
                    .filter(|&bid| filter(&self.blines[bid]))
                    .map(|id| self.blines[id].path.clone())
                    .collect()
            })
    }
}
