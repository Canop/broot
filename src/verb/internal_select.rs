//! utility functions to help handle the `:select` internal

use {
    super::*,
    crate::{
        app::*,
        browser::BrowserState,
        command::TriggerType,
        display::Screen,
        tree::Tree,
    },
    std::path::PathBuf,
};

/// general implementation for verbs based on the :select internal with optionally
/// a bang or an argument.
pub fn on_internal(
    internal_exec: &InternalExecution,
    input_invocation: Option<&VerbInvocation>,
    trigger_type: TriggerType,
    tree: &mut Tree,
    app_state: &AppState,
    cc: &CmdContext,
) -> CmdResult {
    let Some(path) = internal_path::determine_path(
        internal_exec,
        input_invocation,
        trigger_type,
        tree,
        app_state,
        cc,
    ) else {
        return CmdResult::Keep;
    };
    let screen = cc.app.screen;
    let bang = input_invocation.map(|inv| inv.bang).unwrap_or(internal_exec.bang);
    on_path(path, tree, screen, bang)
}

pub fn on_path(
    path: PathBuf,
    tree: &mut Tree,
    screen: Screen,
    in_new_panel: bool,
) -> CmdResult {
    debug!(
        "executing :select on path {:?}",
        &path
    );
    if in_new_panel {
        warn!("bang in :select isn't supported yet");
    }
    if tree.try_select_path(&path) {
        tree.make_selection_visible(BrowserState::page_height(
            screen,
        ));
    }
    CmdResult::Keep
}
