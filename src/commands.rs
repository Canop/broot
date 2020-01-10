//! A command is the parsed representation of what the user types
//!  in the input. It's independant of the state of the application
//!  (verbs arent checked at this point)

use {
    regex::Regex,
    termimad::{Event, InputField},
    crate::{
        app_context::AppContext,
        app_state::AppState,
        keys,
        verb_invocation::VerbInvocation,
        patterns::Pattern,
    },
};

#[derive(Debug, Clone)]
pub struct Command {
    pub raw: String,     // what's visible in the input
    parts: CommandParts, // the parsed parts of the visible input
    pub action: Action, // what's required, based on the last key (which may be not visible, like esc)
}

/// An intermediate parsed representation of the raw string
#[derive(Debug, Clone)]
struct CommandParts {
    pattern: Option<String>,     // either a fuzzy pattern or the core of a regex
    regex_flags: Option<String>, // may be Some("") if user asked for a regex but specified no flag
    verb_invocation: Option<VerbInvocation>, // may be empty if user already typed the separator but no char after
}

#[derive(Debug, Clone)]
pub enum Action {
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
    Unparsed,                     // or unparsable
}

impl CommandParts {
    fn new() -> CommandParts {
        CommandParts {
            pattern: None,
            regex_flags: None,
            verb_invocation: None,
        }
    }
    fn from(raw: &str) -> CommandParts {
        let mut cp = CommandParts::new();
        let r = regex!(
            r"(?x)
                ^
                (?P<slash_before>/)?
                (?P<pattern>[^\s/:]+)?
                (?:/(?P<regex_flags>\w*))?
                (?:[\s:]+(?P<verb_invocation>.*))?
                $
                "
        );
        if let Some(c) = r.captures(raw) {
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
        }
        cp
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

    /// build a command from a string
    /// Note that this isn't used (or usable) for interpretation
    ///  of the in-app user input. It's meant for interpretation
    ///  of a file or from a sequence of commands passed as argument
    ///  of the program.
    /// A ':', even if at the end, is assumed to mean that the
    ///  command must be executed (it's equivalent to the user
    ///  typing `enter` in the app
    /// This specific syntax isn't definitive
    pub fn from(raw: String) -> Command {
        let parts = CommandParts::from(&raw);
        let action = Action::from(&parts, raw.contains(':'));
        Command { raw, parts, action }
    }

    pub fn from_pattern(pattern: &Pattern) -> Self {
        Self::from(match pattern {
            Pattern::Fuzzy(fp) => fp.to_string(),
            Pattern::Regex(rp) => rp.to_string(),
            Pattern::None => String::new(),
        })
    }

    /// apply an event to modify the command.
    /// The command isn't applied to the state
    pub fn add_event(
        &mut self,
        event: &Event,
        input_field: &mut InputField,
        con: &AppContext,
        state: &Box<dyn AppState>,
    ) {
        debug!("add_event {:?}", event);
        self.action = Action::Unparsed;
        match event {
            Event::Click(x, y) => {
                if !input_field.apply_event(&event) {
                    self.action = Action::Click(*x, *y);
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

                if *key == keys::ALT_ENTER {
                    self.action = Action::AltOpenSelection;
                    return;
                }

                if *key == keys::ESC {
                    // Esc it's also a reserved key so order doesn't matter
                    self.action = Action::Back;
                    return;
                }

                if *key == keys::QUESTION && (self.raw.is_empty() || self.parts.verb_invocation.is_some()) {
                    // a '?' opens the help when it's the first char
                    // or when it's part of the verb invocation
                    self.action = Action::Help;
                    return;
                }

                // we now check if the key is the trigger key of one of the verbs
                if let Some(index) = con.verb_store.index_of_key(*key) {
                    if state.can_execute(index, con) {
                        self.action = Action::VerbIndex(index);
                        return;
                    } else {
                        debug!("verb not allowed on current selection");
                    }
                }

                if *key == keys::ENTER {
                    self.action = Action::from(&self.parts, true);
                    return;
                }

                if *key == keys::TAB {
                    self.action = Action::Next;
                    return;
                }

                if *key == keys::BACK_TAB {
                    // should probably be a normal verb instead of an action with a special
                    // handling here
                    self.action = Action::Previous;
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


