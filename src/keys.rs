use {
    crate::{
        app::Mode,
    },
    crokey::*,
    crossterm::event::{
        KeyCode,
        KeyEvent,
        KeyModifiers,
    },
    once_cell::sync::Lazy,
};

pub static KEY_FORMAT: Lazy<KeyEventFormat> = Lazy::new(|| {
    KeyEventFormat::default().with_lowercase_modifiers()
});

pub fn is_reserved(key: KeyEvent) -> bool {
    key == key!(backspace) || key == key!(delete) || key == key!(esc)
}

pub fn is_key_only_modal(
    key: KeyEvent,
) -> bool {
    matches!(key, KeyEvent { code: KeyCode::Char(_), modifiers: KeyModifiers::NONE })
    || matches!(key, KeyEvent { code: KeyCode::Char(_), modifiers: KeyModifiers::SHIFT })
}

pub fn is_key_allowed_for_verb(
    key: KeyEvent,
    mode: Mode,
    input_is_empty: bool,
) -> bool {
    match mode {
        Mode::Input => {
            // in input mode, keys normally used in the input are forbidden
            if key==key!(left) || key==key!(right) {
                input_is_empty
            } else {
                !is_key_only_modal(key)
            }
        }
        Mode::Command => true,
    }
}

