use {
    super::*,
    crate::{
        app::AppContext,
        errors,
        file_sum::FileSum,
        git::TreeGitStatus,
        task_sync::ComputationResult,
        task_sync::Dam,
        tree_build::{BId, BuildReport, TreeBuilder},
    },
    fnv::FnvHashMap,
    std::{
        cmp::Ord,
        mem,
        path::{Path, PathBuf},
    },
};

/// The tree which may be displayed, with onle line per visible line of the panel.
///
/// In the tree structure, every "node" is just a line, there's
///  no link from a child to its parent or from a parent to its children.
#[derive(Debug, Clone)]
pub struct Tree {
    pub lines: Box<[TreeLine]>,
    pub selection: usize, // there's always a selection (starts with root, which is 0)
    pub options: TreeOptions,
    pub scroll: usize, // the number of lines at the top hidden because of scrolling
    pub total_search: bool, // whether the search was made on all children
    pub git_status: ComputationResult<TreeGitStatus>,
    pub build_report: BuildReport,
}

impl Tree {

    /// rebuild the tree with the same root, height, and options
    pub fn refresh(
        &mut self,
        page_height: usize,
        con: &AppContext,
    ) -> Result<(), errors::TreeBuildError> {
        let builder = TreeBuilder::from(
            self.root().to_path_buf(),
            self.options.clone(),
            page_height,
            con,
        )?;
        let mut tree = builder
            .build_tree(
                false, // on refresh we always do a non total search
                &Dam::unlimited(),
            )
            .unwrap(); // should not fail
                       // we save the old selection to try restore it
        let selected_path = self.selected_line().path.to_path_buf();
        mem::swap(&mut self.lines, &mut tree.lines);
        self.scroll = 0;
        if !self.try_select_path(&selected_path) && self.selection >= self.lines.len() {
            self.selection = 0;
        }
        self.make_selection_visible(page_height);
        Ok(())
    }

    /// do what must be done after line additions or removals:
    /// - sort the lines
    /// - compute left branches
    pub fn after_lines_changed(&mut self) {

        // we need to order the lines to build the tree.
        // It's a little complicated because
        //  - we want a case insensitive sort
        //  - we still don't want to confuse the children of AA and Aa
        //  - a node can come from a not parent node, when we followed a link
        let mut bid_parents: FnvHashMap<BId, BId> = FnvHashMap::default();
        let mut bid_lines: FnvHashMap<BId, &TreeLine> = FnvHashMap::default();
        for line in self.lines[..].iter() {
            if let Some(parent_bid) = line.parent_bid {
                bid_parents.insert(line.bid, parent_bid);
            }
            bid_lines.insert(line.bid, line);
        }
        let mut sort_paths: FnvHashMap<BId, String> = FnvHashMap::default();
        for line in self.lines[1..].iter() {
            let mut sort_path = String::new();
            let mut bid = line.bid;
            while let Some(l) = bid_lines.get(&bid) {
                let lower_name = l.path.file_name().map_or(
                    "".to_string(),
                    |name| name.to_string_lossy().to_lowercase(),
                );
                let sort_prefix = match self.options.sort {
                    Sort::TypeDirsFirst => {
                        if l.is_dir() {
                            "              "
                        } else {
                            l.path.extension().and_then(|s| s.to_str()).unwrap_or("")
                        }
                    }
                    Sort::TypeDirsLast => {
                        if l.is_dir() {
                            "~~~~~~~~~~~~~~"
                        } else {
                            l.path.extension().and_then(|s| s.to_str()).unwrap_or("")
                        }
                    }
                    _ => { "" }
                };
                sort_path = format!(
                    "{}{}-{}/{}",
                    sort_prefix,
                    lower_name,
                    bid.index(), // to be sure to separate paths having the same lowercase
                    sort_path,
                );
                if let Some(&parent_bid) = bid_parents.get(&bid) {
                    bid = parent_bid;
                } else {
                    break;
                }
            }
            sort_paths.insert(line.bid, sort_path);
        }
        self.lines[1..].sort_by_key(|line| sort_paths.get(&line.bid).unwrap());

        let mut best_index = 0; // index of the line with the best score
        for i in 1..self.lines.len() {
            if self.lines[i].score > self.lines[best_index].score {
                best_index = i;
            }
            for d in 0..self.lines[i].left_branches.len() {
                self.lines[i].left_branches[d] = false;
            }
        }
        // then we discover the branches (for the drawing)
        // and we mark the last children as pruning, if they have unlisted brothers
        let mut last_parent_index: usize = self.lines.len() + 1;
        for end_index in (1..self.lines.len()).rev() {
            let depth = (self.lines[end_index].depth - 1) as usize;
            let start_index = {
                let parent_index = match self.lines[end_index].parent_bid {
                    Some(parent_bid) => {
                        let mut index = end_index;
                        loop {
                            index -= 1;
                            if self.lines[index].bid == parent_bid {
                                break;
                            }
                            if index == 0 {
                                break;
                            }
                        }
                        index
                    }
                    None => end_index, // Should not happen
                };
                if parent_index != last_parent_index {
                    // the line at end_index is the last listed child of the line at parent_index
                    let unlisted = self.lines[parent_index].unlisted;
                    if unlisted > 0 && self.lines[end_index].nb_kept_children == 0 {
                        if best_index == end_index {
                            //debug!("Avoiding to prune the line with best score");
                        } else {
                            //debug!("turning {:?} into Pruning", self.lines[end_index].path);
                            self.lines[end_index].line_type = TreeLineType::Pruning;
                            self.lines[end_index].unlisted = unlisted + 1;
                            self.lines[end_index].name = format!("{} unlisted", unlisted + 1);
                            self.lines[parent_index].unlisted = 0;
                        }
                    }
                    last_parent_index = parent_index;
                }
                parent_index + 1
            };
            for i in start_index..=end_index {
                self.lines[i].left_branches[depth] = true;
            }
        }
        if self.options.needs_sum() {
            time!("fetch_file_sum", self.fetch_regular_file_sums()); // not the dirs, only simple files
            self.sort_siblings(); // does nothing when sort mode is None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.lines.len() == 1
    }

    pub fn has_branch(&self, line_index: usize, depth: usize) -> bool {
        if line_index >= self.lines.len() {
            return false;
        }
        let line = &self.lines[line_index];
        depth < usize::from(line.depth) && line.left_branches[depth]
    }

    /// select another line
    ///
    /// For example the following one if dy is 1.
    pub fn move_selection(&mut self, dy: i32, page_height: usize, cycle: bool) {
        let l = self.lines.len();
        // we find the new line to select
        loop {
            if dy < 0 {
                let ady = (-dy) as usize;
                if !cycle && self.selection < ady {
                    break;
                }
                self.selection = (self.selection + l - ady) % l;
            } else {
                let dy = dy as usize;
                if !cycle && self.selection + dy >= l {
                    break;
                }
                self.selection = (self.selection + dy) % l;
            }
            if self.lines[self.selection].is_selectable() {
                break;
            }
        }
        // we adjust the scroll
        if l > page_height {
            if self.selection < 3 {
                self.scroll = 0;
            } else if self.selection < self.scroll + 3 {
                self.scroll = self.selection - 3;
            } else if self.selection + 3 > l {
                self.scroll = l - page_height;
            } else if self.selection + 3 > self.scroll + page_height {
                self.scroll = self.selection + 3 - page_height;
            }
        }
    }

    /// Scroll the desired amount and return true, or return false if it's
    /// already at end or the tree fits the page
    pub fn try_scroll(&mut self, dy: i32, page_height: usize) -> bool {
        if self.lines.len() <= page_height {
            return false;
        }
        if dy < 0 { // scroll up
            if self.scroll == 0 {
                return false;
            } else {
                let ady = -dy as usize;
                if ady < self.scroll {
                    self.scroll -= ady;
                } else {
                    self.scroll = 0;
                }
            }
        } else { // scroll down
            let max = self.lines.len() - page_height;
            if self.scroll >= max {
                return false;
            }
            self.scroll = (self.scroll + dy as usize).min(max);
        }
        self.select_visible_line(page_height);
        true
    }

    /// try to select a line by index of visible line
    /// (works if y+scroll falls on a selectable line)
    pub fn try_select_y(&mut self, y: usize) -> bool {
        let y = y + self.scroll;
        if y < self.lines.len() && self.lines[y].is_selectable() {
            self.selection = y;
            return true;
        }
        false
    }
    /// fix the selection so that it's a selectable visible line
    fn select_visible_line(&mut self, page_height: usize) {
        if self.selection < self.scroll || self.selection >= self.scroll + page_height {
            self.selection = self.scroll;
            let l = self.lines.len();
            loop {
                self.selection = (self.selection + l + 1) % l;
                if self.lines[self.selection].is_selectable() {
                    break;
                }
            }
        }
    }

    pub fn make_selection_visible(&mut self, page_height: usize) {
        if page_height >= self.lines.len() || self.selection < 3 {
            self.scroll = 0;
        } else if self.selection <= self.scroll {
            self.scroll = self.selection - 2;
        } else if self.selection > self.lines.len() - 2 {
            self.scroll = self.lines.len() - page_height;
        } else if self.selection >= self.scroll + page_height {
            self.scroll = self.selection + 2 - page_height;
        }
    }
    pub fn selected_line(&self) -> &TreeLine {
        &self.lines[self.selection]
    }
    pub fn root(&self) -> &PathBuf {
        &self.lines[0].path
    }
    /// select the line with the best matching score
    pub fn try_select_best_match(&mut self) {
        let mut best_score = 0;
        for (idx, line) in self.lines.iter().enumerate() {
            if !line.is_selectable() {
                continue;
            }
            if best_score > line.score {
                continue;
            }
            if line.score == best_score {
                // in case of equal scores, we prefer the shortest path
                if self.lines[idx].depth >= self.lines[self.selection].depth {
                    continue;
                }
            }
            best_score = line.score;
            self.selection = idx;
        }
    }
    /// return true when we could select the given path
    pub fn try_select_path(&mut self, path: &Path) -> bool {
        for (idx, line) in self.lines.iter().enumerate() {
            if !line.is_selectable() {
                continue;
            }
            if path == line.path {
                self.selection = idx;
                return true;
            }
        }
        false
    }
    pub fn try_select_first(&mut self) -> bool {
        for idx in 0..self.lines.len() {
            let line = &self.lines[idx];
            if line.is_selectable() {
                self.selection = idx;
                self.scroll = 0;
                return true;
            }
        }
        false
    }
    pub fn try_select_last(&mut self, page_height: usize) -> bool {
        for idx in (0..self.lines.len()).rev() {
            let line = &self.lines[idx];
            if line.is_selectable() {
                self.selection = idx;
                self.make_selection_visible(page_height);
                return true;
            }
        }
        false
    }
    pub fn try_select_previous_same_depth(&mut self, page_height: usize) -> bool {
        let depth = self.lines[self.selection].depth;
        for di in (0..self.lines.len()).rev() {
            let idx = (self.selection + di) % self.lines.len();
            let line = &self.lines[idx];
            if !line.is_selectable() || line.depth != depth {
                continue;
            }
            self.selection = idx;
            self.make_selection_visible(page_height);
            return true;
        }
        false
    }
    pub fn try_select_next_same_depth(&mut self, page_height: usize) -> bool {
        let depth = self.lines[self.selection].depth;
        for di in 0..self.lines.len() {
            let idx = (self.selection + di + 1) % self.lines.len();
            let line = &self.lines[idx];
            if !line.is_selectable() || line.depth != depth {
                continue;
            }
            self.selection = idx;
            self.make_selection_visible(page_height);
            return true;
        }
        false
    }
    pub fn try_select_previous_filtered<F>(
        &mut self,
        filter: F,
        page_height: usize,
    ) -> bool where
        F: Fn(&TreeLine) -> bool,
    {
        for di in (0..self.lines.len()).rev() {
            let idx = (self.selection + di) % self.lines.len();
            let line = &self.lines[idx];
            if !line.is_selectable() {
                continue;
            }
            if !filter(line) {
                continue;
            }
            if line.score > 0 {
                self.selection = idx;
                self.make_selection_visible(page_height);
                return true;
            }
        }
        false
    }
    pub fn try_select_next_filtered<F>(
        &mut self,
        filter: F,
        page_height: usize,
    ) -> bool where
        F: Fn(&TreeLine) -> bool,
    {
        for di in 0..self.lines.len() {
            let idx = (self.selection + di + 1) % self.lines.len();
            let line = &self.lines[idx];
            if !line.is_selectable() {
                continue;
            }
            if !filter(line) {
                continue;
            }
            if line.score > 0 {
                self.selection = idx;
                self.make_selection_visible(page_height);
                return true;
            }
        }
        false
    }

    pub fn has_dir_missing_sum(&self) -> bool {
        self.options.needs_sum()
            && self
                .lines
                .iter()
                .any(|line| line.line_type == TreeLineType::Dir && line.sum.is_none())
    }

    pub fn is_missing_git_status_computation(&self) -> bool {
        self.git_status.is_not_computed()
    }

    /// fetch the file_sums of regular files (thus avoiding the
    /// long computation which is needed for directories)
    pub fn fetch_regular_file_sums(&mut self) {
        for i in 1..self.lines.len() {
            match self.lines[i].line_type {
                TreeLineType::Dir | TreeLineType::Pruning => {}
                _ => {
                    self.lines[i].sum = Some(FileSum::from_file(&self.lines[i].path));
                }
            }
        }
        self.sort_siblings();
    }

    /// compute the file_sum of one directory
    ///
    /// To compute the size of all of them, this should be called until
    ///  has_dir_missing_sum returns false
    pub fn fetch_some_missing_dir_sum(&mut self, dam: &Dam, con: &AppContext) {
        // we prefer to compute the root directory last: its computation
        // is faster when its first level children are already computed
        for i in (0..self.lines.len()).rev() {
            if self.lines[i].sum.is_none() && self.lines[i].line_type == TreeLineType::Dir {
                self.lines[i].sum = FileSum::from_dir(&self.lines[i].path, dam, con);
                self.sort_siblings();
                return;
            }
        }
    }

    /// Sort files according to the sort option
    ///
    /// (does nothing if it's None)
    fn sort_siblings(&mut self) {
        match self.options.sort {
            Sort::Count => {
                // we'll try to keep the same path selected
                let selected_path = self.selected_line().path.to_path_buf();
                self.lines[1..].sort_by(|a, b| {
                    let acount = a.sum.map_or(0, |s| s.to_count());
                    let bcount = b.sum.map_or(0, |s| s.to_count());
                    bcount.cmp(&acount)
                });
                self.try_select_path(&selected_path);
            }
            Sort::Date => {
                let selected_path = self.selected_line().path.to_path_buf();
                self.lines[1..].sort_by(|a, b| {
                    let adate = a.sum.map_or(0, |s| s.to_seconds());
                    let bdate = b.sum.map_or(0, |s| s.to_seconds());
                    bdate.cmp(&adate)
                });
                self.try_select_path(&selected_path);
            }
            Sort::Size => {
                let selected_path = self.selected_line().path.to_path_buf();
                self.lines[1..].sort_by(|a, b| {
                    let asize = a.sum.map_or(0, |s| s.to_size());
                    let bsize = b.sum.map_or(0, |s| s.to_size());
                    bsize.cmp(&asize)
                });
                self.try_select_path(&selected_path);
            }
            _ => {}
        }
    }

    /// compute and return the size of the root
    pub fn total_sum(&self) -> FileSum {
        if let Some(sum) = self.lines[0].sum {
            // if the real total sum is computed, it's in the root line
            sum
        } else {
            // if we don't have the sum in root, the nearest estimate is
            // the sum of sums of lines at depth 1
            let mut sum = FileSum::zero();
            for i in 1..self.lines.len() {
                if self.lines[i].depth == 1 {
                    if let Some(line_sum) = self.lines[i].sum {
                        sum += line_sum;
                    }
                }
            }
            sum
        }
    }
}

