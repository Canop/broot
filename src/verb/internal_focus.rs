//! utility functions to help handle the `:focus` internal

use {
    super::*,
    crate::{
        app::*,
        browser::BrowserState,
        command::TriggerType,
        display::Screen,
        path::{self, PathAnchor},
        preview::PreviewState,
        task_sync::Dam,
        tree::TreeOptions,
        path::PathBufWrapper,
    },
    std::path::Path,
};

pub fn on_path(
    path: PathBufWrapper,
    screen: Screen,
    tree_options: TreeOptions,
    in_new_panel: bool,
    con: &AppContext,
) -> AppStateCmdResult {
    if in_new_panel {
        new_panel_on_path(
            path,
            screen,
            tree_options,
            PanelPurpose::None,
            con,
            HDir::Right,
        )
    } else {
        new_state_on_path(path, screen, tree_options, con)
    }
}

pub fn new_state_on_path(
    path: PathBufWrapper,
    screen: Screen,
    tree_options: TreeOptions,
    con: &AppContext,
) -> AppStateCmdResult {
    let path = path::closest_dir(&path);
    AppStateCmdResult::from_optional_state(
        BrowserState::new(path, tree_options, screen, con, &Dam::unlimited()),
        false,
    )
}

pub fn new_panel_on_path(
    path: PathBufWrapper,
    screen: Screen,
    tree_options: TreeOptions,
    purpose: PanelPurpose,
    con: &AppContext,
    direction: HDir,
) -> AppStateCmdResult {
    if purpose.is_preview() {
        let pattern = tree_options.pattern.tree_to_preview();
        AppStateCmdResult::NewPanel {
            state: Box::new(PreviewState::new(path, pattern, None, tree_options, con)),
            purpose,
            direction,
        }
    } else {
        let path = path::closest_dir(&path);
        match BrowserState::new(path, tree_options, screen, con, &Dam::unlimited()) {
            Ok(Some(os)) => AppStateCmdResult::NewPanel {
                state: Box::new(os),
                purpose,
                direction,
            },
            Ok(None) => AppStateCmdResult::Keep, // this isn't supposed to happen
            Err(e) => AppStateCmdResult::DisplayError(e.to_string()),
        }
    }
}

/// general implementation for verbs based on the :focus internal with optionally
/// a bang or an argument.
pub fn on_internal(
    internal_exec: &InternalExecution,
    input_invocation: Option<&VerbInvocation>,
    trigger_type: TriggerType,
    selected_path: &Path,
    screen: Screen,
    con: &AppContext,
    tree_options: TreeOptions,
) -> AppStateCmdResult {
    if let Some(arg) = &internal_exec.arg {
        // the internal_execution specifies the path to use
        // (it may come from a configured verb whose execution is
        //  `:focus some/path`).
        // The given path may be relative hence the need for the
        // state's selection
        let path = path::path_from(selected_path, PathAnchor::Unspecified, arg);
        let bang = input_invocation
            .map(|inv| inv.bang)
            .unwrap_or(internal_exec.bang);
        return on_path(path, screen, tree_options, bang, con);
    }
    if let Some(input_invocation) = &input_invocation {
        if let Some(input_arg) = &input_invocation.args {
            // user typed a path in the input
            match trigger_type {
                TriggerType::Input => {
                    // the :focus internal execution was triggered from the
                    // input (which must be a kind of alias for :focus)
                    // so we do exactly what the input asks for
                    let path = path::path_from(selected_path, PathAnchor::Unspecified, input_arg);
                    let bang = input_invocation.bang || internal_exec.bang;
                    return on_path(path, screen, tree_options, bang, con);
                }
                _ => {
                    // the :focus internal was triggered by a key, and without internal arg,
                    // which means the user wants to explore the arg with purpose
                    // of selecting a path
                    let base_dir = selected_path.to_string_lossy();
                    let path = path::path_from(&*base_dir, PathAnchor::Unspecified, input_arg);
                    let arg_type = SelectionType::Any; // We might do better later
                    let purpose = PanelPurpose::ArgEdition { arg_type };
                    return new_panel_on_path(path, screen, tree_options, purpose, con, HDir::Right);
                }
            }
        }
    }
    // user only wants to open the selected path, either in the same panel or
    // in a new one
    let bang = input_invocation
        .map(|inv| inv.bang)
        .unwrap_or(internal_exec.bang);
    on_path(selected_path.into(), screen, tree_options, bang, con)
}
