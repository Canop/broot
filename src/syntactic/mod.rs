mod text_view;
mod syntax_theme;
mod syntaxer;

pub use {
    text_view::TextView,
    syntax_theme::*,
    syntaxer::{
        SYNTAXER,
        Syntaxer,
    },
};
