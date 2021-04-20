use {
    super::*,
    crate::{
        pattern::*,
    },
    std::{
        path::PathBuf,
    },
};

#[derive(Clone)]
pub struct FilteredStage {
    stage_version: usize,
    paths_idx: Vec<usize>, // indexes of the matching paths in the stage
    pattern: InputPattern, // an optional filtering pattern
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
        }
    }
    pub fn update(&mut self, stage: &Stage) -> bool {
        if stage.version() == self.stage_version {
            false
        } else {
            self.paths_idx = Self::compute(stage, &self.pattern.pattern);
            true
        }
    }
    pub fn len(&self) -> usize {
        self.paths_idx.len()
    }
    pub fn path<'s>(&self, stage: &'s Stage, idx: usize) -> Option<&'s PathBuf> {
        self.paths_idx
            .get(idx)
            .and_then(|&idx| stage.paths().get(idx))
    }
    pub fn pattern(&self) -> &InputPattern {
        &self.pattern
    }
}
