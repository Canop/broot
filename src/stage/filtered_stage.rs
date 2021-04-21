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
    fn compute(stage: &Stage, pattern: &Pattern) -> Vec<usize> {
        stage.paths().iter()
            .enumerate()
            .filter(|(_, path)| {
                if pattern.is_none() {
                    true
                } else {
                    path.file_name()
                        .map(|file_name| {
                            let subpath = path.to_string_lossy().to_string();
                            let name = file_name.to_string_lossy().to_string();
                            let regular_file = path.is_file();
                            let candidate = Candidate {
                                path,
                                subpath: &subpath,
                                name: &name,
                                regular_file,
                            };
                            pattern.score_of(candidate).is_some()
                        })
                        .unwrap_or(false)
                }
            })
            .map(|(idx, _)| idx)
            .collect()
    }
    pub fn filtered(stage: &Stage, pattern: InputPattern) -> Self {
        Self {
            stage_version: stage.version(),
            paths_idx: Self::compute(stage, &pattern.pattern),
            pattern,
            selection: None,
        }
    }
    /// chech whether the stage has changed, and update the
    /// filtered list if necessary
    pub fn update(&mut self, stage: &Stage) -> bool {
        if stage.version() == self.stage_version {
            false
        } else {
            debug!("filtering stage");
            let selected_path_before = self.selection
                .map(|idx| &stage.paths()[self.paths_idx[idx]]);
            self.paths_idx = Self::compute(stage, &self.pattern.pattern);
            self.selection = selected_path_before
                .and_then(|p| {
                    self.paths_idx.iter()
                        .position(|&pi| p==&stage.paths()[self.paths_idx[pi]])
                });
            true
        }
    }
    /// change the pattern, keeping the selection if possible
    /// Assumes the stage didn't change (if it changed, we lose the
    /// selection)
    pub fn set_pattern(&mut self, stage: &Stage, pattern: InputPattern) {
        let selected_idx_before = self.selection
            .filter(|_| self.stage_version == stage.version())
            .map(|idx| self.paths_idx[idx]);
        self.stage_version = stage.version(); // in case it changed
        self.pattern = pattern;
        self.paths_idx = Self::compute(stage, &self.pattern.pattern);
        self.selection = selected_idx_before
            .and_then(|pi| self.paths_idx.iter().position(|&v| v==pi));
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
            self.paths_idx = Self::compute(stage, &self.pattern.pattern);
            if spi >= self.paths_idx.len() {
                self.selection = None;
            }
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
        } else {
            Some(0)
        };
    }
}
