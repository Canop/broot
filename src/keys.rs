//! parsing keys from strings, and describing keys in strings

use {
    crate::{
        errors::ConfError,
    },
    crossterm::event::{
        KeyCode,
        KeyEvent,
        KeyModifiers,
    },
};

macro_rules! const_key {
    ($name:ident, $code:ident) => {
        pub const $name: KeyEvent = KeyEvent{code:KeyCode::$code, modifiers:KeyModifiers::empty()};
    };
    ($name:ident, $code:expr) => {
        pub const $name: KeyEvent = KeyEvent{code:$code, modifiers:KeyModifiers::empty()};
    };
    ($name:ident, $code:ident, $mod:expr) => {
        pub const $name: KeyEvent = KeyEvent{code:KeyCode::$code, modifiers:$mod};
    };
    ($name:ident, $code:expr, $mod:expr) => {
        pub const $name: KeyEvent = KeyEvent{code:$code, modifiers:$mod};
    };
}

// we define a few constants which make it easier to check key events
const_key!(ALT_ENTER, KeyCode::Char('\r'), KeyModifiers::ALT);
const_key!(ENTER, Enter);
const_key!(BACKSPACE, Backspace);
const_key!(BACK_TAB, BackTab);
const_key!(DELETE, Delete);
const_key!(DOWN, Down);
const_key!(END, End);
const_key!(ESC, Esc);
const_key!(HOME, Home);
const_key!(LEFT, Left);
const_key!(QUESTION, KeyCode::Char('?'));
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
        KeyCode::Char('\r') | KeyCode::Char('\n') => {
            s.push_str("enter");
        }
        KeyCode::Char(c) => {
            s.push(c);
        }
        KeyCode::F(u) => {
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
        "esc" => KeyCode::Esc,
        "enter" => KeyCode::Enter,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "backtab" => KeyCode::BackTab,
        "backspace" => KeyCode::Backspace,
        "delete" => KeyCode::Delete,
        "insert" => KeyCode::Insert,
        "ins" => KeyCode::Insert,
        "f1" => KeyCode::F(1),
        "f2" => KeyCode::F(2),
        "f3" => KeyCode::F(3),
        "f4" => KeyCode::F(4),
        "f5" => KeyCode::F(5),
        "f6" => KeyCode::F(6),
        "f7" => KeyCode::F(7),
        "f8" => KeyCode::F(8),
        "f9" => KeyCode::F(9),
        "f10" => KeyCode::F(10),
        "f11" => KeyCode::F(11),
        "f12" => KeyCode::F(12),
        c if c.len()==1 => KeyCode::Char(c.chars().next().unwrap()),
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
            KeyCode,
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
        check_ok("f1", KeyEvent::from(KeyCode::F(1)));
        check_ok("F2", KeyEvent::from(KeyCode::F(2)));
        check_ok("Enter", KeyEvent::from(KeyCode::Enter));
        check_ok("alt-enter", KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT));
        check_ok("insert", KeyEvent::from(KeyCode::Insert));
        check_ok("ctrl-Q", KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL));
    }
}

