
use regex::Regex;

#[derive(Debug)]
pub enum Action {
    Select(String),
    MoveSelection(i16),
    Quit,
    Unparsed, // or unparsable
}

impl Action {
    // analyzes the raw command to fill key, verb.
    // Only makes sense when there was no special key
    pub fn from(raw: &str) -> Action {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(?x)
                ^
                (?P<key>[0-1a-z]*)
                (?:\s+(?P<verb>\w+))?
                $
            ").unwrap();
        }
        match RE.captures(raw) {
            Some(c) => {
                let key = match c.name("key") {
                    Some(key)   => String::from(key.as_str()),
                    None        => String::from(""), // should not happen
                };
                // TODO handle verb
                Action::Select(key)
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
    pub finished: bool, // user hit <enter>
    pub action: Action,
}

impl Command {
    pub fn new() -> Command {
        Command {
            raw: String::new(),
            finished: false,
            action: Action::Unparsed,
        }
    }
}
