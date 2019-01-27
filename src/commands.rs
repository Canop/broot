//! A command is the parsed representation of what the user types
//!  in the input. It's independant of the state of the application
//!  (verbs arent checked at this point)

use regex::Regex;
use termion::event::Key;

#[derive(Debug)]
pub struct Command {
    pub raw: String,         // what's visible in the input
    pub parts: CommandParts, // the parsed parts of the visible input
    pub action: Action, // what's required, based on the last key (which may be not visible, like esc)
}

#[derive(Debug, Clone)]
pub struct CommandParts {
    pub has_regex: bool,
    pub pattern: Option<String>,
    pub verb: Option<String>, // may be Some("") if the user already typed the separator
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

impl CommandParts {
    fn new() -> CommandParts {
        CommandParts {
            has_regex: false,
            pattern: None,
            verb: None,
        }
    }
    fn from(raw: &str) -> CommandParts {
        let mut cp = CommandParts::new();
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"(?x)
                ^
                (?P<slash>/)?
                (?P<pattern>[^\s/:]+)?
                (?:[\s:]+(?P<verb>\S*))?
                $
                "
            )
            .unwrap();
        }
        if let Some(c) = RE.captures(raw) {
            cp.has_regex = c.name("slash").is_some();
            if let Some(pattern) = c.name("pattern") {
                cp.pattern = Some(String::from(pattern.as_str()));
            }
            if let Some(verb) = c.name("verb") {
                cp.verb = Some(String::from(verb.as_str()));
            }
        }
        cp
    }
}

impl Action {
    pub fn from(cp: &CommandParts, finished: bool) -> Action {
        if let Some(verb) = &cp.verb {
            match finished {
                false => Action::VerbEdit(String::from(verb.as_str())),
                true => Action::Verb(String::from(verb.as_str())),
            }
        } else if finished {
            Action::OpenSelection
        } else if let Some(pattern) = &cp.pattern {
            Action::PatternEdit(String::from(pattern.as_str()))
        } else {
            Action::PatternEdit(String::from(""))
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
    // build a new command, after execution of a verb
    // (in the future the new action might be built by the state
    //  which would be cleaner)
    pub fn pop_verb(&self) -> Command {
        let mut c = Command::new();
        if self.parts.verb.is_some() {
            if let Some(pat) = &self.parts.pattern {
                c.raw = pat.to_owned();
            }
        }
        c
    }
    // build a command from a string
    // Note that this isn't used (or usable) for interpretation
    //  of the in-app user input. It's meant for interpretation
    //  of a file or from a sequence of commands passed as argument
    //  of the program.
    // A ':', even if at the end, is assumed to mean that the
    //  command must be executed (it's equivalent to the user
    //  typing `enter` in the app
    // This specific syntax isn't definitive
    pub fn from(raw: String) -> Command {
        let parts = CommandParts::from(&raw);
        let action = Action::from(&parts, raw.contains(":"));
        Command { raw, parts, action }
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
                self.action = Action::from(&self.parts, true);
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
                self.parts = CommandParts::from(&self.raw);
                self.action = Action::from(&self.parts, false);
            }
            Key::Esc => {
                self.action = Action::Back;
            }
            Key::Backspace => {
                if self.raw == "" {
                    self.action = Action::Back;
                } else {
                    self.raw.pop();
                    self.parts = CommandParts::from(&self.raw);
                    self.action = Action::from(&self.parts, false);
                }
            }
            _ => {}
        }
    }
}
