use {
    super::*,
    crate::{
        app::AppContext,
        file_sum::FileSum,
        task_sync::Dam,
    },
};

#[derive(Clone, Copy, Default)]
pub struct StageSum {
    stage_version: usize,
    sum: Option<FileSum>,
}

impl StageSum {
    /// invalidates the computed sum if the version at compilation
    /// time is older than the current one
    pub fn see_stage(&mut self, stage: &Stage) {
        if stage.version() != self.stage_version {
            self.sum = None;
        }
    }
    pub fn is_up_to_date(&self) -> bool {
        self.sum.is_some()
    }
    pub fn clear(&mut self) {
        self.sum = None;
    }
    pub fn compute(&mut self, stage: &Stage, dam: &Dam, con: &AppContext) -> Option<FileSum> {
        if self.stage_version != stage.version() {
            self.sum = None;
        }
        self.stage_version = stage.version();
        if self.sum.is_none() {
            // produces None in case of interruption
            self.sum = stage.compute_sum(dam, con);
        }
        self.sum
    }
    pub fn computed(&self) -> Option<FileSum> {
        self.sum
    }
}
