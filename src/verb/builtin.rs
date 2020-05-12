use {super::Verb, crate::keys::*};

/// declare the built_in verbs, the ones which are available
/// in standard (they still may be overriden by configuration)
pub fn builtin_verbs() -> Vec<Verb> {
    use super::{ExternalExecutionMode::*, Internal::*};
    vec![
        Verb::internal(back),
        Verb::from(super::cd::CD.clone())
            .with_description("change directory and quit (mapped to *alt*-*enter*)"),
        #[cfg(unix)]
        Verb::external("chmod {args}", "chmod {args} {file}", StayInBroot).unwrap(),
        Verb::internal(complete).with_key(TAB),
        Verb::internal(close_panel_ok)
            //.with_key(BACK_TAB),
            .with_control_key('w'),
        Verb::internal(close_panel_cancel),
        Verb::external(
            "cp {newpath}",
            "/bin/cp -r {file} {newpath:path-from-parent}",
            StayInBroot,
        )
        .unwrap(),
        Verb::internal(focus) // hardcoded Enter
            .with_shortcut("goto"),
        Verb::internal(help).with_key(F1).with_shortcut("?"),
        Verb::internal(line_down).with_key(DOWN),
        Verb::internal(line_up).with_key(UP),
        Verb::external(
            "mkdir {subpath}",
            "/bin/mkdir -p {subpath:path-from-directory}",
            StayInBroot,
        )
        .unwrap()
        .with_shortcut("md"),
        Verb::external(
            "mv {newpath}",
            "/bin/mv {file} {newpath:path-from-parent}",
            StayInBroot,
        )
        .unwrap(),
        Verb::internal_bang(focus)
            .with_control_key('t'),
        Verb::internal_bang(start_end_panel)
            .with_control_key('p'),
        Verb::internal(open_stay)
            .with_key(ENTER)
            .with_shortcut("os"),
        Verb::internal(open_leave)
            .with_key(ALT_ENTER)
            .with_shortcut("ol"),
        Verb::internal(parent).with_shortcut("p"),
        Verb::internal(page_down).with_key(PAGE_DOWN),
        Verb::internal(page_up).with_key(PAGE_UP),
        Verb::internal(parent).with_shortcut("p"),
        Verb::internal(print_path).with_shortcut("pp"),
        Verb::internal(print_relative_path).with_shortcut("prp"),
        Verb::internal(print_tree).with_shortcut("pt"),
        Verb::internal(quit)
            .with_control_key('c')
            .with_control_key('q')
            .with_shortcut("q"),
        Verb::internal(refresh).with_key(F5),
        Verb::external("rm", "/bin/rm -rf {file}", StayInBroot).unwrap(),
        Verb::internal(toggle_dates).with_shortcut("dates"),
        Verb::internal(toggle_files).with_shortcut("files"),
        Verb::internal(toggle_git_ignore).with_shortcut("gi"),
        Verb::internal(toggle_git_file_info).with_shortcut("gf"),
        Verb::internal(toggle_git_status).with_shortcut("gs"),
        Verb::internal(toggle_hidden).with_shortcut("h"),
        #[cfg(unix)]
        Verb::internal(toggle_perm).with_shortcut("perm"),
        Verb::internal(toggle_sizes).with_shortcut("sizes"),
        Verb::internal(toggle_trim_root).with_shortcut("t"),
        Verb::internal(total_search).with_key(CTRL_S),
        Verb::internal(up_tree).with_shortcut("up"),
    ]
}
