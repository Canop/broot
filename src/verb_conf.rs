use crossterm::KeyEvent;
use regex::Regex;

use crate::errors::ConfError;

/// what's needed to handle a verb
#[derive(Debug)]
pub struct VerbConf {
    pub shortcut: Option<String>,
    pub invocation: String,
    pub key: Option<KeyEvent>,
    pub execution: String,
    pub description: Option<String>,
    pub from_shell: Option<bool>,
    pub leave_broot: Option<bool>,
    pub confirm: Option<bool>,
}

fn bad_key(raw: &str) -> Result<KeyEvent, ConfError> {
    Err(ConfError::InvalidKey {
        raw: raw.to_owned(),
    })
}

/// parse a string as a keyboard key definition.
///
/// Note that some mappings allowed by crossterm aren't
/// parsed because we don't want to let the user override
/// the related behaviors.
pub fn parse_key(raw: &str) -> Result<KeyEvent, ConfError> {
    let key_regex = regex!(
        r"(?x)
        ^
        (?P<major>([a-zA-Z]+|\^))
        (?:\W?(?P<minor>\w\d?)\W?)?
        $
        "
    );
    match key_regex.captures(raw) {
        Some(c) => Ok(
            match (
                c.name("major")
                    .unwrap()
                    .as_str()
                    .to_ascii_lowercase()
                    .as_ref(),
                c.name("minor"),
            ) {
                ("left", None) => KeyEvent::Left,
                ("right", None) => KeyEvent::Right,
                ("up", None) => KeyEvent::Up,
                ("down", None) => KeyEvent::Down,
                ("home", None) => KeyEvent::Home,
                ("end", None) => KeyEvent::End,
                ("pageup", None) => KeyEvent::PageUp,
                ("pagedown", None) => KeyEvent::PageDown,
                ("backtab", None) => KeyEvent::BackTab,
                ("delete", None) => KeyEvent::Delete,
                ("insert", None) => KeyEvent::Insert,
                ("f", Some(minor)) => match minor.as_str().parse() {
                    Ok(digit) => KeyEvent::F(digit),
                    _ => bad_key(raw)?,
                },
                ("alt", Some(minor)) => {
                    KeyEvent::Alt(minor.as_str().chars().next().unwrap().to_ascii_lowercase())
                }
                ("ctrl", Some(minor)) | ("^", Some(minor)) => {
                    KeyEvent::Ctrl(minor.as_str().chars().next().unwrap().to_ascii_lowercase())
                }
                // other possible mappings are disabled as they would break basic behaviors of broot
                _ => bad_key(raw)?,
            },
        ),
        None => bad_key(raw),
    }
}

#[cfg(test)]
mod key_parsing_tests {

    use crossterm::KeyEvent::*;

    use crate::verb_conf::*;

    fn check_ok(raw: &str, key: KeyEvent) {
        let parsed = parse_key(raw);
        assert!(parsed.is_ok(), "failed to parse {:?} as key", raw);
        assert_eq!(parsed.unwrap(), key);
    }

    #[test]
    fn check_key_parsing() {
        check_ok("left", Left);
        check_ok("right", Right);
        check_ok("Home", Home);
        check_ok("f1", F(1));
        check_ok("F2", F(2));
        check_ok("F-3", F(3));
        check_ok("F(4)", F(4));
        check_ok("F 12", F(12));
        check_ok("F(12)", F(12));
        assert!(parse_key("F(a)").is_err(), "should not have parsed F(a)");
        check_ok("Up", Up);
        check_ok("down", Down);
        check_ok("insert", Insert);
        check_ok("alt(4)", Alt('4'));
        check_ok("alt-D", Alt('d'));
        check_ok("ctrl-q", Ctrl('q'));
        check_ok("ctrl-Q", Ctrl('q'));
        check_ok("ctrl Q", Ctrl('q'));
        check_ok("^Q", Ctrl('q'));
    }
}
