#![warn(clippy::all)]

use regex::Regex;
use termion::event::Key;

/// A command is the parsed representation of what the user types
///  in the input. It's independant of the state of the application
///  (verbs arent checked at this point)
#[derive(Debug)]
pub enum Action {
    MoveSelection(i32),  // up (neg) or down (positive) in the list
    ScrollPage(i32),     // in number of pages, not lines
    OpenSelection,       // open the selected line (which can't be the root by construct)
    VerbEdit(String),    // verb, unfinished
    Verb(String),        // verb
    PatternEdit(String), // a pattern being edited
    Back,                // back to last app state, or clear pattern
    Next,
    Help(String),
    Unparsed, // or unparsable
}

impl Action {
    pub fn from(raw: &str, finished: bool) -> Action {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"(?x)
                ^
                (?P<pattern>[^\s/:]*)
                (?:[\s:]+(?P<verb>\w*))?
                $
                "
            )
            .unwrap();
        }
        if let Some(c) = RE.captures(raw) {
            if let Some(verb) = c.name("verb") {
                return if finished {
                    Action::Verb(String::from(verb.as_str()))
                } else {
                    Action::VerbEdit(String::from(verb.as_str()))
                };
            }
            if let Some(pattern) = c.name("pattern") {
                let pattern = pattern.as_str();

                return if finished {
                    Action::OpenSelection
                } else {
                    Action::PatternEdit(String::from(pattern))
                };
            }
        } else {
            warn!("unexpected lack of capture");
        }
        Action::Unparsed
    }
}

#[derive(Debug)]
pub struct Command {
    pub raw: String,
    pub action: Action,
}

impl Command {
    pub fn new() -> Command {
        Command {
            raw: String::new(),
            action: Action::Unparsed,
        }
    }
    pub fn add_key(&mut self, key: Key) {
        match key {
            Key::Char('\t') => {
                self.action = Action::Next;
            }
            Key::Char('?') => {
                // we might be a little more subtle in the future
                self.action = Action::Help(self.raw.to_owned());
            }
            Key::Char('\n') => {
                self.action = Action::from(&self.raw, true);
            }
            Key::Up => {
                self.action = Action::MoveSelection(-1);
            }
            Key::Down => {
                self.action = Action::MoveSelection(1);
            }
            Key::PageUp => {
                self.action = Action::ScrollPage(-1);
            }
            Key::PageDown => {
                self.action = Action::ScrollPage(1);
            }
            Key::Char(c) => {
                self.raw.push(c);
                self.action = Action::from(&self.raw, false);
            }
            Key::Esc => {
                self.action = Action::Back;
            }
            Key::Backspace => {
                if self.raw == "" {
                    self.action = Action::Back;
                } else {
                    self.raw.pop();
                    self.action = Action::from(&self.raw, false);
                }
            }
            _ => {}
        }
    }
}
