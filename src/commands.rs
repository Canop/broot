
use regex::Regex;

#[derive(Debug)]
pub struct Command {
    pub raw: String,
    pub finished: bool, // user hit <enter>
    pub key: String,
}

impl Command {
    pub fn new() -> Command {
        Command {
            raw: String::new(),
            key: String::from(""),
            finished: false,
        }
    }
    // analyzes the raw command to fill key, verb
    pub fn parse(&mut self) {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(?x)
                ^
                (?P<key>[0-1a-z]+)?
                (?:\s+(?P<verb>\w+))?
                $
            ").unwrap();
        }
        match RE.captures(&self.raw) {
            Some(c) => {
                self.key = match c.name("key") {
                    Some(key)   => String::from(key.as_str()),
                    None        => String::from(""),
                };
            },
            None    => {
                self.key = String::from("");
            }
        }

    }
}
