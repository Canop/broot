use crate::patterns::Pattern;

#[derive(Debug, Clone)]
pub struct TreeOptions {
    pub show_hidden: bool,
    pub only_folders: bool,
    pub show_sizes: bool,
    pub pattern: Option<Pattern>, // remove from there and make the treeOptions Copy ?
}

impl TreeOptions {
    pub fn new() -> TreeOptions {
        TreeOptions {
            show_hidden: false,
            only_folders: false,
            show_sizes: false,
            pattern: None,
        }
    }
    pub fn without_pattern(&self) -> TreeOptions {
        TreeOptions {
            show_hidden: self.show_hidden,
            only_folders: self.only_folders,
            show_sizes: self.show_sizes,
            pattern: None,
        }
    }
}
