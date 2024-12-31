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
        set_title(w, pattern, app_state, con);
    }
}

/// Reset the terminal's title to its default value (which may be the one
/// just before broot was launched, but may also be different)
pub fn reset_title(
    w: &mut W,
    con: &AppContext,
) {
    if con.terminal_title_pattern.is_some() && con.reset_terminal_title_on_exit {
        let _ = write!(w, "\u{1b}]2;\u{07}");
        let _ = w.flush();
    }
}

fn set_title(
    w: &mut W,
    pattern: &ExecPattern,
    app_state: &AppState,
    con: &AppContext,
) {
    let builder = ExecutionStringBuilder::without_invocation(
        SelInfo::from_path(&app_state.root),
        app_state,
    );
    let title = builder.shell_exec_string(pattern, con);
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

