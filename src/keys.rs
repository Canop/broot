use {
    crokey::*,
    crossterm::event::{
        KeyCode,
        KeyModifiers,
    },
    once_cell::sync::Lazy,
};

pub static KEY_FORMAT: Lazy<KeyCombinationFormat> =
    Lazy::new(|| KeyCombinationFormat::default().with_lowercase_modifiers());

pub fn is_reserved(key: KeyCombination) -> bool {
    key == key!(backspace) || key == key!(delete) || key == key!(esc)
}

/// Tell whether the key can only be used as a shortcut key if the
/// modal mode is active.
pub fn is_key_only_modal(key: KeyCombination) -> bool {
    matches!(
        key,
        KeyCombination {
            codes: OneToThree::One(KeyCode::Char(_)),
            modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
        }
    )
}


