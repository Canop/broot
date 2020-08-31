//! declare the internal functions which may be used in verbs.
//! They don't take any user argument other than the selection
//! (this may change if the needs arise).
//! They can be called as ":some_name" from builtin verbs and
//! from configured verbs.

use {
    crate::errors::ConfError,
    std::path::Path,
};

macro_rules! Internals {
    (
        $($name:ident: $description:expr,)*
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq)]
        #[allow(non_camel_case_types)]
        pub enum Internal {
            $($name,)*
        }
        impl Internal {
            pub fn try_from(verb: &str) -> Result<Internal, ConfError> {
                use Internal::*;
                match verb {
                    $(stringify!($name) => Ok($name),)*
                    _ => Err(ConfError::UnknownInternal{ verb: verb.to_string() }),
                }
            }
        }
        impl Internal {
            pub fn name(self) -> &'static str {
                use Internal::*;
                match self {
                    $($name => stringify!($name),)*
                }
            }
            pub fn description(self) -> &'static str {
                use Internal::*;
                match self {
                    $($name => $description,)*
                }
            }
            pub fn applied_description(self, path: &Path) -> Option<String> {
                if self == Internal::focus {
                    Some(format!("focus `{}`", path.to_string_lossy()))
                } else {
                    None
                }
            }
        }
    }
}


Internals! {
    back: "revert to the previous state (mapped to *esc*)",
    close_panel_ok: "close the panel, validating the selected path",
    close_panel_cancel: "close the panel, not using the selected path",
    copy_path: "copy path to system clipboard (mapped to *alt-c*)",
    focus: "display the directory (mapped to *enter*)",
    help: "display broot's help",
    input_del_char_left: "delete the char left of the cursor",
    input_del_char_below: "delete the char left at the cursor's position",
    input_del_word_left: "delete the word left of the cursor",
    input_del_word_right: "delete the word right of the cursor",
    input_go_to_end: "move the cursor to the end of input",
    input_go_left: "move the cursor to the left",
    input_go_right: "move the cursor to the right",
    input_go_to_start: "move the cursor to the start of input",
    input_go_word_left: "move the cursor one word to the left",
    input_go_word_right: "move the cursor one word to the right",
    line_down: "move one line down",
    line_up: "move one line up",
    open_stay: "open file or directory according to OS (stay in broot)",
    open_stay_filter: "display the directory, keeping the current pattern",
    open_leave: "open file or directory according to OS (quit broot)",
    next_match: "select the next match",
    no_sort: "don't sort",
    page_down: "scroll one page down",
    page_up: "scroll one page up",
    parent: "move to the parent directory",
    panel_left: "focus panel on left",
    panel_right: "focus panel on right",
    previous_match: "select the previous match",
    open_preview: "open the preview panel",
    close_preview: "close the preview panel",
    toggle_preview: "open/close the preview panel",
    preview_image: "preview the selection as image",
    preview_text: "preview the selection as text",
    preview_binary: "preview the selection as binary",
    print_path: "print path and leaves broot",
    print_relative_path: "print relative path and leaves broot",
    print_tree: "print tree and leaves broot",
    start_end_panel: "either open or close an additional panel",
    quit: "quit Broot",
    refresh: "refresh tree and clear size cache",
    //restore_pattern: "restore a pattern which was just removed",
    select_first: "select the first file",
    select_last: "select the last file",
    sort_by_count: "sort by count",
    sort_by_date: "sort by date",
    sort_by_size: "sort by size",
    toggle_counts: "toggle showing number of files in directories",
    toggle_dates: "toggle showing last modified dates",
    toggle_files: "toggle showing files (or just folders)",
    toggle_git_ignore: "toggle use of .gitignore",
    toggle_git_file_info: "toggle display of git file information",
    toggle_git_status: "toggle showing only files relevant for git status",
    toggle_hidden: "toggle showing hidden files",
    toggle_perm: "toggle showing file permissions",
    toggle_sizes: "toggle showing sizes",
    toggle_trim_root: "toggle removing nodes at first level too",
    total_search: "search again but on all children",
    up_tree: "focus the parent of the current root",
}

impl Internal {
    /// whether this internal accept a path as (optional) argument
    pub fn accept_path(self) -> bool {
        match self {
            Internal::focus => true,
            _ => false,
        }
    }
}
