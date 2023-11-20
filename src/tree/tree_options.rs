use {
    super::Sort,
    crate::{
        cli::Args,
        conf::Conf,
        display::{Cols, DEFAULT_COLS},
        errors::ConfError,
        pattern::*,
    },
    std::convert::TryFrom,
};

/// Options defining how the tree should be build and|or displayed
#[derive(Debug, Clone)]
pub struct TreeOptions {
    pub show_selection_mark: bool, // whether to have a triangle left of selected line
    pub show_hidden: bool, // whether files whose name starts with a dot should be shown
    pub only_folders: bool, // whether to hide normal files and links
    pub show_counts: bool, // whether to show the number of files (> 1 only for dirs)
    pub show_dates: bool,  // whether to show the last modified date
    pub show_sizes: bool,  // whether to show sizes of files and dirs
    pub show_git_file_info: bool,
    pub show_device_id: bool,
    pub show_root_fs: bool, // show information relative to the fs of the root
    pub trim_root: bool,    // whether to cut out direct children of root
    pub show_permissions: bool, // show classic rwx unix permissions (only on unix)
    pub respect_git_ignore: bool, // hide files as requested by .gitignore ?
    pub filter_by_git_status: bool, // only show files whose git status is not nul
    pub pattern: InputPattern, // an optional filtering/scoring pattern
    pub date_time_format: &'static str,
    pub sort: Sort,
    pub show_tree: bool, // whether to show the tree
    pub cols_order: Cols, // order of columns
    pub show_matching_characters_on_path_searches: bool,
}

impl TreeOptions {
    /// clone self but without the pattern (if any)
    pub fn without_pattern(&self) -> Self {
        TreeOptions {
            show_selection_mark: self.show_selection_mark,
            show_hidden: self.show_hidden,
            only_folders: self.only_folders,
            show_counts: self.show_counts,
            show_dates: self.show_dates,
            show_sizes: self.show_sizes,
            show_permissions: self.show_permissions,
            respect_git_ignore: self.respect_git_ignore,
            filter_by_git_status: self.filter_by_git_status,
            show_git_file_info: self.show_git_file_info,
            show_device_id: self.show_device_id,
            show_root_fs: self.show_root_fs,
            trim_root: self.trim_root,
            pattern: InputPattern::none(),
            date_time_format: self.date_time_format,
            sort: self.sort,
            show_tree: self.show_tree,
            cols_order: self.cols_order,
            show_matching_characters_on_path_searches: self.show_matching_characters_on_path_searches,
        }
    }
    /// counts must be computed, either for sorting or just for display
    pub fn needs_counts(&self) -> bool {
        self.show_counts || self.sort == Sort::Count
    }
    /// dates must be computed, either for sorting or just for display
    pub fn needs_dates(&self) -> bool {
        self.show_dates || self.sort == Sort::Date
    }
    /// sizes must be computed, either for sorting or just for display
    pub fn needs_sizes(&self) -> bool {
        self.show_sizes || self.sort == Sort::Size
    }
    pub fn needs_sum(&self) -> bool {
        self.needs_counts() || self.needs_dates() || self.needs_sizes()
    }
    /// this method does not exist, you saw nothing
    /// (at least don't call it other than with the config, once)
    pub fn set_date_time_format(&mut self, format: String) {
        self.date_time_format = Box::leak(format.into_boxed_str());
    }
    /// change tree options according to configuration
    /// (but not the default_flags part, which is handled separately)
    pub fn apply_config(&mut self, config: &Conf) -> Result<(), ConfError> {
        if let Some(b) = config.show_selection_mark {
            self.show_selection_mark = b;
        }
        if let Some(format) = &config.date_time_format {
            self.set_date_time_format(format.clone());
        }
        if let Some(b) = config.show_matching_characters_on_path_searches {
            self.show_matching_characters_on_path_searches = b;
        }
        self.cols_order = config
            .cols_order
            .as_ref()
            .map(Cols::try_from)
            .transpose()?
            .unwrap_or(DEFAULT_COLS);
        Ok(())
    }
    /// change tree options according to broot launch arguments
    pub fn apply_launch_args(&mut self, cli_args: &Args) {
        if cli_args.sizes {
            self.show_sizes = true;
            self.show_root_fs = true;
        } else if cli_args.no_sizes {
            self.show_sizes = false;
        }
        if cli_args.whale_spotting {
            self.show_hidden = true;
            self.respect_git_ignore = false;
            self.sort = Sort::Size;
            self.show_sizes = true;
            self.show_root_fs = true;
        }
        if cli_args.only_folders {
            self.only_folders = true;
        } else if cli_args.no_only_folders {
            self.only_folders = false;
        }
        if cli_args.git_status {
            self.filter_by_git_status = true;
            self.show_hidden = true;
        }
        if cli_args.hidden {
            self.show_hidden = true;
        } else if cli_args.no_hidden {
            self.show_hidden = false;
        }
        if cli_args.dates {
            self.show_dates = true;
        } else if cli_args.no_dates {
            self.show_dates = false;
        }
        if cli_args.permissions {
            self.show_permissions = true;
        } else if cli_args.no_permissions {
            self.show_permissions = false;
        }
        if cli_args.show_root_fs {
            self.show_root_fs = true;
        }
        if cli_args.git_ignored {
            self.respect_git_ignore = false;
        } else if cli_args.no_git_ignored {
            self.respect_git_ignore = true;
        }
        if cli_args.show_git_info {
            self.show_git_file_info = true;
        } else if cli_args.no_show_git_info {
            self.show_git_file_info = false;
        }
        if cli_args.sort_by_count {
            self.sort = Sort::Count;
            self.show_counts = true;
        }
        if cli_args.sort_by_date {
            self.sort = Sort::Date;
            self.show_dates = true;
        }
        if cli_args.sort_by_size {
            self.sort = Sort::Size;
            self.show_sizes = true;
        }
        if cli_args.tree {
            self.show_tree = true;
        } else if cli_args.no_tree {
            self.show_tree = false;
        }
        if cli_args.sort_by_type_dirs_first || cli_args.sort_by_type {
            self.sort = Sort::TypeDirsFirst;
        }
        if cli_args.sort_by_type_dirs_last {
            self.sort = Sort::TypeDirsLast;
        }
        if cli_args.no_sort {
            self.sort = Sort::None;
        }
        if cli_args.trim_root {
            self.trim_root = true;
        } else if cli_args.no_trim_root {
            self.trim_root = false;
        }
    }
}

impl Default for TreeOptions {
    fn default() -> Self {
        Self {
            show_selection_mark: false,
            show_hidden: false,
            only_folders: false,
            show_counts: false,
            show_dates: false,
            show_sizes: false,
            show_git_file_info: false,
            show_device_id: false,
            show_root_fs: false,
            trim_root: false,
            show_permissions: false,
            respect_git_ignore: true,
            filter_by_git_status: false,
            pattern: InputPattern::none(),
            date_time_format: "%Y/%m/%d %R",
            sort: Sort::None,
            show_tree: true,
            cols_order: DEFAULT_COLS,
            show_matching_characters_on_path_searches: true,
        }
    }
}
