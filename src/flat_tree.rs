//! in the flat_tree structure, every "node" is just a line, there's
//!  no link from a child to its parent or from a parent to its childs.

use std::cmp::{self, Ordering};
use std::path::PathBuf;

use crate::file_sizes::Size;
use crate::task_sync::TaskLifetime;
use crate::tree_options::TreeOptions;

#[derive(Debug, Clone, PartialEq)]
pub enum LineType {
    File,
    Dir,
    SymLink(String), // store the lineType of destination ?
    Pruning,
}

#[derive(Debug)]
pub struct TreeLine {
    pub left_branchs: Box<[bool]>,
    pub depth: u16,
    pub name: String, // name of the first unlisted, in case of Pruning
    pub path: PathBuf,
    pub content: LineType, // FIXME rename
    pub has_error: bool,
    pub unlisted: usize, // number of not listed childs (Dir) or brothers (Pruning)
    pub score: i32,      // 0 if there's no pattern
    pub size: Option<Size>, // None when not measured
}

#[derive(Debug)]
pub struct Tree {
    pub lines: Box<[TreeLine]>,
    pub selection: usize, // there's always a selection (starts with root, which is 0)
    pub options: TreeOptions,
    pub scroll: i32, // FIXME usize
}

impl TreeLine {
    pub fn is_selectable(&self) -> bool {
        match &self.content {
            LineType::Pruning => false,
            _ => true,
        }
    }
    pub fn is_dir(&self) -> bool {
        match &self.content {
            LineType::Dir => true,
            _ => false,
        }
    }
    pub fn is_file(&self) -> bool {
        match &self.content {
            LineType::File => true,
            _ => false,
        }
    }
}
impl PartialEq for TreeLine {
    fn eq(&self, other: &TreeLine) -> bool {
        self.path == other.path
    }
}

impl Eq for TreeLine {}

impl Ord for TreeLine {
    // paths are sorted in a complete ignore case way
    // (A<a<B<b)
    fn cmp(&self, other: &TreeLine) -> Ordering {
        let mut sci = self.path.components();
        let mut oci = other.path.components();
        loop {
            match sci.next() {
                Some(sc) => {
                    match oci.next() {
                        Some(oc) => {
                            let scs = sc.as_os_str().to_string_lossy();
                            let ocs = oc.as_os_str().to_string_lossy();
                            let lower_ordering = scs.to_lowercase().cmp(&ocs.to_lowercase());
                            if lower_ordering != Ordering::Equal {
                                return lower_ordering;
                            }
                            let ordering = scs.cmp(&ocs);
                            if ordering != Ordering::Equal {
                                return ordering;
                            }
                        }
                        None => {
                            return Ordering::Greater;
                        }
                    };
                }
                None => {
                    if oci.next().is_some() {
                        return Ordering::Less;
                    } else {
                        return Ordering::Equal;
                    }
                }
            };
        }
    }
}

impl PartialOrd for TreeLine {
    fn partial_cmp(&self, other: &TreeLine) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Tree {
    // do what must be done after line additions or removals:
    // - sort the lines
    // - compute left branchs
    pub fn after_lines_changed(&mut self) {
        // we sort the lines
        self.lines.sort();

        for i in 1..self.lines.len() {
            for d in 0..self.lines[i].left_branchs.len() {
                self.lines[i].left_branchs[d] = false;
            }
        }
        // then we discover the branches (for the drawing)
        // and we mark the last childs as pruning, if they have unlisted brothers
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
                    if unlisted > 0 {
                        self.lines[end_index].content = LineType::Pruning;
                        self.lines[end_index].unlisted = unlisted + 1;
                        self.lines[parent_index].unlisted = 0;
                    }
                    last_parent_index = parent_index;
                }
                parent_index + 1
            };
            for i in start_index..=end_index {
                self.lines[i].left_branchs[depth] = true;
            }
        }
    }
    pub fn has_branch(&self, line_index: usize, depth: usize) -> bool {
        if line_index >= self.lines.len() {
            return false;
        }
        let line = &self.lines[line_index];
        match depth >= line.depth as usize {
            true => false,
            false => line.left_branchs[depth],
        }
    }
    pub fn move_selection(&mut self, dy: i32, page_height: i32) {
        // only work for +1 or -1
        let l = self.lines.len();
        loop {
            self.selection = (self.selection + ((l as i32) + dy) as usize) % l;
            if self.lines[self.selection].is_selectable() {
                break;
            }
        }
        // we adjust the scroll
        let l = l as i32;
        let sel = self.selection as i32;
        if dy < 0 && sel < self.scroll + 5 {
            self.scroll = (self.scroll + 2 * dy).max(0);
        } else if dy > 0 && l > page_height && sel > self.scroll + page_height - 5 {
            self.scroll = self.scroll + 2 * dy;
        }
    }
    pub fn try_scroll(&mut self, dy: i32, page_height: i32) {
        self.scroll = (self.scroll + dy).max(0).min(self.lines.len() as i32 - 5);
        self.select_visible_line(page_height);
    }
    pub fn select_visible_line(&mut self, page_height: i32) {
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
    // select the line with the best matching score
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
    pub fn try_select_next_match(&mut self) -> bool {
        for di in 0..self.lines.len() {
            let idx = (self.selection + di + 1) % self.lines.len();
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
    pub fn has_dir_missing_size(&self) -> bool {
        if !self.options.show_sizes {
            return false;
        }
        for i in 1..self.lines.len() {
            if self.lines[i].size.is_none() && self.lines[i].is_dir() {
                return true;
            }
        }
        false
    }
    pub fn fetch_file_sizes(&mut self) {
        for i in 1..self.lines.len() {
            if self.lines[i].is_file() {
                self.lines[i].size = Some(Size::from_file(&self.lines[i].path));
            }
        }
    }
    pub fn fetch_some_missing_dir_size(&mut self, tl: &TaskLifetime) {
        for i in 1..self.lines.len() {
            if self.lines[i].size.is_none() && self.lines[i].is_dir() {
                self.lines[i].size = Size::from_dir(&self.lines[i].path, tl);
                return;
            }
        }
    }
    pub fn total_size(&self) -> Size {
        if let Some(size) = self.lines[0].size {
            // if the real total size is computed, it's in the root line
            size
        } else {
            // if we don't have the size in root, the nearest estimate is
            // the sum of sizes of lines at depth 1
            let mut sum = Size::from(0);
            for i in 1..self.lines.len() {
                if self.lines[i].depth == 1 {
                    if let Some(size) = self.lines[i].size {
                        sum += size;
                    }
                }
            }
            sum
        }
    }
}
