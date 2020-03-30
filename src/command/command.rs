//! A command is the parsed representation of what the user types
//!  in the input. It's independant of the state of the application

use {
    crate::{
        app::{
            AppContext,
            AppState,
        },
        keys,
        pattern::Pattern,
    },
    super::*,
    termimad::{Event, InputField},
};

#[derive(Debug, Clone)]
pub struct Command {
    pub raw: String,     // what's visible in the input
    parts: CommandParts, // the parsed parts of the visible input
    pub action: Action, // what's required, based on the last key (which may be not visible, like esc)
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
            Event::Click(x, y, ..) => {
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

                if *key == keys::LEFT && self.raw.is_empty() {
                    self.set_action(Action::Back);
                    return;
                }

                if *key == keys::RIGHT && self.raw.is_empty() {
                    self.set_action(Action::OpenSelection);
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
