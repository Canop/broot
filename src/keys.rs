//! parsing keys from strings, and describing keys in strings

use {
    crate::{
        errors::ConfError,
    },
    crossterm::event::{
        KeyCode::*,
        KeyEvent,
        KeyModifiers,
    },
};

macro_rules! const_key {
    ($name:ident, $code:expr) => {
        pub const $name: KeyEvent = KeyEvent{code:$code, modifiers:KeyModifiers::empty()};
    };
    ($name:ident, $code:expr, $mod:expr) => {
        pub const $name: KeyEvent = KeyEvent{code:$code, modifiers:$mod};
    };
}

// we define a few constants which make it easier to check key events
const_key!(ALT_ENTER, Enter, KeyModifiers::ALT);
const_key!(ENTER, Enter);
const_key!(BACKSPACE, Backspace);
const_key!(BACK_TAB, BackTab);
const_key!(CTRL_S, Char('s'), KeyModifiers::CONTROL);
const_key!(DELETE, Delete);
const_key!(DOWN, Down);
const_key!(END, End);
const_key!(ESC, Esc);
const_key!(HOME, Home);
const_key!(LEFT, Left);
const_key!(QUESTION, Char('?'));
const_key!(RIGHT, Right);
const_key!(TAB, Tab);
const_key!(UP, Up);

/// build a human description of a key event
pub fn key_event_desc(key: KeyEvent) -> String {
    let mut s = String::new();
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        s.push_str("ctrl-");
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        s.push_str("alt-");
    }
    if key.modifiers.contains(KeyModifiers::SHIFT) {
        s.push_str("shift-");
    }
    match key.code {
        Char('\r') | Char('\n') | Enter => {
            s.push_str("enter");
        }
        Char(c) => {
            s.push(c);
        }
        F(u) => {
            s.push_str(&format!("F{}", u));
        }
        _ => {
            s.push_str(&format!("{:?}", key.code)); // FIXME check
        }
    }
    s
}

fn bad_key(raw: &str) -> Result<KeyEvent, ConfError> {
    Err(ConfError::InvalidKey {
        raw: raw.to_owned(),
    })
}

pub fn is_reserved(key: KeyEvent) -> bool {
    match key {
        BACKSPACE => true, // needed for the input field
        HOME => true, // needed for the input field
        END => true, // needed for the input field
        LEFT => true, // needed for the input field
        RIGHT => true, // needed for the input field
        DELETE => true, // needed for the input field
        ESC => true, // basic navigation
        UP => true, // basic navigation
        DOWN => true, // basic navigation
        _ => false,
    }
}

/// parse a string as a keyboard key definition.
///
pub fn parse_key(raw: &str) -> Result<KeyEvent, ConfError> {
    let tokens: Vec<&str> = raw.split('-').collect();
    let last = tokens[tokens.len()-1].to_ascii_lowercase();
    let code = match last.as_ref() {
        "esc" => Esc,
        "enter" => Enter,
        "left" => Left,
        "right" => Right,
        "up" => Up,
        "down" => Down,
        "home" => Home,
        "end" => End,
        "pageup" => PageUp,
        "pagedown" => PageDown,
        "backtab" => BackTab,
        "backspace" => Backspace,
        "delete" => Delete,
        "insert" => Insert,
        "ins" => Insert,
        "f1" => F(1),
        "f2" => F(2),
        "f3" => F(3),
        "f4" => F(4),
        "f5" => F(5),
        "f6" => F(6),
        "f7" => F(7),
        "f8" => F(8),
        "f9" => F(9),
        "f10" => F(10),
        "f11" => F(11),
        "f12" => F(12),
        c if c.len()==1 => Char(c.chars().next().unwrap()),
        _=> {
            return bad_key(raw);
        }
    };
    let mut modifiers = KeyModifiers::empty();
    for i in 0..tokens.len()-1 {
        let token = tokens[i];
        match token.to_ascii_lowercase().as_ref() {
            "ctrl" => {
                modifiers.insert(KeyModifiers::CONTROL);
            }
            "alt" => {
                modifiers.insert(KeyModifiers::ALT);
            }
            "shift" => {
                modifiers.insert(KeyModifiers::SHIFT);
            }
            _=> {
                return bad_key(raw);
            }
        }
    }
    Ok(KeyEvent{ code, modifiers })
}
#[cfg(test)]
mod key_parsing_tests {

    use {
        crate::keys::*,
        crossterm::event::{
            KeyEvent,
            KeyCode::*,
        },
    };

    #[test]
    fn check_key_description(){
        assert_eq!(key_event_desc(ALT_ENTER), "alt-enter");
    }

    fn check_ok(raw: &str, key: KeyEvent) {
        let parsed = parse_key(raw);
        assert!(parsed.is_ok(), "failed to parse {:?} as key", raw);
        assert_eq!(parsed.unwrap(), key);
    }

    #[test]
    fn check_key_parsing() {
        check_ok("left", LEFT);
        check_ok("RIGHT", RIGHT);
        check_ok("Home", HOME);
        check_ok("f1", KeyEvent::from(F(1)));
        check_ok("F2", KeyEvent::from(F(2)));
        check_ok("Enter", KeyEvent::from(Enter));
        check_ok("alt-enter", KeyEvent::new(Enter, KeyModifiers::ALT));
        check_ok("insert", KeyEvent::from(Insert));
        check_ok("ctrl-Q", KeyEvent::new(Char('q'), KeyModifiers::CONTROL));
    }
}

