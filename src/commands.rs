
use regex::Regex;
use std::io::{self};
use termion::event::Key;

#[derive(Debug)]
pub enum Action {
    MoveSelection(i16),          // up (neg) or down (positive) in the list
    Select(String),              // select by key
    OpenSelection,               // open the selected line (which can't be the root by construct)
    NudeVerb(String),             // verb without selection
    NudeVerbEdit(String),        // verb without selection, unfinished
    VerbSelection(String),        // verb without selection
    VerbSelectionEdit(String),   // verb without selection, unfinished
    Back,                        // back to last app state
    Quit,
    Unparsed,                    // or unparsable
}

impl Action {
    pub fn from(raw: &str, finished: bool) -> Action {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(?x)
                ^
                (?P<key>[0-1a-zA-Z]*)
                (?:\s+(?P<verb>\w+))?
                $
            ").unwrap();
        }
        match RE.captures(raw) {
            Some(c) => {
                match (c.name("key"), c.name("verb"), finished) {
                    (Some(_key), Some(verb), false) => Action::VerbSelectionEdit(String::from(verb.as_str())),
                    (Some(_key), Some(verb), true)  => Action::VerbSelection(String::from(verb.as_str())),
                    (Some(key), None, false)       => Action::Select(String::from(key.as_str())),
                    (Some(_key), None, true)        => Action::OpenSelection,
                    (None, Some(verb), false)      => Action::NudeVerbEdit(String::from(verb.as_str())),
                    (None, Some(verb), true)       => Action::NudeVerb(String::from(verb.as_str())),
                    _                              => Action::Unparsed, // exemple: finishes with a space
                }
            },
            None    => {
                Action::Unparsed
            }
        }
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
    pub fn from(raw: &str) -> Command {
        Command {
            raw: String::from(raw),
            action: Action::from(raw, false),
        }
    }
    pub fn add_key(&mut self, key: Key) -> io::Result<()> {
        match key {
            Key::Char('\n') => { // enter
                if self.raw == "" {
                    self.action = Action::Quit;
                } else {
                    self.action = Action::from(&self.raw, true);
                }
            },
            Key::Up         => {
                self.action = Action::MoveSelection(-1);
            },
            Key::Down       => {
                self.action = Action::MoveSelection(1);
            },
            Key::Char(c)    => {
                self.raw.push(c);
                self.action = Action::from(&self.raw, false);
            },
            Key::Esc        => {
                if self.raw == "" {
                    self.action = Action::Back;
                } else {
                    self.raw.clear();
                    self.action = Action::Select(String::from(""));
                }
            },
            Key::Backspace  => {
                if self.raw == "" {
                    self.action = Action::Back;
                } else {
                    self.raw.pop();
                    self.action = Action::from(&self.raw, false);
                }
            },
            _               => {
            },
        }
        Ok(())
    }
}
