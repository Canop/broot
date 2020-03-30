use {
    crate::{
        verb_invocation::VerbInvocation,
    },
    super::*,
};


#[derive(Debug, Clone)]
pub enum Action {
    Unparsed,
    MoveSelection(i32),           // up (neg) or down (positive) in the list
    OpenSelection,                // open the selected line
    AltOpenSelection,             // alternate open the selected line
    VerbEdit(VerbInvocation),     // verb invocation, unfinished
    VerbInvocate(VerbInvocation), // verb invocation, after the user hit enter
    VerbIndex(usize),             // verb call, withtout specific argument (using a trigger key)
    FuzzyPatternEdit(String),     // a pattern being edited
    RegexEdit(String, String),    // a regex being edited (core & flags)
    Back,                         // back to last app state, or clear pattern
    Next,                         // goes to the next matching entry
    Previous,                     // goes to the previous matching entry
    Help,                         // goes to help state
    Click(u16, u16),              // usually a mouse click
    DoubleClick(u16, u16),        // always come after a simple click at same position
    Resize(u16, u16),             // terminal was resized to those dimensions
}

impl Action {
    pub fn from(cp: &CommandParts, finished: bool) -> Action {
        if let Some(verb_invocation) = &cp.verb_invocation {
            if finished {
                Action::VerbInvocate(verb_invocation.clone())
            } else {
                Action::VerbEdit(verb_invocation.clone())
            }
        } else if finished {
            Action::OpenSelection
        } else if let Some(pattern) = &cp.pattern {
            let pattern = String::from(pattern.as_str());
            if let Some(regex_flags) = &cp.regex_flags {
                Action::RegexEdit(pattern, String::from(regex_flags.as_str()))
            } else {
                Action::FuzzyPatternEdit(String::from(pattern.as_str()))
            }
        } else {
            Action::FuzzyPatternEdit(String::from(""))
        }
    }
}

