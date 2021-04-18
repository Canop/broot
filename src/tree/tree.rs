use {
    super::*,
    crate::{
        app::AppContext,
        errors,
        file_sum::FileSum,
        git::TreeGitStatus,
        task_sync::ComputationResult,
        task_sync::Dam,
        tree_build::TreeBuilder,
    },
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
    pub scroll: i32, // the number of lines at the top hidden because of scrolling
    pub nb_gitignored: u32, // number of times a gitignore pattern excluded a file
    pub total_search: bool, // whether the search was made on all children
    pub git_status: ComputationResult<TreeGitStatus>,
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
            .build(
                false, // on refresh we always do a non total search
                &Dam::unlimited(),
            )
            .unwrap(); // should not fail
                       // we save the old selection to try restore it
        let selected_path = self.selected_line().path.to_path_buf();
        mem::swap(&mut self.lines, &mut tree.lines);
        self.scroll = 0;
        if !self.try_select_path(&selected_path) {
            if self.selection >= self.lines.len() {
                self.selection = 0;
            }
        }
        self.make_selection_visible(page_height as i32);
        Ok(())
    }

    /// do what must be done after line additions or removals:
    /// - sort the lines
    /// - compute left branchs
    pub fn after_lines_changed(&mut self) {
        // we sort the lines (this is mandatory to avoid crashes)
        self.lines[1..].sort();

        let mut best_index = 0; // index of the line with the best score
        for i in 1..self.lines.len() {
            if self.lines[i].score > self.lines[best_index].score {
                best_index = i;
            }
            for d in 0..self.lines[i].left_branchs.len() {
                self.lines[i].left_branchs[d] = false;
            }
        }
        // then we discover the branches (for the drawing)
        // and we mark the last children as pruning, if they have unlisted brothers
        let mut last_parent_index: usize = self.lines.len() + 1;
        for end_index in (1..self.lines.len()).rev() {
            let depth = (self.lines[end_index].depth - 1) as usize;
            let start_index = {
                let parent_index = {
                    let parent_path = &self.lines[end_index].path.parent();
                    match parent_path {
                        Some(parent_path) => {
                            let mut index = end_index;
                            loop {
                                index -= 1;
                                if self.lines[index].path == *parent_path {
                                    break;
                                }
                                if index == 0 {
                                    break;
                                }
                            }
                            index
                        }
                        None => end_index, // Should not happen
                    }
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
                self.lines[i].left_branchs[depth] = true;
            }
        }
        if self.options.needs_sum() {
            time!("fetch_file_sum", self.fetch_regular_file_sums()); // not the dirs, only simple files
            self.sort_siblings(); // does nothing when sort mode is None
        }
    }

    pub fn has_branch(&self, line_index: usize, depth: usize) -> bool {
        if line_index >= self.lines.len() {
            return false;
        }
        let line = &self.lines[line_index];
        depth < usize::from(line.depth) && line.left_branchs[depth]
    }

    /// select another line
    ///
    /// For example the following one if dy is 1.
    pub fn move_selection(&mut self, dy: i32, page_height: i32, cycle: bool) {
        // FIXME may not work well if dy is too big
        let l = self.lines.len() as i32;
        loop {
            if !cycle {
                let s = dy + (self.selection as i32);
                if s < 0 || s >= l {
                    break;
                }
            }
            self.selection = (self.selection + (l + dy) as usize) % self.lines.len();
            if self.lines[self.selection].is_selectable() {
                break;
            }
        }
        // we adjust the scroll
        let sel = self.selection as i32;
        if l > page_height {
            if dy < 0 {
                if sel == l - 1 {
                    // cycling
                    self.scroll = l - page_height;
                } else if sel < self.scroll + 5 {
                    self.scroll = (self.scroll + 2 * dy).max(0);
                }
            } else {
                if sel == 0 {
                    // cycling brought us back to top
                    self.scroll = 0;
                } else if sel > self.scroll + page_height - 5 {
                    self.scroll = (self.scroll + 2 * dy).min(l - page_height);
                }
            }
        }
    }

    pub fn try_scroll(&mut self, dy: i32, page_height: i32) {
        self.scroll = (self.scroll + dy).max(0).min(self.lines.len() as i32 - 5);
        self.select_visible_line(page_height);
    }

    /// try to select a line (works if y+scroll falls on a selectable line)
    pub fn try_select_y(&mut self, y: i32) -> bool {
        let y = y + self.scroll;
        if y >= 0 && y < self.lines.len() as i32 {
            let y = y as usize;
            if self.lines[y].is_selectable() {
                self.selection = y;
                return true;
            }
        }
        false
    }
    /// fix the selection so that it's a selectable visible line
    fn select_visible_line(&mut self, page_height: i32) {
        let sel = self.selection as i32;
        if sel < self.scroll || sel >= self.scroll + page_height {
            self.selection = self.scroll as usize;
            let l = self.lines.len();
            loop {
                self.selection = (self.selection + ((l as i32) + 1) as usize) % l;
                if self.lines[self.selection].is_selectable() {
                    break;
                }
            }
        }
    }

    pub fn make_selection_visible(&mut self, page_height: i32) {
        let sel = self.selection as i32;
        let l = self.lines.len() as i32;
        if sel < self.scroll {
            self.scroll = (self.selection as i32 - 2).max(0);
        } else if l > page_height && sel >= self.scroll + page_height {
            self.scroll = (self.selection as i32 - page_height + 2) as i32;
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
    pub fn try_select_last(&mut self, page_height: i32) -> bool {
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
    pub fn try_select_next_match(&mut self) -> bool {
        for di in 0..self.lines.len() {
            let idx = (self.selection + di + 1) % self.lines.len();
            let line = &self.lines[idx];
            if !line.is_selectable() {
                continue;
            }
            if !line.direct_match {
                continue;
            }
            if line.score > 0 {
                self.selection = idx;
                return true;
            }
        }
        false
    }
    pub fn try_select_previous_same_depth(&mut self) -> bool {
        let depth = self.lines[self.selection].depth;
        for di in (0..self.lines.len()).rev() {
            let idx = (self.selection + di) % self.lines.len();
            let line = &self.lines[idx];
            if !line.is_selectable() || line.depth != depth {
                continue;
            }
            self.selection = idx;
            return true;
        }
        false
    }
    pub fn try_select_next_same_depth(&mut self) -> bool {
        let depth = self.lines[self.selection].depth;
        for di in 0..self.lines.len() {
            let idx = (self.selection + di + 1) % self.lines.len();
            let line = &self.lines[idx];
            if !line.is_selectable() || line.depth != depth {
                continue;
            }
            self.selection = idx;
            return true;
        }
        false
    }
    pub fn try_select_previous_match(&mut self) -> bool {
        for di in (0..self.lines.len()).rev() {
            let idx = (self.selection + di) % self.lines.len();
            let line = &self.lines[idx];
            if !line.is_selectable() {
                continue;
            }
            if line.score > 0 {
                self.selection = idx;
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
        if !self.options.sort.is_some() {
            return;
        }
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
            Sort::None => {}
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

