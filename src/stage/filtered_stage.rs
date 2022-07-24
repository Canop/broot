use {
    super::*,
    crate::{
        pattern::*,
    },
    std::{
        convert::TryFrom,
        path::Path,
    },
};

#[derive(Clone)]
pub struct FilteredStage {
    stage_version: usize,
    paths_idx: Vec<usize>, // indexes of the matching paths in the stage
    pattern: InputPattern, // an optional filtering pattern
    selection: Option<usize>, // index in paths_idx, always in [0, paths_idx.len()[
}

impl FilteredStage {
    pub fn unfiltered(stage: &Stage) -> Self {
        Self::filtered(stage, InputPattern::none())
    }
    /// compute the paths_idx and maybe change the selection
    fn compute(&mut self, stage: &Stage) {
        if self.pattern.is_none() {
            self.paths_idx = stage.paths().iter()
                .enumerate()
                .map(|(idx, _)| idx)
                .collect();
        } else {
            let mut best_score = None;
            self.paths_idx.clear();
            for (idx, path) in stage.paths().iter().enumerate() {
                if let Some(file_name) = path.file_name() {
                    let subpath = path.to_string_lossy().to_string();
                    let name = file_name.to_string_lossy().to_string();
                    let regular_file = path.is_file();
                    let candidate = Candidate {
                        path,
                        subpath: &subpath,
                        name: &name,
                        regular_file,
                    };
                    if let Some(score) = self.pattern.pattern.score_of(candidate) {
                        let is_best = match best_score {
                            Some(old_score) if old_score < score => true,
                            None => true,
                            _ => false,
                        };
                        if is_best {
                            self.selection = Some(self.paths_idx.len());
                            best_score = Some(score);
                        }
                        self.paths_idx.push(idx);
                    }
                }
            }
        }
    }
    pub fn filtered(stage: &Stage, pattern: InputPattern) -> Self {
        let mut fs = Self {
            stage_version: stage.version(),
            paths_idx: Vec::new(),
            pattern,
            selection: None,
        };
        fs.compute(stage);
        fs
    }
    /// check whether the stage has changed, and update the
    /// filtered list if necessary
    pub fn update(&mut self, stage: &Stage) -> bool {
        if stage.version() == self.stage_version {
            false
        } else {
            self.compute(stage);
            true
        }
    }
    /// change the pattern, keeping the selection if possible
    /// Assumes the stage didn't change (if it changed, we lose the
    /// selection)
    pub fn set_pattern(&mut self, stage: &Stage, pattern: InputPattern) {
        self.stage_version = stage.version(); // in case it changed
        self.pattern = pattern;
        self.compute(stage);
    }
    pub fn len(&self) -> usize {
        self.paths_idx.len()
    }
    pub fn path<'s>(&self, stage: &'s Stage, idx: usize) -> Option<&'s Path> {
        self.paths_idx
            .get(idx)
            .and_then(|&idx| stage.paths().get(idx))
            .map(|p| p.as_path())
    }
    pub fn path_sel<'s>(&self, stage: &'s Stage, idx: usize) -> Option<(&'s Path, bool)> {
        self.path(stage, idx)
            .map(|p| (p, self.selection.map_or(false, |si| idx==si)))
    }
    pub fn pattern(&self) -> &InputPattern {
        &self.pattern
    }
    pub fn selection(&self) -> Option<usize> {
        self.selection
    }
    pub fn has_selection(&self) -> bool {
        self.selection.is_some()
    }
    pub fn try_select_idx(&mut self, idx: usize) -> bool {
        if idx < self.paths_idx.len() {
            self.selection = Some(idx);
            true
        } else {
            false
        }
    }
    pub fn selected_path<'s>(&self, stage: &'s Stage) -> Option<&'s Path> {
        self.selection
            .and_then(|pi| self.paths_idx.get(pi))
            .and_then(|&idx| stage.paths().get(idx))
            .map(|p| p.as_path())
    }
    pub fn unselect(&mut self) {
        self.selection = None
    }
    /// unstage the selection, if any, or return false.
    /// If possible we select the item below so that the user
    /// may easily remove a few items
    pub fn unstage_selection(&mut self, stage: &mut Stage) -> bool {
        if let Some(spi) = self.selection {
            stage.remove_idx(self.paths_idx[spi]);
            self.stage_version = stage.version();
            self.compute(stage);
            if spi >= self.paths_idx.len() {
                self.selection = Some(spi);
            };
            true
        } else {
            false
        }
    }
    pub fn move_selection(&mut self, dy: i32, cycle: bool) {
        self.selection = if self.paths_idx.is_empty() {
            None
        } else if let Some(sel_idx) = self.selection.and_then(|i| i32::try_from(i).ok()) {
            let new_sel_idx = sel_idx + dy;
            Some(
                if new_sel_idx < 0 {
                    if cycle && sel_idx == 0 {
                        self.paths_idx.len() - 1
                    } else {
                        0
                    }
                } else if new_sel_idx as usize >= self.paths_idx.len() {
                    if cycle && sel_idx == self.paths_idx.len() as i32 - 1 {
                        0
                    } else {
                        self.paths_idx.len() - 1
                    }
                } else {
                    new_sel_idx as usize
                }
            )
        } else if dy < 0 {
            Some(self.paths_idx.len() - 1)
        } else {
            Some(0)
        };
    }
}
