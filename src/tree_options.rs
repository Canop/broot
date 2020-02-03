use {
    crate::{
        patterns::Pattern,
    },
};

#[derive(Debug, Clone)]
pub struct TreeOptions {
    pub show_hidden: bool, // whether files whose name starts with a dot should be shown
    pub only_folders: bool, // whether to hide normal files and links
    pub show_sizes: bool,  // whether to compute and show sizes of files and dirs
    pub show_dates: bool,  // whether to show the last modified date
    pub show_git_file_info: bool,
    pub trim_root: bool,   // whether to cut out direct children of root
    pub show_permissions: bool, // show classic rwx unix permissions
    pub respect_git_ignore: bool, // hide files as requested by .gitignore ?
    pub filter_by_git_status: bool, // only show files whose git status is not nul
    pub pattern: Pattern,  // an optional filtering/scoring pattern
}

impl TreeOptions {
    pub fn without_pattern(&self) -> TreeOptions {
        TreeOptions {
            show_hidden: self.show_hidden,
            only_folders: self.only_folders,
            show_sizes: self.show_sizes,
            show_dates: self.show_dates,
            show_git_file_info: self.show_git_file_info,
            trim_root: self.trim_root,
            show_permissions: self.show_permissions,
            respect_git_ignore: self.respect_git_ignore,
            filter_by_git_status: self.filter_by_git_status,
            pattern: Pattern::None,
        }
    }
}

impl Default for TreeOptions {
    fn default() -> Self {
        Self {
            show_hidden: false,
            only_folders: false,
            show_sizes: false,
            show_dates: false,
            show_git_file_info: false,
            trim_root: true,
            show_permissions: false,
            respect_git_ignore: true,
            filter_by_git_status: false,
            pattern: Pattern::None,
        }
    }
}
