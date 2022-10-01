use {
    super::*,
    crate::{
        browser::BrowserState,
        command::Sequence,
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

/// the symbolic reference to the panel to close
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PanelReference {
    Active,
    Leftest,
    Rightest,
    Id(PanelId),
    Preview,
}

/// Result of applying a command to a state
pub enum CmdResult {
    ApplyOnPanel {
        id: PanelId,
    },
    ClosePanel {
        validate_purpose: bool,
        panel_ref: PanelReference,
    },
    DisplayError(String),
    ExecuteSequence {
        sequence: Sequence,
    },
    HandleInApp(Internal), // command must be handled at the app level
    Keep,
    Launch(Box<Launchable>),
    NewPanel {
        state: Box<dyn PanelState>,
        purpose: PanelPurpose,
        direction: HDir,
    },
    NewState {
        state: Box<dyn PanelState>,
        message: Option<&'static str>, // explaining why there's a new state
    },
    PopStateAndReapply, // the state asks the command be executed on a previous state
    PopState,
    Quit,
    RefreshState {
        clear_cache: bool,
    },
}

impl CmdResult {
    pub fn verb_not_found(text: &str) -> CmdResult {
        CmdResult::DisplayError(format!("verb not found: {:?}", &text))
    }
    pub fn from_optional_state(
        os: Result<BrowserState, TreeBuildError>,
        message: Option<&'static str>,
        in_new_panel: bool,
    ) -> CmdResult {
        match os {
            Ok(os) => {
                if in_new_panel {
                    CmdResult::NewPanel { // TODO keep the message ?
                        state: Box::new(os),
                        purpose: PanelPurpose::None,
                        direction: HDir::Right,
                    }
                } else {
                    CmdResult::NewState {
                        state: Box::new(os),
                        message,
                    }
                }
            }
            Err(TreeBuildError::Interrupted) => CmdResult::Keep,
            Err(e) => CmdResult::error(e.to_string()),
        }
    }
    pub fn new_state(state: Box<dyn PanelState>) -> Self {
        Self::NewState { state, message: None }
    }
    pub fn error<S: Into<String>>(message: S) -> Self {
        Self::DisplayError(message.into())
    }
}

impl From<Launchable> for CmdResult {
    fn from(launchable: Launchable) -> Self {
        CmdResult::Launch(Box::new(launchable))
    }
}

impl fmt::Debug for CmdResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CmdResult::ApplyOnPanel { .. } => "ApplyOnPanel",
                CmdResult::ClosePanel {
                    validate_purpose: false, ..
                } => "CancelPanel",
                CmdResult::ClosePanel {
                    validate_purpose: true, ..
                } => "OkPanel",
                CmdResult::DisplayError(_) => "DisplayError",
                CmdResult::ExecuteSequence{ .. } => "ExecuteSequence",
                CmdResult::Keep => "Keep",
                CmdResult::Launch(_) => "Launch",
                CmdResult::NewState { .. } => "NewState",
                CmdResult::NewPanel { .. } => "NewPanel",
                CmdResult::PopStateAndReapply => "PopStateAndReapply",
                CmdResult::PopState => "PopState",
                CmdResult::HandleInApp(_) => "HandleInApp",
                CmdResult::Quit => "Quit",
                CmdResult::RefreshState { .. } => "RefreshState",
            }
        )
    }
}
