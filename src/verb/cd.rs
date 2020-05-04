use super::{ExternalExecution, ExternalExecutionMode};

lazy_static! {
    pub static ref CD: ExternalExecution = ExternalExecution::new(
        "cd",
        "cd {directory}",
        ExternalExecutionMode::FromParentShell,
    )
    .unwrap();
}
