use {
    syntect::{
        parsing::SyntaxSet,
        highlighting::ThemeSet,
    },
};

/// wrap heavy to initialize syntect things
pub struct Syntaxer {
    pub syntax_set: SyntaxSet,
    pub theme_set: ThemeSet,
}
impl Syntaxer {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_nonewlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }
}
