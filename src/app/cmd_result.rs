use {
    super::*,
    crate::{
        browser::BrowserState,
        errors::TreeBuildError,
        launchable::Launchable,
    },
    std::fmt,
};

/// Result of applying a command to a state
pub enum AppStateCmdResult {
    Quit,
    Keep,
    Launch(Box<Launchable>),
    DisplayError(String),
    NewPanel {
        state: Box<dyn AppState>,
        purpose: PanelPurpose,
    },
    NewState(Box<dyn AppState>),
    PopStateAndReapply, // the state asks the command be executed on a previous state
    PopState,
    ClosePanel {
        validate_purpose: bool,
    },
    RefreshState {
        clear_cache: bool,
    },
}

impl AppStateCmdResult {
    pub fn verb_not_found(text: &str) -> AppStateCmdResult {
        AppStateCmdResult::DisplayError(format!("verb not found: {:?}", &text))
    }
    pub fn from_optional_state(
        os: Result<Option<BrowserState>, TreeBuildError>,
        in_new_panel: bool,
    ) -> AppStateCmdResult {
        match os {
            Ok(Some(os)) => {
                if in_new_panel {
                    AppStateCmdResult::NewPanel {
                        state: Box::new(os),
                        purpose: PanelPurpose::None,
                    }
                } else {
                    AppStateCmdResult::NewState(Box::new(os))
                }
            }
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

impl fmt::Debug for AppStateCmdResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AppStateCmdResult::Quit => "Quit",
                AppStateCmdResult::Keep => "Keep",
                AppStateCmdResult::Launch(_) => "Launch",
                AppStateCmdResult::DisplayError(_) => "DisplayError",
                AppStateCmdResult::NewState { .. } => "NewState",
                AppStateCmdResult::NewPanel { .. } => "NewPanel",
                AppStateCmdResult::PopStateAndReapply => "PopStateAndReapply",
                AppStateCmdResult::PopState => "PopState",
                AppStateCmdResult::ClosePanel {
                    validate_purpose: false,
                } => "CancelPanel",
                AppStateCmdResult::ClosePanel {
                    validate_purpose: true,
                } => "OkPanel",
                AppStateCmdResult::RefreshState { .. } => "RefreshState",
            }
        )
    }
}
