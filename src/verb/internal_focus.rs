use {
    super::*,
    crate::{
        app::*,
        browser::BrowserState,
        command::TriggerType,
        display::Screen,
        path,
        selection_type::SelectionType,
        task_sync::Dam,
        tree::TreeOptions,
    },
    std::path::{Path, PathBuf},
};

pub fn on_path(
    path: PathBuf,
    screen: &mut Screen,
    tree_options: TreeOptions,
    in_new_panel: bool,
) -> AppStateCmdResult {
    if in_new_panel {
        new_state_on_path(path, screen, tree_options)
    } else {
        new_panel_on_path(path, screen, tree_options, PanelPurpose::None)
    }
}

pub fn new_state_on_path(
    path: PathBuf,
    screen: &mut Screen,
    tree_options: TreeOptions,
) -> AppStateCmdResult {
    let path = path::closest_dir(&path);
    AppStateCmdResult::from_optional_state(
        BrowserState::new(path, tree_options, screen, &Dam::unlimited()),
        false,
    )
}

pub fn new_panel_on_path(
    path: PathBuf,
    screen: &mut Screen,
    tree_options: TreeOptions,
    purpose: PanelPurpose,
) -> AppStateCmdResult {
    let path = path::closest_dir(&path);
    match BrowserState::new(path, tree_options, screen, &Dam::unlimited()) {
        Ok(Some(os)) => {
            AppStateCmdResult::NewPanel {
                state: Box::new(os),
                purpose,
            }
        }
        Ok(None) => AppStateCmdResult::Keep, // this isn't supposed to happen
        Err(e) => AppStateCmdResult::DisplayError(e.to_string()),
    }
}

/// general implementation  for verbs based on the :focus internal with optionally
/// a bang or an argument.
pub fn on_internal(
    internal_exec: &InternalExecution,
    input_invocation: Option<&VerbInvocation>,
    trigger_type: TriggerType,
    selected_path: &Path,
    screen: &mut Screen,
    _con: &AppContext,
    tree_options: TreeOptions,
) -> AppStateCmdResult {
    if let Some(arg) = &internal_exec.arg {
        // the internal_execution specifies the path to use
        // (it may come from a configured verb whose execution is
        //  `:focus some/path`).
        // The given path may be relative hence the need for the
        // state's selection
        let base_dir = selected_path.to_string_lossy();
        let path = path::path_from(&base_dir, arg);
        let path = PathBuf::from(path);
        let bang = input_invocation
            .map(|inv| inv.bang)
            .unwrap_or(internal_exec.bang);
        return on_path(path, screen, tree_options, bang);
    }
    if let Some(input_invocation) = &input_invocation {
        if let Some(input_arg) = &input_invocation.args {
            // user typed a path in the input
            match trigger_type {
                TriggerType::Input => {
                    // the :focus internal execution was triggered from the
                    // input (which must be a kind of alias for :focus)
                    // so we do exactly what the input asks for
                    let base_dir = selected_path.to_string_lossy();
                    let path = path::path_from(&base_dir, input_arg);
                    let path = PathBuf::from(path);
                    let bang = input_invocation.bang || internal_exec.bang;
                    return on_path(path, screen, tree_options, bang);
                }
                _ => {
                    // the :focus internal was triggered by a key, which
                    // means the user wants to explore the arg with purpose
                    // of selecting a path
                    let base_dir = selected_path.to_string_lossy();
                    let path = path::path_from(&base_dir, input_arg);
                    let path = PathBuf::from(path);
                    let arg_type = SelectionType::Any; // We might do better later
                    let purpose = PanelPurpose::ArgEdition { arg_type };
                    return new_panel_on_path(path, screen, tree_options, purpose);
                }
            }
        }
    }
    // user only wants to open the selected path, either in the same panel or
    // in a new one
    let bang = input_invocation
        .map(|inv| inv.bang)
        .unwrap_or(internal_exec.bang);
    on_path(selected_path.to_path_buf(), screen, tree_options, bang)
}
