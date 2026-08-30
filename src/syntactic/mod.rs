mod text_view;
mod syntax_theme;
mod syntaxer;

pub use {
    text_view::{
        SEPARATOR_FILLING,
        TextView,
        is_char_unprintable,
    },
    syntax_theme::*,
    syntaxer::{
        SYNTAXER,
        Syntaxer,
    },
};
