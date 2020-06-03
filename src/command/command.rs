use {
    super::*,
    crate::{
        pattern::Pattern,
        verb::{Internal, VerbInvocation},
    },
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
        index: usize,
        input_invocation: Option<VerbInvocation>,
    },

    /// a pattern being edited
    PatternEdit(PatternParts),

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

    /// build a command from the parsed string representation
    ///
    /// The command being finished is the difference between
    /// a command being edited and a command launched (which
    /// happens on enter in the input).
    pub fn from_parts(cp: &CommandParts, finished: bool) -> Self {
        if let Some(verb_invocation) = &cp.verb_invocation {
            if finished {
                Self::VerbInvocate(verb_invocation.clone())
            } else {
                Self::VerbEdit(verb_invocation.clone())
            }
        } else if finished {
            Self::Internal {
                internal: Internal::open_stay,
                input_invocation: None,
            }
        } else if let Some(pattern) = &cp.pattern {
            Self::PatternEdit(pattern.clone())
        } else {
            Self::PatternEdit(PatternParts::default())
        }
    }


    /// tells whether this action is a verb being invocated on enter
    /// in the input field
    pub fn is_verb_invocated_from_input(&self) -> bool {
        match self {
            Self::VerbInvocate(_) => true,
            _ => false,
        }
    }

    /// create a command from a raw input.
    ///
    /// `finished` makes the command an executed form,
    /// it's equivalent to using the Enter key in the Gui.
    pub fn from_raw(raw: String, finished: bool) -> Self {
        let parts = CommandParts::from(&raw);
        Self::from_parts(&parts, finished)
    }

    /// build a non executed command from a pattern
    pub fn from_pattern(pattern: &Pattern) -> Self {
        Command::from_raw(
            pattern.as_input(),
            false,
        )
    }
}

impl Default for Command {
    fn default() -> Command {
        Command::empty()
    }
}
