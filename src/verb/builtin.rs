use {
    super::Verb,
    crate::keys::*,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};

/// declare the built_in verbs, the ones which are available
/// in standard (they still may be overriden by configuration)
pub fn builtin_verbs() -> Vec<Verb> {
    use super::{ExternalExecutionMode::*, Internal::*};
    vec![
        Verb::internal(back),
        Verb::from(super::cd::CD.clone())
            .with_description("change directory and quit (mapped to *alt*-*enter*)"),
        #[cfg(unix)]
        Verb::external(
            "chmod {args}",
            "chmod {args} {file}",
            StayInBroot,
        ).unwrap(),
        Verb::internal(open_preview),
        Verb::internal(close_preview),
        Verb::internal(toggle_preview),
        Verb::internal(preview_image),
        Verb::internal(preview_text),
        Verb::internal(preview_binary),
        Verb::internal(close_panel_ok),
        Verb::internal(close_panel_cancel)
            .with_key(BACK_TAB)
            .with_control_key('w'),
        Verb::external(
            "copy {newpath:path-from-parent}",
            "/bin/cp -r {file} {newpath:path-from-parent}",
            StayInBroot,
        ).unwrap()
            .with_shortcut("cp"),
        #[cfg(feature="clipboard")]
        Verb::internal(copy_path)
            .with_alt_key('c'),
        Verb::external(
            "copy_to_panel",
            "/bin/cp -r {file} {other-panel-directory}",
            StayInBroot,
        ).unwrap()
            .with_shortcut("cpp"),
        // :focus is also hardcoded on Enter on directories
        // but ctrl-f is useful for focusing on a file's parent
        // (and keep the filter)
        Verb::internal(focus)
            .with_control_key('f'),
        Verb::internal(help).with_key(F1).with_shortcut("?"),
        #[cfg(feature="clipboard")]
        Verb::internal(input_paste)
            .with_control_key('v'),
        Verb::internal(line_down).with_key(DOWN),
        Verb::internal(line_up).with_key(UP),
        Verb::external(
            "mkdir {subpath}",
            "/bin/mkdir -p {subpath:path-from-directory}",
            StayInBroot,
        ).unwrap()
            .with_shortcut("md"),
        Verb::external(
            "move {newpath:path-from-parent}",
            "/bin/mv {file} {newpath:path-from-parent}",
            StayInBroot,
        ).unwrap()
            .with_shortcut("mv"),
        Verb::external(
            "move_to_panel",
            "/bin/mv {file} {other-panel-directory}",
            StayInBroot,
        ).unwrap()
            .with_shortcut("mvp"),
        Verb::internal_bang(start_end_panel)
            .with_control_key('p'),
        Verb::internal(next_match)
            .with_key(TAB),
        Verb::internal(no_sort)
            .with_shortcut("ns"),
        Verb::internal(open_stay)
            .with_key(ENTER)
            .with_shortcut("os"),
        Verb::internal(open_stay_filter)
            .with_shortcut("osf"),
        Verb::internal(open_leave)
            .with_key(ALT_ENTER)
            .with_shortcut("ol"),
        Verb::internal(parent).with_shortcut("p"),
        Verb::internal(page_down).with_key(PAGE_DOWN),
        Verb::internal(page_up).with_key(PAGE_UP),
        Verb::internal(panel_left)
            .with_key(KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::CONTROL,
            }),
        Verb::internal(panel_right)
            .with_key(KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::CONTROL,
            }),
        Verb::internal(print_path).with_shortcut("pp"),
        Verb::internal(print_relative_path).with_shortcut("prp"),
        Verb::internal(print_tree).with_shortcut("pt"),
        Verb::internal(quit)
            .with_control_key('c')
            .with_control_key('q')
            .with_shortcut("q"),
        Verb::internal(refresh).with_key(F5),
        Verb::internal(sort_by_count).with_shortcut("sc"),
        Verb::internal(sort_by_date).with_shortcut("sd"),
        Verb::internal(sort_by_size).with_shortcut("ss"),
        Verb::external(
            "rm",
            "/bin/rm -rf {file}",
            StayInBroot,
        ).unwrap(),
        Verb::internal(toggle_counts).with_shortcut("counts"),
        Verb::internal(toggle_dates).with_shortcut("dates"),
        Verb::internal(toggle_files).with_shortcut("files"),
        Verb::internal(toggle_git_ignore).with_shortcut("gi"),
        Verb::internal(toggle_git_file_info).with_shortcut("gf"),
        Verb::internal(toggle_git_status).with_shortcut("gs"),
        Verb::internal(toggle_hidden).with_shortcut("h"),
        #[cfg(unix)]
        Verb::internal(toggle_perm).with_shortcut("perm"),
        Verb::internal(toggle_sizes).with_shortcut("sizes"),
        Verb::internal(toggle_trim_root),
        Verb::internal(total_search).with_control_key('s'),
        Verb::internal(up_tree).with_shortcut("up"),
    ]
}
