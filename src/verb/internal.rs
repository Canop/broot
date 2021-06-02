//! declare the internal functions which may be used in verbs.
//! They don't take any user argument other than the selection
//! (this may change if the needs arise).
//! They can be called as ":some_name" from builtin verbs and
//! from configured verbs.

use {
    crate::errors::ConfError,
};

macro_rules! Internals {
    (
        $($name:ident: $description:literal $need_path:literal,)*
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
            pub fn need_path(self) -> bool {
                use Internal::*;
                match self {
                    $($name => $need_path,)*
                }
            }
        }
    }
}


// internals:
//  name: "description" needs_a_path
Internals! {
    back: "revert to the previous state (mapped to *esc*)" false,
    close_panel_ok: "close the panel, validating the selected path" false,
    close_panel_cancel: "close the panel, not using the selected path" false,
    copy_line: "copy selected line (in tree or preview)" true,
    copy_path: "copy path to system clipboard" true,
    filesystems: "list mounted filesystems" false,
    focus: "display the directory (mapped to *enter*)" true,
    help: "display broot's help" false,
    input_clear: "empty the input" false,
    input_del_char_left: "delete the char left of the cursor" false,
    input_del_char_below: "delete the char left at the cursor's position" false,
    input_del_word_left: "delete the word left of the cursor" false,
    input_del_word_right: "delete the word right of the cursor" false,
    input_go_to_end: "move the cursor to the end of input" false,
    input_go_left: "move the cursor to the left" false,
    input_go_right: "move the cursor to the right" false,
    input_go_to_start: "move the cursor to the start of input" false,
    input_go_word_left: "move the cursor one word to the left" false,
    input_go_word_right: "move the cursor one word to the right" false,
    input_paste: "paste the clipboard content into the input" false,
    line_down: "move one line down" false,
    line_up: "move one line up" false,
    line_down_no_cycle: "move one line down" false,
    line_up_no_cycle: "move one line up" false,
    open_stay: "open file or directory according to OS (stay in broot)" true,
    open_stay_filter: "display the directory, keeping the current pattern" true,
    open_leave: "open file or directory according to OS (quit broot)" true,
    mode_input: "enter the input mode" false,
    mode_command: "enter the command mode" false,
    next_match: "select the next match" false,
    next_same_depth: "select the next file at the same depth" false,
    no_sort: "don't sort" false,
    page_down: "scroll one page down" false,
    page_up: "scroll one page up" false,
    parent: "move to the parent directory" false,
    panel_left: "focus panel on left" false,
    panel_right: "focus panel on right" false,
    previous_match: "select the previous match" false,
    previous_same_depth: "select the previous file at the same depth" false,
    open_preview: "open the preview panel" true,
    close_preview: "close the preview panel" false,
    toggle_preview: "open/close the preview panel" false,
    preview_image: "preview the selection as image" true,
    preview_text: "preview the selection as text" true,
    preview_binary: "preview the selection as binary" true,
    print_path: "print path and leaves broot" true,
    print_relative_path: "print relative path and leaves broot" true,
    print_tree: "print tree and leaves broot" true,
    start_end_panel: "either open or close an additional panel" true,
    quit: "quit Broot" false,
    refresh: "refresh tree and clear size cache" false,
    //restore_pattern: "restore a pattern which was just removed" false,
    select_first: "select the first item" false,
    select_last: "select the last item" false,
    sort_by_count: "sort by count" false,
    sort_by_date: "sort by date" false,
    sort_by_size: "sort by size" false,
    clear_stage: "empty the staging area" false,
    stage: "add selection to staging area" true,
    unstage: "remove selection from staging area" true,
    open_staging_area: "open the staging area" false,
    close_staging_area: "close the staging area panel" false,
    toggle_staging_area: "open/close the staging area panel" false,
    toggle_stage: "add or remove selection to staging area" true,
    toggle_counts: "toggle showing number of files in directories" false,
    toggle_dates: "toggle showing last modified dates" false,
    toggle_files: "toggle showing files (or just folders)" false,
    toggle_git_ignore: "toggle use of .gitignore" false,
    toggle_git_file_info: "toggle display of git file information" false,
    toggle_git_status: "toggle showing only files relevant for git status" false,
    toggle_root_fs: "toggle showing filesystem info on top" false,
    toggle_hidden: "toggle showing hidden files" false,
    toggle_perm: "toggle showing file permissions" false,
    toggle_sizes: "toggle showing sizes" false,
    toggle_trim_root: "toggle removing nodes at first level too" false,
    toggle_second_tree: "toggle display of a second tree panel" true,
    total_search: "search again but on all children" false,
    up_tree: "focus the parent of the current root" true,
}

impl Internal {
    pub fn invocation_pattern(self) -> &'static str {
        match self {
            Internal::focus => r"focus (?P<path>.*)?",
            Internal::line_down => r"line_down (?P<count>\d*)?",
            Internal::line_up => r"line_up (?P<count>\d*)?",
            Internal::line_down_no_cycle => r"line_down_no_cycle (?P<count>\d*)?",
            Internal::line_up_no_cycle => r"line_up_no_cycle (?P<count>\d*)?",
            _ => self.name(),
        }
    }
    pub fn exec_pattern(self) -> &'static str {
        match self {
            Internal::focus => r"focus {path}",
            Internal::line_down => r"line_down {count}",
            Internal::line_up => r"line_up {count}",
            Internal::line_down_no_cycle => r"line_down_no_cycle {count}",
            Internal::line_up_no_cycle => r"line_up_no_cycle {count}",
            _ => self.name(),
        }
    }
    pub fn needs_selection(self, arg: &Option<String>) -> bool {
        match self {
            Internal::focus => arg.is_none(),
            _ => self.need_path(),
        }
    }
}
