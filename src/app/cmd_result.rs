use {
    crate::{
        browser::BrowserState,
        command::Command,
        errors::TreeBuildError,
        launchable::Launchable,
    },
    super::*,
};

/// Result of applying a command to a state
pub enum AppStateCmdResult {
    Quit,
    Keep,
    Launch(Box<Launchable>),
    DisplayError(String),
    NewState { state: Box<dyn AppState>, cmd: Command, in_new_panel: bool },
    PopStateAndReapply, // the state asks the command be executed on a previous state
    PopState,
    PopPanel,
    RefreshState { clear_cache: bool },
}

impl AppStateCmdResult {
    pub fn verb_not_found(text: &str) -> AppStateCmdResult {
        AppStateCmdResult::DisplayError(format!("verb not found: {:?}", &text))
    }
    pub fn from_optional_state(
        os: Result<Option<BrowserState>, TreeBuildError>,
        cmd: Command,
        in_new_panel: bool,
    ) -> AppStateCmdResult {
        match os {
            Ok(Some(os)) => AppStateCmdResult::NewState{
                state: Box::new(os),
                cmd,
                in_new_panel,
            },
            Ok(None) => AppStateCmdResult::Keep,
            Err(e) => AppStateCmdResult::DisplayError(e.to_string()),
        }
    }
}

impl From<Launchable> for AppStateCmdResult {
    fn from(launchable: Launchable) -> Self {
        AppStateCmdResult::Launch(Box::new(launchable))
    }
}
