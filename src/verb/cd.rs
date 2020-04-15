
use {
    super::{
        External,
        ExternalExecutionMode,
    },
};

lazy_static! {
    pub static ref CD: External = External::new(
        "cd",
        "cd {directory}",
        ExternalExecutionMode::FromParentShell,
    ).unwrap();
}

