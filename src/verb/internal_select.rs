//! utility functions to help handle the `:select` internal

use {
    super::*,
    crate::{
        app::*,
        browser::BrowserState,
        command::TriggerType,
        display::Screen,
        path::{self, PathAnchor},
        tree::Tree,
    },
    std::path::{Path, PathBuf},
};


/// general implementation for verbs based on the :select internal with optionally
/// a bang or an argument.
pub fn on_internal(
    internal_exec: &InternalExecution,
    input_invocation: Option<&VerbInvocation>,
    trigger_type: TriggerType,
    tree: &mut Tree,
    app_state: & AppState,
    cc: &CmdContext,
) -> CmdResult {
    let screen = cc.app.screen;
    info!(
        "internal_select.on_internal internal_exec={:?} input_invocation={:?} trygger_type={:?}",
        internal_exec,
        input_invocation,
        trigger_type,
    );
    let bang = input_invocation
            .map(|inv| inv.bang)
            .unwrap_or(internal_exec.bang);
    let input_arg = input_invocation.as_ref()
        .and_then(|invocation| invocation.args.as_ref());
    match trigger_type {
        TriggerType::Input(verb) => {
            let path = path_from_input(
                verb,
                internal_exec,
                &tree.selected_line().path,
                input_arg,
                app_state,
            );
            on_path(path, tree, screen, bang)
        }
        _ => {
            // the :select internal was triggered by a key
            if let Some(arg) = &internal_exec.arg {
                // the internal_execution specifies the path to use
                // (it may come from a configured verb whose execution is
                //  `:select some/path`).
                // The given path may be relative hence the need for the
                // state's selection
                let path = path::path_from(
                    &tree.selected_line().path,
                    PathAnchor::Unspecified,
                    arg,
                );
                let bang = input_invocation
                    .map(|inv| inv.bang)
                    .unwrap_or(internal_exec.bang);
                on_path(path, tree, screen, bang)
            } else {
                // there's nothing really to do here
                CmdResult::Keep
            }
        }
    }
}


/// Compute the path to go to in case of the internal being triggered from
/// the input.
///
/// This path depends on the verb (which may hardcore the path or have a
/// pattern), from the selection,
fn path_from_input(
    verb: &Verb,
    internal_exec: &InternalExecution,
    base_path: &Path, // either the selected path or the root path
    input_arg: Option<&String>,
    app_state: &AppState,
) -> PathBuf {
    match (input_arg, internal_exec.arg.as_ref()) {
        (Some(input_arg), Some(verb_arg)) => {
            // The verb probably defines some patttern which uses the input.
            // For example:
            // {
            //     invocation: "gotar {path}"
            //     execution: ":select {path}/target"
            // }
            // (or that input is useless)
            let path_builder = ExecutionStringBuilder::with_invocation(
                &verb.invocation_parser,
                SelInfo::from_path(base_path),
                app_state,
                Some(input_arg),
            );
            path_builder.path(verb_arg)
        }
        (Some(input_arg), None) => {
            // the verb defines nothing
            // The :select internal execution was triggered from the
            // input (which must be a kind of alias for :select)
            // so we do exactly what the input asks for
            path::path_from(base_path, PathAnchor::Unspecified, input_arg)
        }
        (None, Some(verb_arg)) => {
            // the verb defines the path where to go..
            // the internal_execution specifies the path to use
            // (it may come from a configured verb whose execution is
            //  `:select some/path`).
            // The given path may be relative hence the need for the
            // state's selection
            // (we assume a check before ensured it doesn't need an input)
            path::path_from(base_path, PathAnchor::Unspecified, verb_arg)
        }
        (None, None) => {
            // This doesn't really make sense: we're selecting the currently
            // selected path
            base_path.to_path_buf()
        }
    }

}

pub fn on_path(
    path: PathBuf,
    tree: &mut Tree,
    screen: Screen,
    in_new_panel: bool,
) -> CmdResult {
    info!("executing :select on path {:?}", &path);
    if in_new_panel {
        warn!("bang in :select isn't supported yet");
    }
    if tree.try_select_path(&path) {
        tree.make_selection_visible(BrowserState::page_height(screen));
    }
    CmdResult::Keep
}
