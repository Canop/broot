use patterns::Pattern;

#[derive(Debug, Clone)]
pub struct TreeOptions {
    pub show_hidden: bool,
    pub pattern: Option<Pattern>, // remove from there?
}

impl TreeOptions {
    pub fn new() -> TreeOptions {
        TreeOptions {
            show_hidden: false,
            pattern: None,
        }
    }
}
