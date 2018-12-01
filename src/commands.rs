use regex::Regex;
use std::io;
use termion::event::Key;

#[derive(Debug)]
pub enum Action {
    MoveSelection(i16), // up (neg) or down (positive) in the list
    Select(String),     // select by key
    OpenSelection,      // open the selected line (which can't be the root by construct)
    VerbEdit(String),   // verb, unfinished
    Verb(String),       // verb
    PatternEdit(String),// a pattern being edited
    FixPattern,
    Back, // back to last app state, or clear pattern
    Next,
    Quit,
    Unparsed, // or unparsable
}

impl Action {
    pub fn from(raw: &str, finished: bool) -> Action {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"(?x)
                ^
                (?:/(?P<pattern>[^\s/:]*))?
                (?P<key>[0-9a-zA-Z]*)
                (?:[\s:]+(?P<verb>\w+))?
                $
                "
            ).unwrap();
        }
        if let Some(c) = RE.captures(raw) {
            if let Some(verb) = c.name("verb") {
                return match finished {
                    false => Action::VerbEdit(String::from(verb.as_str())),
                    true => Action::Verb(String::from(verb.as_str())),
                };
            }
            if let Some(pattern) = c.name("pattern") {
                let pattern = pattern.as_str();
                return match finished {
                    false => Action::PatternEdit(String::from(pattern)),
                    true => Action::FixPattern,
                };
            }
            if let Some(key) = c.name("key") {
                return match finished {
                    false => Action::Select(String::from(key.as_str())),
                    true => Action::OpenSelection,
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
    pub fn add_key(&mut self, key: Key) -> io::Result<()> {
        match key {
            Key::Char('\t') => {
               self.action = Action::Next;
            }
            Key::Char('\n') => {
                if self.raw == "" {
                    self.action = Action::Quit;
                } else {
                    self.action = Action::from(&self.raw, true);
                }
            }
            Key::Up => {
                self.action = Action::MoveSelection(-1);
            }
            Key::Down => {
                self.action = Action::MoveSelection(1);
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
        Ok(())
    }
}
