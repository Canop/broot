use {
    crate::{
        app::*,
        display::W,
        verb::*,
    },
    std::io::Write,
};

/// Change the terminal's title if broot was configured with
/// a `terminal_title` entry
#[inline]
pub fn update_title(
    w: &mut W,
    app_state: &AppState,
    con: &AppContext,
) {
    if let Some(pattern) = &con.terminal_title_pattern {
        set_title(w, pattern, app_state);
    }
}

fn set_title(
    w: &mut W,
    pattern: &ExecPattern,
    app_state: &AppState,
) {
    let builder = ExecutionStringBuilder::without_invocation(
        SelInfo::from_path(&app_state.root),
        app_state,
    );
    let title = builder.shell_exec_string(pattern);
    set_title_str(w, &title)
}

#[inline]
fn set_title_str(
    w: &mut W,
    title: &str,
) {
    let _ = write!(w, "\u{1b}]0;{title}\u{07}");
    let _ = w.flush();
}
