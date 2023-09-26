use {
    super::*,
    crate::{
        pattern::*,
        verb::{Internal, VerbInvocation, VerbId},
    },
    bet::BeTree,
};

/// a command which may result in a change in the application state.
///
/// It may come from a shortcut, from the parsed input, from an argument
/// given on launch.
#[derive(Debug, Clone)]
pub enum Command {

    /// no command
    None,

    /// a verb invocation, unfinished
    /// (user didn't hit enter)
    VerbEdit(VerbInvocation),

    /// verb invocation, finished
    /// (coming from --cmd, or after the user hit enter)
    VerbInvocate(VerbInvocation),

    /// call of an internal done without the input
    /// (using a trigger key for example)
    Internal {
        internal: Internal,
        input_invocation: Option<VerbInvocation>,
    },

    /// call of a verb done without the input
    /// (using a trigger key for example)
    VerbTrigger {
        verb_id: VerbId,
        input_invocation: Option<VerbInvocation>,
    },

    /// a pattern being edited
    PatternEdit {
        raw: String,
        expr: BeTree<PatternOperator, PatternParts>,
    },

    /// a mouse click
    Click(u16, u16),

    /// a mouse double-click
    /// Always come after a simple click at same position
    DoubleClick(u16, u16),
}

impl Command {

    pub fn empty() -> Command {
        Command::None
    }

    pub fn as_verb_invocation(&self) -> Option<&VerbInvocation> {
        match self {
            Self::VerbEdit(vi) => Some(vi),
            Self::VerbInvocate(vi) => Some(vi),
            Self::Internal { input_invocation, .. } =>  input_invocation.as_ref(),
            Self::VerbTrigger { input_invocation, .. } =>  input_invocation.as_ref(),
            _ => None,
        }
    }

    /// build a command from the parsed string representation
    ///
    /// The command being finished is the difference between
    /// a command being edited and a command launched (which
    /// happens on enter in the input).
    pub fn from_parts(mut cp: CommandParts, finished: bool) -> Self {
        if let Some(verb_invocation) = cp.verb_invocation.take() {
            if finished {
                Self::VerbInvocate(verb_invocation)
            } else {
                Self::VerbEdit(verb_invocation)
            }
        } else if finished {
            Self::Internal {
                internal: Internal::open_stay,
                input_invocation: None,
            }
        } else {
            Self::PatternEdit {
                raw: cp.raw_pattern,
                expr: cp.pattern,
            }
        }
    }

    /// tells whether this action is a verb being invocated on enter
    /// in the input field
    pub fn is_verb_invocated_from_input(&self) -> bool {
        matches!(self, Self::VerbInvocate(_))
    }

    /// create a command from a raw input.
    ///
    /// `finished` makes the command an executed form,
    /// it's equivalent to using the Enter key in the Gui.
    pub fn from_raw(raw: String, finished: bool) -> Self {
        let parts = CommandParts::from(raw);
        Self::from_parts(parts, finished)
    }

    /// build a non executed command from a pattern
    pub fn from_pattern(pattern: &InputPattern) -> Self {
        Command::from_raw(pattern.raw.clone(), false)
    }
}

impl Default for Command {
    fn default() -> Command {
        Command::empty()
    }
}
