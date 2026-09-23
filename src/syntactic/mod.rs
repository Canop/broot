mod text_view;
mod syntax_theme;
mod syntaxer;

pub use {
    text_view::{
        SEPARATOR_FILLING,
        TextView,
        printable_line,
    },
    syntax_theme::*,
    syntaxer::{
        SYNTAXER,
        Syntaxer,
    },
};
