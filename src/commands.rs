//! A command is the parsed representation of what the user types
//!  in the input. It's independant of the state of the application
//!  (verbs arent checked at this point)

use regex::Regex;
use termion::event::Key;
use crate::verb_invocation::VerbInvocation;

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
    MoveSelection(i32),        // up (neg) or down (positive) in the list
    ScrollPage(i32),           // in number of pages, not lines
    OpenSelection,             // open the selected line
    AltOpenSelection,          // alternate open the selected line
    VerbEdit(VerbInvocation),          // verb invocation, unfinished
    Verb(VerbInvocation),              // verb invocation, after the user hit enter
    FuzzyPatternEdit(String),  // a pattern being edited
    RegexEdit(String, String), // a regex being edited (core & flags)
    Back,                      // back to last app state, or clear pattern
    Next,                      // goes to the next matching entry
    Help,                      // goes to help state
    Quit,                      // quit broot
    Unparsed,                  // or unparsable
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
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"(?x)
                ^
                (?P<slash_before>/)?
                (?P<pattern>[^\s/:]+)?
                (?:/(?P<regex_flags>\w*))?
                (?:[\s:]+(?P<verb_invocation>.*))?
                $
                "
            )
            .unwrap();
        }
        if let Some(c) = RE.captures(raw) {
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
                Action::Verb(verb_invocation.clone())
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
        let action = Action::from(&parts, raw.contains(':'));
        Command { raw, parts, action }
    }
    pub fn add_key(&mut self, key: Key) {
        match key {
            Key::Char('\t') => {
                self.action = Action::Next;
            }
            Key::Char('\n') => {
                self.action = Action::from(&self.parts, true);
            }
            Key::Alt('\r') | Key::Alt('\n') => {
                self.action = Action::AltOpenSelection;
            }
            Key::Ctrl('q') => {
                self.action = Action::Quit;
            }
            Key::Up => {
                self.action = Action::MoveSelection(-1);
            }
            Key::Down => {
                self.action = Action::MoveSelection(1);
            }
            Key::PageUp | Key::Ctrl('u') => {
                self.action = Action::ScrollPage(-1);
            }
            Key::PageDown | Key::Ctrl('d') => {
                self.action = Action::ScrollPage(1);
            }
            Key::Char(c) => {
                if c == '?' && self.raw.is_empty() {
                    // as first character, a '?' is a request for help
                    self.action = Action::Help;
                } else {
                    self.raw.push(c);
                    self.parts = CommandParts::from(&self.raw);
                    self.action = Action::from(&self.parts, false);
                }
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
