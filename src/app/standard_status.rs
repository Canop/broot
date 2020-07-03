use {
    super::*,
    crate::{
        keys,
        selection_type::SelectionType,
        verb::{
            Internal,
            VerbStore,
        },
    },
    crossterm::event::KeyEvent,
};

/// the standard status that will be used when no verb is typed
pub struct StandardStatus {
    pub root_pat: Status,
    pub root_no_pat: Status,
    pub dir_pat: Status,
    pub dir_no_pat: Status,
    pub file_pat: Status,
    pub file_no_pat: Status,
}

impl StandardStatus {
    pub fn new(verb_store: &VerbStore) -> Self {
        Self {
            root_pat: Status::from_message(
                "Hit *esc* to remove the filter, *enter* to go up, '?' for help"
            ),
            root_no_pat: Status::from_message(
                "Hit *esc* to go back, *enter* to go up, *?* for help, or a few letters to search"
            ),
            dir_pat: compose_status(true, SelectionType::Directory, verb_store),
            dir_no_pat: compose_status(false, SelectionType::Directory, verb_store),
            file_pat: compose_status(true, SelectionType::File, verb_store),
            file_no_pat: compose_status(false, SelectionType::File, verb_store),
        }
    }
}

fn status_for(
    key: KeyEvent,
    selection_type: SelectionType,
    verb_store: &VerbStore,
) -> Option<String> {
    for verb in &verb_store.verbs {
        for verb_key in &verb.keys {
            if *verb_key == key {
                // add a 'input_related' property to internals & verbs ?
                if selection_type.respects(verb.selection_condition) {
                    let kd = keys::key_event_desc(key);
                    let s;
                    let action = match (verb.get_internal(), selection_type) {
                        (Some(Internal::open_stay), SelectionType::Directory) => "focus",
                        (Some(Internal::open_stay), SelectionType::File) => "open the file",
                        (Some(Internal::open_leave), SelectionType::Directory) => "cd",
                        (Some(Internal::open_leave), SelectionType::File) => "open and quit",
                        _ => {
                            if let Some(name) = verb.names.get(0) {
                                name
                            } else if verb.description.code {
                                s = format!("`{}`", &verb.description.content);
                                &s
                            } else {
                                &verb.description.content
                            }
                        }
                    };
                    return Some(format!("*{}* to {}", kd, action));

                }
            }
        }
    }
    None
}

fn compose_status(
    has_pattern: bool,
    selection_type: SelectionType,
    verb_store: &VerbStore,
) ->  Status {
    let mut parts = Vec::new();
    if let Some(md) = status_for(keys::ENTER, selection_type, verb_store) {
        parts.push(md);
    }
    if let Some(md) = status_for(keys::ALT_ENTER, selection_type, verb_store) {
        parts.push(md);
    }
    if has_pattern {
        parts.push("*esc* to clear the filter".to_string());
    } else if parts.len() < 2 {
        parts.push("a few letters to search".to_string());
    }
    if parts.len() < 4 {
        parts.push("or a space then a verb".to_string());
    }
    Status::from_message(format!("Hit {}", parts.join(", ")))
}


