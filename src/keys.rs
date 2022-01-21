//! parsing keys from strings, and describing keys in strings

use {
    crate::{
        app::Mode,
        errors::ConfError,
    },
    crossterm::event::{
        KeyCode::{self, *},
        KeyEvent,
        KeyModifiers,
    },
};

macro_rules! const_key {
    ($name:ident, $code:expr) => {
        pub const $name: KeyEvent = KeyEvent {
            code: $code,
            modifiers: KeyModifiers::empty(),
        };
    };
    ($name:ident, $code:expr, $mod:expr) => {
        pub const $name: KeyEvent = KeyEvent {
            code: $code,
            modifiers: $mod,
        };
    };
}

// we define a few constants which make it easier to check key events
const_key!(ALT_ENTER, Enter, KeyModifiers::ALT);
const_key!(ENTER, Enter);
const_key!(BACKSPACE, Backspace);
const_key!(BACK_TAB, BackTab, KeyModifiers::SHIFT); // backtab needs shift
const_key!(DELETE, Delete);
const_key!(DOWN, Down);
const_key!(PAGE_DOWN, PageDown);
const_key!(END, End);
const_key!(ESC, Esc);
const_key!(HOME, Home);
const_key!(LEFT, Left);
const_key!(QUESTION, Char('?'));
const_key!(RIGHT, Right);
const_key!(SPACE, Char(' '));
const_key!(TAB, Tab);
const_key!(UP, Up);
const_key!(PAGE_UP, PageUp);
const_key!(F1, F(1));
const_key!(F2, F(2));
const_key!(F3, F(3));
const_key!(F4, F(4));
const_key!(F5, F(5));
const_key!(F6, F(6));

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
            s.push_str(&format!("F{u}"));
        }
        _ => {
            s.push_str(&format!("{:?}", key.code)); // FIXME check
        }
    }
    s
}

pub fn is_reserved(key: KeyEvent) -> bool {
    key == BACKSPACE || key == DELETE || key == ESC
}

pub fn is_key_allowed_for_verb(
    key: KeyEvent,
    mode: Mode,
    input_is_empty: bool,
) -> bool {
    match mode {
        Mode::Input => {
            // in input mode, keys normally used in the input are forbidden
            if key==LEFT || key==RIGHT {
                input_is_empty
            } else {
                !matches!(key, KeyEvent { code: KeyCode::Char(_), modifiers: KeyModifiers::NONE })
            }
        }
        Mode::Command => true,
    }
}

/// return the raw char if the event is a letter event
pub fn as_letter(key: KeyEvent) -> Option<char> {
    match key {
        KeyEvent { code: KeyCode::Char(l), modifiers: KeyModifiers::NONE } => Some(l),
        _ => None,
    }
}

/// parse a string as a keyboard key definition.
///
/// About the case:
/// The char we receive as code from crossterm is usually lowercase
/// but uppercase when it was typed with shift (i.e. we receive
/// "g" for a lowercase, and "shift-G" for an uppercase)
pub fn parse_key(raw: &str) -> Result<KeyEvent, ConfError> {
    fn bad_key(raw: &str) -> Result<KeyEvent, ConfError> {
        Err(ConfError::InvalidKey {
            raw: raw.to_owned(),
        })
    }
    let tokens: Vec<&str> = raw.split('-').collect();
    let last = tokens[tokens.len() - 1].to_ascii_lowercase();
    let mut code = match last.as_ref() {
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
        "del" => Delete,
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
        "space" => Char(' '),
        "tab" => Tab,
        c if c.len() == 1 => Char(c.chars().next().unwrap()),
        _ => {
            return bad_key(raw);
        }
    };
    let mut modifiers = KeyModifiers::empty();
    if code == BackTab {
        // Crossterm always sends the shift key with
        // backtab
        modifiers.insert(KeyModifiers::SHIFT);
    }
    for token in tokens.iter().take(tokens.len() - 1) {
        match token.to_ascii_lowercase().as_ref() {
            "ctrl" => {
                modifiers.insert(KeyModifiers::CONTROL);
            }
            "alt" => {
                modifiers.insert(KeyModifiers::ALT);
            }
            "shift" => {
                modifiers.insert(KeyModifiers::SHIFT);
                if let Char(c) = code {
                    if c.is_ascii_lowercase() {
                        code = Char(c.to_ascii_uppercase());
                    }
                }
            }
            _ => {
                return bad_key(raw);
            }
        }
    }
    Ok(KeyEvent { code, modifiers })
}
#[cfg(test)]
mod key_parsing_tests {

    use {
        crate::keys::*,
        crossterm::event::KeyEvent,
    };

    #[test]
    fn check_key_description() {
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
        check_ok("ctrl-q", KeyEvent::new(Char('q'), KeyModifiers::CONTROL));
        check_ok("shift-q", KeyEvent::new(Char('Q'), KeyModifiers::SHIFT));
        check_ok("ctrl-Q", KeyEvent::new(Char('q'), KeyModifiers::CONTROL));
        check_ok("shift-Q", KeyEvent::new(Char('Q'), KeyModifiers::SHIFT));
    }
}
