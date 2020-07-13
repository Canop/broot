use {
    super::*,
    crate::{
        browser::BrowserState,
        errors::TreeBuildError,
        launchable::Launchable,
        verb::Internal,
    },
    std::fmt,
};

/// Either left or right
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HDir {
    Left,
    Right,
}

/// Result of applying a command to a state
pub enum AppStateCmdResult {
    ClosePanel {
        validate_purpose: bool,
        id: Option<PanelId>, // None if current panel
    },
    DisplayError(String),
    Keep,
    Launch(Box<Launchable>),
    NewPanel {
        state: Box<dyn AppState>,
        purpose: PanelPurpose,
        direction: HDir,
    },
    NewState(Box<dyn AppState>),
    PopStateAndReapply, // the state asks the command be executed on a previous state
    PopState,
    Propagate(Internal), // command must be handled at a upper level
    Quit,
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
                        direction: HDir::Right,
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
                AppStateCmdResult::ClosePanel {
                    validate_purpose: false, ..
                } => "CancelPanel",
                AppStateCmdResult::ClosePanel {
                    validate_purpose: true, ..
                } => "OkPanel",
                AppStateCmdResult::DisplayError(_) => "DisplayError",
                AppStateCmdResult::Keep => "Keep",
                AppStateCmdResult::Launch(_) => "Launch",
                AppStateCmdResult::NewState { .. } => "NewState",
                AppStateCmdResult::NewPanel { .. } => "NewPanel",
                AppStateCmdResult::PopStateAndReapply => "PopStateAndReapply",
                AppStateCmdResult::PopState => "PopState",
                AppStateCmdResult::Propagate(_) => "Propagate",
                AppStateCmdResult::Quit => "Quit",
                AppStateCmdResult::RefreshState { .. } => "RefreshState",
            }
        )
    }
}
