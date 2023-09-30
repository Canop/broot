use {
    super::*,
    crate::{
        verb::{
            Internal,
            VerbStore,
        },
    },
};

/// All the precomputed status which don't involve a verb
pub struct StandardStatus {
    tree_top_focus: String, // go up (if not at root)
    tree_dir_focus: String,
    tree_dir_cd: Option<String>, // TODO check outcmd
    tree_file_open_stay: Option<String>,
    tree_file_open_stay_long: Option<String>,
    tree_file_open_leave: Option<String>,
    tree_unfiltered: String,
    tree_filtered: String,
    preview_unfiltered: String, // ctrl-left to close, or a pattern to filter
    preview_filtered: Option<String>,
    preview_restorable_filter: Option<String>,
    not_first_state: String, // "esc to go back"
    help: String,
    no_verb: String,
    pub all_files_hidden: Option<String>,
    pub all_files_git_ignored: Option<String>,
}

impl StandardStatus {
    pub fn new(verb_store: &VerbStore) -> Self {
        let tree_top_focus = "*enter* to go up".to_string(); // enter is hardcoded on focus
        let tree_dir_focus = "*enter* to focus".to_string();
        let tree_dir_cd = verb_store
            .key_desc_of_internal_stype(Internal::open_leave, SelectionType::Directory)
            .map(|k| format!("*{k}* to cd"));
        let tree_file_open_stay = verb_store
            .key_desc_of_internal_stype(Internal::open_stay, SelectionType::File)
            .map(|k| format!("*{k}* to open"));
        let tree_file_open_stay_long = verb_store
            .key_desc_of_internal_stype(Internal::open_stay, SelectionType::File)
            .map(|k| format!("*{k}* to open the file"));
        let tree_file_open_leave = verb_store
            .key_desc_of_internal_stype(Internal::open_leave, SelectionType::File)
            .map(|k| format!("*{k}* to open and quit"));
        //let tree_file_enter = None; // TODO (for when enter is customized)
        let tree_unfiltered = "a few letters to search".to_string();
        let tree_filtered = "*esc* to clear the filter".to_string();
        let preview_unfiltered = "a pattern to filter".to_string();
        let preview_filtered = verb_store
            .key_desc_of_internal(Internal::panel_right)
            .map(|k| format!("*{k}* to reveal the text"));
        let preview_restorable_filter = verb_store
            .key_desc_of_internal(Internal::panel_left_no_open)
            .map(|k| format!("*{k}* to restore the filter"));
        let not_first_state = "*esc* to go back".to_string();
        let help = "*?* for help".to_string();
        let no_verb = "a space then a verb".to_string();
        let all_files_hidden = verb_store
            .key_desc_of_internal(Internal::toggle_hidden)
            .map(|k| format!("Some files are hidden, use *{k}* to display them"));
        let all_files_git_ignored = verb_store
            .key_desc_of_internal(Internal::toggle_git_ignore)
            .map(|k| format!("Some files are git-ignored, use *{k}* to display them"));
        Self {
            tree_top_focus,
            tree_dir_focus,
            tree_dir_cd,
            tree_file_open_stay,
            tree_file_open_stay_long,
            tree_file_open_leave,
            //tree_file_enter,
            tree_unfiltered,
            tree_filtered,
            preview_unfiltered,
            preview_filtered,
            preview_restorable_filter,
            not_first_state,
            help,
            no_verb,
            all_files_hidden,
            all_files_git_ignored,
        }
    }
    pub fn builder<'s>(
        &'s self,
        state_type: PanelStateType,
        selection: Selection<'s>,
        width: usize, // available width
    ) -> StandardStatusBuilder<'s> {
        StandardStatusBuilder::new(self, state_type, selection, width)
    }
}

#[derive(Default)]
struct StatusParts<'b> {
    md_parts: Vec<&'b str>,
}
impl<'b> StatusParts<'b> {
    fn add(&mut self, md: &'b str) {
        self.md_parts.push(md);
    }
    fn addo(&mut self, md: &'b Option<String>) {
        if let Some(md) = md {
            self.md_parts.push(md);
        }
    }
    fn len(&self) -> usize {
        self.md_parts.len()
    }
    /// Build the markdown of the complete status by combining parts
    /// while not going much over the available width so that we
    /// don't have too much elision (otherwise it would be too hard to read)
    fn to_status(&self, available_width: usize) -> Status {
        let mut md = String::new();
        // notes about the truncation:
        // - in case of truncation, we don't use the long ", or "
        //   separator. It's OK, assuming truncation is only for
        //   when the screen is very small, and not the standard case.
        let mut sum_len = 0;
        let max_len = available_width + 3;
        for (i, p) in self.md_parts.iter().enumerate() {
            let sep = if i == 0 {
                "Hit "
            } else if i == self.md_parts.len() - 1 {
                ", or "
            } else {
                ", "
            };
            sum_len += sep.len() + p.len() - 2; // -2 is an estimate of hidden chars
            if i > 0 && sum_len > max_len {
                break;
            }
            md.push_str(sep);
            md.push_str(p);
        }
        Status::from_message(md)
    }
}

pub struct StandardStatusBuilder<'s> {
    ss: &'s StandardStatus,
    state_type: PanelStateType,
    selection: Selection<'s>,
    pub has_previous_state: bool,
    pub is_filtered: bool,
    pub has_removed_pattern: bool,
    pub on_tree_root: bool, // should this be part of the Selection struct ?
    pub width: usize, // available width
}
impl<'s> StandardStatusBuilder<'s> {
    fn new(
        ss: &'s StandardStatus,
        state_type: PanelStateType,
        selection: Selection<'s>,
        width: usize, // available width
    ) -> Self {
        Self {
            ss,
            state_type,
            selection,
            has_previous_state: true,
            is_filtered: false,
            has_removed_pattern: false,
            on_tree_root: false,
            width,
        }
    }
    pub fn status(self) -> Status {
        let ss = &self.ss;
        let mut parts = StatusParts::default();
        if self.has_previous_state && !self.is_filtered {
            parts.add(&ss.not_first_state);
        }
        match self.state_type {
            PanelStateType::Tree => {
                if self.on_tree_root {
                    if self.selection.path.file_name().is_some() { // it's not '/'
                        parts.add(&ss.tree_top_focus);
                    }
                } else if self.selection.stype == SelectionType::Directory {
                    parts.add(&ss.tree_dir_focus);
                    parts.addo(&ss.tree_dir_cd);
                } else if self.selection.stype == SelectionType::File {
                    // maybe add "ctrl-right to preview" ? Or just sometimes ?
                    //  (need check no preview)
                    if self.width > 105 {
                        parts.addo(&ss.tree_file_open_stay_long);
                    } else {
                        parts.addo(&ss.tree_file_open_stay);
                    }
                    parts.addo(&ss.tree_file_open_leave);
                }
                if self.is_filtered {
                    parts.add(&ss.tree_filtered);
                }
                if parts.len() < 3 {
                    parts.add(&ss.help);
                }
                if parts.len() < 4 {
                    if self.on_tree_root && !self.is_filtered {
                        parts.add(&ss.tree_unfiltered);
                    } else {
                        parts.add(&ss.no_verb);
                    }
                }
            }
            PanelStateType::Preview => {
                if self.is_filtered {
                    parts.addo(&ss.preview_filtered);
                } else if self.has_removed_pattern {
                    parts.addo(&ss.preview_restorable_filter);
                } else {
                    parts.add(&ss.preview_unfiltered);
                }
                parts.add(&ss.no_verb);
            }
            PanelStateType::Help => {
                // not yet used, help_state has its own hard status
                if parts.len() < 4 {
                    parts.add(&ss.no_verb);
                }
            }
            PanelStateType::Fs => {
                warn!("TODO fs status");
            }
            PanelStateType::Stage => {
                warn!("TODO stage status");
            }
        }
        parts.to_status(self.width)
    }
}
