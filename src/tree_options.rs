use crate::patterns::Pattern;

#[derive(Debug, Clone)]
pub struct TreeOptions {
    pub show_hidden: bool,
    pub only_folders: bool,
    pub show_sizes: bool,
    pub respect_git_ignore: bool,
    pub pattern: Option<Pattern>,
}

impl TreeOptions {
    pub fn new() -> TreeOptions {
        TreeOptions {
            show_hidden: false,
            only_folders: false,
            show_sizes: false,
            respect_git_ignore: true,
            pattern: None,
        }
    }
    pub fn without_pattern(&self) -> TreeOptions {
        TreeOptions {
            show_hidden: self.show_hidden,
            only_folders: self.only_folders,
            show_sizes: self.show_sizes,
            respect_git_ignore: self.respect_git_ignore,
            pattern: None,
        }
    }
}
