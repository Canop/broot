use {
    crate::{
        stage::Stage,
    },
};


/// global mutable state
#[derive(Debug, Default)]
pub struct AppState {
    pub stage: Stage,
}

impl AppState {
}
