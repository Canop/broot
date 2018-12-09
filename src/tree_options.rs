use patterns::Pattern;

#[derive(Debug, Clone)]
pub struct TreeOptions {
    pub show_hidden: bool,
    pub only_folders: bool,
    pub pattern: Option<Pattern>, // remove from there?
}

impl TreeOptions {
    pub fn new() -> TreeOptions {
        TreeOptions {
            show_hidden: false,
            only_folders: false,
            pattern: None,
        }
    }
}
