use {
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

