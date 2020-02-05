//! A command is the parsed representation of what the user types
//!  in the input. It's independant of the state of the application
//!  (verbs arent checked at this point)

use {
    crate::{
        app_context::AppContext, app_state::AppState, keys, patterns::Pattern,
        verb_invocation::VerbInvocation,
    },
    regex::Regex,
    termimad::{Event, InputField},
};

#[derive(Debug, Clone)]
pub struct Command {
    pub raw: String,     // what's visible in the input
    parts: CommandParts, // the parsed parts of the visible input
    pub action: Action, // what's required, based on the last key (which may be not visible, like esc)
}

/// An intermediate parsed representation of the raw string
#[derive(Debug, Clone)]
pub struct CommandParts {
    pattern: Option<String>,     // either a fuzzy pattern or the core of a regex
    regex_flags: Option<String>, // may be Some("") if user asked for a regex but specified no flag
    verb_invocation: Option<VerbInvocation>, // may be empty if user typed the separator but no char after
}

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

impl CommandParts {
    fn new() -> CommandParts {
        CommandParts {
            pattern: None,
            regex_flags: None,
            verb_invocation: None,
        }
    }
    fn from(raw: &str) -> Self {
        let mut cp = CommandParts::new();
        let c = regex!(
            r"(?x)
                ^
                (?P<slash_before>/)?
                (?P<pattern>[^\s/:]+)?
                (?:/(?P<regex_flags>\w*))?
                (?:[\s:]+(?P<verb_invocation>.*))?
                $
            "
        )
        .captures(raw);
        if let Some(c) = c {
            if let Some(pattern) = c.name("pattern") {
                cp.pattern = Some(String::from(pattern.as_str()));
                if let Some(rxf) = c.name("regex_flags") {
                    cp.regex_flags = Some(String::from(rxf.as_str()));
                } else if c.name("slash_before").is_some() {
                    cp.regex_flags = Some("".into());
                }
            }
            if let Some(verb) = c.name("verb_invocation") {
                cp.verb_invocation = Some(VerbInvocation::from(verb.as_str()));
            }
        } else {
            // Non matching pattterns include "///"
            // We decide the whole is a fuzzy search pattern, in this case
            // (this will change when we release the new input syntax)
            cp.pattern = Some(String::from(raw));
        }
        cp
    }
    /// split an input into its two possible parts, the pattern
    /// and the verb invocation. Each part, when defined, is
    /// suitable to create a command on its own.
    pub fn split(raw: &str) -> (Option<String>, Option<String>) {
        let captures = regex!(
            r"(?x)
                ^
                (?P<pattern_part>/?[^\s/:]+/?\w*)?
                (?P<verb_part>[\s:]+(.+))?
                $
            "
        ).captures(raw).unwrap(); // all parts optional : always captures
        (
            captures.name("pattern_part").map(|c| c.as_str().to_string()),
            captures.name("verb_part").map(|c| c.as_str().to_string()),
        )
    }
}

impl Default for CommandParts {
    fn default() -> CommandParts {
        CommandParts::new()
    }
}

impl Action {
    fn from(cp: &CommandParts, finished: bool) -> Action {
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

impl Command {
    pub fn new() -> Command {
        Command {
            raw: String::new(),
            parts: CommandParts::new(),
            action: Action::Unparsed,
        }
    }

    /// create a command from a raw input. `finished` makes
    /// the command an executed form, it's equivalent to
    /// using the Enter key in the Gui.
    pub fn from_raw(raw: String, finished: bool) -> Self {
        let parts = CommandParts::from(&raw);
        let action = Action::from(&parts, finished);
        Self { raw, action, parts }
    }

    /// build a non executed command from a pattern
    pub fn from_pattern(pattern: &Pattern) -> Self {
        Command::from_raw(
            match pattern {
                Pattern::Fuzzy(fp) => fp.to_string(),
                Pattern::Regex(rp) => rp.to_string(),
                Pattern::None => String::new(),
            },
            false,
        )
    }

    /// set the action and clears the other parts :
    ///  the command is now just the action.
    /// This isn't used when the parts must be kept,
    ///  like on edition or enter
    fn set_action(&mut self, action: Action) {
        self.raw = "".to_string();
        self.parts = CommandParts::new();
        self.action = action;
    }

    /// apply an event to modify the command.
    /// The command isn't applied to the state
    pub fn add_event(
        &mut self,
        event: &Event,
        input_field: &mut InputField,
        con: &AppContext,
        state: &dyn AppState,
    ) {
        debug!("add_event {:?}", event);
        self.action = Action::Unparsed;
        match event {
            Event::Click(x, y) => {
                if !input_field.apply_event(&event) {
                    self.set_action(Action::Click(*x, *y));
                }
            }
            Event::DoubleClick(x, y) => {
                self.action = Action::DoubleClick(*x, *y);
            }
            Event::Key(key) => {
                // we first handle the cases that MUST absolutely
                // not be overriden by configuration

                if *key == keys::ENTER && self.parts.verb_invocation.is_some() {
                    self.action = Action::from(&self.parts, true);
                    return;
                }

                if *key == keys::ESC {
                    // Esc it's also a reserved key so order doesn't matter
                    self.set_action(Action::Back);
                    return;
                }

                if *key == keys::QUESTION
                    && (self.raw.is_empty() || self.parts.verb_invocation.is_some())
                {
                    // a '?' opens the help when it's the first char
                    // or when it's part of the verb invocation
                    self.set_action(Action::Help);
                    return;
                }

                // we now check if the key is the trigger key of one of the verbs
                if let Some(index) = con.verb_store.index_of_key(*key) {
                    if state.can_execute(index, con) {
                        self.set_action(Action::VerbIndex(index));
                        return;
                    } else {
                        debug!("verb not allowed on current selection");
                    }
                }

                if *key == keys::ENTER {
                    self.action = Action::from(&self.parts, true);
                    return;
                }

                if *key == keys::ALT_ENTER {
                    self.action = Action::AltOpenSelection;
                    return;
                }

                if *key == keys::TAB {
                    self.set_action(Action::Next);
                    return;
                }

                if *key == keys::BACK_TAB {
                    // should probably be a normal verb instead of an action with a special
                    // handling here
                    self.set_action(Action::Previous);
                    return;
                }

                // input field management
                if input_field.apply_event(&event) {
                    self.raw = input_field.get_content();
                    self.parts = CommandParts::from(&self.raw);
                    self.action = Action::from(&self.parts, false);
                    return;
                }

                // following handling have the lowest priority

                // and there's none, in fact
            }
            Event::Resize(w, h) => {
                self.action = Action::Resize(*w, *h);
            }
            Event::Wheel(lines_count) => {
                self.action = Action::MoveSelection(*lines_count);
            }
            _ => {}
        }
    }
}

impl Default for Command {
    fn default() -> Command {
        Command::new()
    }
}
