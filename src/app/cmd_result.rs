use {
    super::*,
    crate::{
        browser::BrowserState,
        command::Sequence,
        display::LayoutInstruction,
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
pub enum CmdResult {
    ApplyOnPanel {
        id: PanelId,
    },
    ClosePanel {
        validate_purpose: bool,
        panel_ref: PanelReference,
        clear_cache: bool,
    },
    ChangeLayout(LayoutInstruction),
    DisplayError(String),
    ExecuteSequence {
        sequence: Sequence,
    },
    HandleInApp(Internal), // command must be handled at the app level
    Keep,
    Message(String),
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
    #[must_use]
    pub fn verb_not_found(text: &str) -> CmdResult {
        CmdResult::DisplayError(format!("verb not found: {:?}", &text))
    }
    #[must_use]
    pub fn from_optional_browser_state(
        os: Result<BrowserState, TreeBuildError>,
        message: Option<&'static str>,
        in_new_panel: bool,
    ) -> CmdResult {
        match os {
            Ok(os) => {
                if in_new_panel {
                    CmdResult::NewPanel {
                        // TODO keep the message ?
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
    #[must_use]
    pub fn new_state(state: Box<dyn PanelState>) -> Self {
        Self::NewState {
            state,
            message: None,
        }
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
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        match self {
            CmdResult::ApplyOnPanel { id } => f
                .debug_struct("CmdResult::ApplyOnPanel")
                .field("id", id)
                .finish(),
            CmdResult::ClosePanel {
                validate_purpose,
                panel_ref,
                clear_cache,
            } => f
                .debug_struct("CmdResult::ClosePanel")
                .field("validate_purpose", validate_purpose)
                .field("panel_ref", panel_ref)
                .field("clear_cache", clear_cache)
                .finish(),
            CmdResult::ChangeLayout(layout_instruction) => f
                .debug_tuple("CmdResult::ChangeLayout")
                .field(layout_instruction)
                .finish(),
            CmdResult::DisplayError(message) => f
                .debug_tuple("CmdResult::DisplayError")
                .field(message)
                .finish(),
            CmdResult::ExecuteSequence { sequence } => f
                .debug_struct("CmdResult::ExecuteSequence")
                .field("sequence", sequence)
                .finish(),
            CmdResult::HandleInApp(internal) => f
                .debug_tuple("CmdResult::HandleInApp")
                .field(internal)
                .finish(),
            CmdResult::Keep => write!(f, "CmdResult::Keep"),
            CmdResult::Message(message) => f
                .debug_tuple("CmdResult::Message")
                .field(message)
                .finish(),
            CmdResult::Launch(launchable) => f
                .debug_tuple("CmdResult::Launch")
                .field(launchable)
                .finish(),
            CmdResult::NewPanel {
                state: _,
                purpose,
                direction,
            } => f
                .debug_struct("CmdResult::NewPanel")
                .field("purpose", purpose)
                .field("direction", direction)
                .finish_non_exhaustive(),

            CmdResult::NewState { state: _, message } => f
                .debug_struct("CmdResult::NewState")
                .field("message", message)
                .finish_non_exhaustive(),
            CmdResult::PopStateAndReapply => write!(f, "CmdResult::PopStateAndReapply"),
            CmdResult::PopState => write!(f, "CmdResult::PopState"),
            CmdResult::Quit => write!(f, "CmdResult::Quit"),
            CmdResult::RefreshState { clear_cache } => f
                .debug_struct("CmdResult::RefreshState")
                .field("clear_cache", clear_cache)
                .finish(),
        }
    }
}
