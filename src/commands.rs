use regex::Regex;
use termion::event::Key;

/// A command is the parsed representation of what the user types
///  in the input. It's independant of the state of the application
///  (verbs arent checked at this point)

struct CommandParts {
    pattern: Option<String>,
    verb: Option<String>,
}

impl CommandParts {
    fn from(raw: &str) -> CommandParts {
        let mut cp = CommandParts {
            pattern: None,
            verb: None,
        };
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"(?x)
                ^
                (?P<pattern>[^\s/:]*)
                (?:[\s:]+(?P<verb>\S*))?
                $
                "
            )
            .unwrap();
        }
        if let Some(c) = RE.captures(raw) {
            if let Some(pattern) = c.name("pattern") {
                cp.pattern = Some(String::from(pattern.as_str()));
            }
            if let Some(verb) = c.name("verb") {
                cp.verb = Some(String::from(verb.as_str()));
            }
        } else {
            warn!("unexpected lack of capture");
        }
        cp
    }
}

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
        let cp = CommandParts::from(raw);
        if let Some(verb) = cp.verb {
            return match finished {
                false => Action::VerbEdit(String::from(verb.as_str())),
                true => Action::Verb(String::from(verb.as_str())),
            };
        }
        if let Some(pattern) = cp.pattern {
            let pattern = pattern.as_str();
            return match finished {
                false => Action::PatternEdit(String::from(pattern)),
                true => Action::OpenSelection,
            };
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
    // build a new command, after execution of a verb
    // (in the future the new action might be built by the state
    //  which would be cleaner)
    pub fn pop_verb(&self) -> Command {
        let mut c = Command::new();
        let cp = CommandParts::from(&self.raw);
        if cp.verb.is_some() {
            if let Some(pat) = cp.pattern {
                c.raw = pat;
            }
        }
        c
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
