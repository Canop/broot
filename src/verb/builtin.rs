use {
    super::*,
    crate::{
        app::SelectionType,
        keys::*,
    },
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};

fn build_internal(
    internal: Internal,
    bang: bool,
) -> Verb {
    let invocation = internal.invocation_pattern();
    let execution = VerbExecution::Internal(
        InternalExecution::from_internal_bang(internal, bang)
    );
    let description = VerbDescription::from_text(internal.description().to_string());
    Verb::new(Some(invocation), execution, description).unwrap()
}

fn internal(
    internal: Internal,
) -> Verb {
    build_internal(internal, false)
}

fn internal_bang(
    internal: Internal,
) -> Verb {
    build_internal(internal, true)
}

fn external(
    invocation_str: &str,
    execution_str: &str,
    exec_mode: ExternalExecutionMode,
) -> Verb {
    let execution = VerbExecution::External(
        ExternalExecution::new(ExecPattern::from_string(execution_str), exec_mode)
    );
    Verb::new(
        Some(invocation_str),
        execution,
        VerbDescription::from_code(execution_str.to_string()),
    ).unwrap()
}

/// declare the built_in verbs, the ones which are available
/// in standard (they still may be overriden by configuration)
pub fn builtin_verbs() -> Vec<Verb> {
    use super::{ExternalExecutionMode::*, Internal::*};
    vec![
        internal(back),

        // input actions, not visible in doc, but available for
        // exemple in remote control
        internal(input_clear).no_doc(),
        internal(input_del_char_left).no_doc(),
        internal(input_del_char_below).no_doc(),
        internal(input_del_word_left).no_doc(),
        internal(input_del_word_right).no_doc(),
        internal(input_go_to_end).no_doc(),
        internal(input_go_left).no_doc(),
        internal(input_go_right).no_doc(),
        internal(input_go_to_start).no_doc(),
        internal(input_go_word_left).no_doc(),
        internal(input_go_word_right).no_doc(),

        // those two operations are mapped on ALT-ENTER, one
        // for directories and the other one for the other files
        external("cd", "cd {directory}", FromParentShell)
            .with_stype(SelectionType::Directory)
            .with_key(ALT_ENTER)
            .with_description("change directory and quit"),
        internal(open_leave) // calls the system open
            .with_key(ALT_ENTER)
            .with_shortcut("ol"),

        #[cfg(unix)]
        external("chmod {args}", "chmod {args} {file}", StayInBroot)
            .with_stype(SelectionType::File),
        #[cfg(unix)]
        external("chmod {args}", "chmod -R {args} {file}", StayInBroot)
            .with_stype(SelectionType::Directory),
        internal(open_preview),
        internal(close_preview),
        internal(toggle_preview),
        internal(preview_image),
        internal(preview_text),
        internal(preview_binary),
        internal(close_panel_ok),
        internal(close_panel_cancel)
            .with_control_key('w'),
        external(
            "copy {newpath}",
            "cp -r {file} {newpath:path-from-parent}",
            StayInBroot,
        )
            .with_shortcut("cp"),
        #[cfg(feature = "clipboard")]
        internal(copy_line)
            .with_alt_key('c'),
        #[cfg(feature = "clipboard")]
        internal(copy_path),
        external(
            "copy_to_panel",
            "cp -r {file} {other-panel-directory}",
            StayInBroot,
        )
            .with_shortcut("cpp"),
        #[cfg(unix)]
        internal(filesystems)
            .with_shortcut("fs"),
        // :focus is also hardcoded on Enter on directories
        // but ctrl-f is useful for focusing on a file's parent
        // (and keep the filter)
        internal(focus)
            .with_char_key('l')
            .with_control_key('f'),
        internal(help)
            .with_key(F1).with_shortcut("?"),
        #[cfg(feature="clipboard")]
        internal(input_paste)
            .with_control_key('v'),
        internal(line_down)
            .with_key(DOWN)
            .with_char_key('j'),
        internal(line_up)
            .with_key(UP)
            .with_char_key('k'),
        external(
            "mkdir {subpath}",
            "mkdir -p {subpath:path-from-directory}",
            StayInBroot,
        )
            .with_shortcut("md"),
        external(
            "move {newpath}",
            "mv {file} {newpath:path-from-parent}",
            StayInBroot,
        )
            .with_shortcut("mv"),
        external(
            "move_to_panel",
            "mv {file} {other-panel-directory}",
            StayInBroot,
        )
            .with_shortcut("mvp"),
        external(
            "rename {new_filename:file-name}",
            "mv {file} {parent}/{new_filename}",
            StayInBroot,
        )
            .with_auto_exec(false)
            .with_key(F2),
        internal_bang(start_end_panel)
            .with_control_key('p'),
        // the char keys for mode_input are handled differently as they're not
        // consumed by the command
        internal(mode_input)
            .with_char_key(' ')
            .with_char_key(':')
            .with_char_key('/'),
        internal(previous_match)
            .with_key(BACK_TAB),
        internal(next_match)
            .with_key(TAB),
        internal(no_sort)
            .with_shortcut("ns"),
        internal(open_stay)
            .with_key(ENTER)
            .with_shortcut("os"),
        internal(open_stay_filter)
            .with_shortcut("osf"),
        internal(parent)
            .with_char_key('h')
            .with_shortcut("p"),
        internal(page_down)
            .with_control_key('d')
            .with_key(PAGE_DOWN),
        internal(page_up)
            .with_control_key('u')
            .with_key(PAGE_UP),
        internal(panel_left)
            .with_key(KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::CONTROL,
            }),
        internal(panel_right)
            .with_key(KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::CONTROL,
            }),
        internal(print_path).with_shortcut("pp"),
        internal(print_relative_path).with_shortcut("prp"),
        internal(print_tree).with_shortcut("pt"),
        internal(quit)
            .with_control_key('c')
            .with_control_key('q')
            .with_shortcut("q"),
        internal(refresh).with_key(F5),
        internal(select_first).with_key(HOME),
        internal(select_last).with_key(END),
        internal(clear_stage).with_shortcut("cls"),
        internal(stage)
            .with_char_key('+'),
        internal(unstage)
            .with_char_key('-'),
        internal(toggle_stage)
            .with_control_key('g'),
        internal(open_staging_area).with_shortcut("osa"),
        internal(close_staging_area).with_shortcut("csa"),
        internal(toggle_staging_area).with_shortcut("tsa"),
        internal(sort_by_count).with_shortcut("sc"),
        internal(sort_by_date).with_shortcut("sd"),
        internal(sort_by_size).with_shortcut("ss"),
        external("rm", "rm -rf {file}", StayInBroot),
        internal(toggle_counts).with_shortcut("counts"),
        internal(toggle_dates).with_shortcut("dates"),
        internal(toggle_files).with_shortcut("files"),
        internal(toggle_git_ignore).with_shortcut("gi"),
        internal(toggle_git_file_info).with_shortcut("gf"),
        internal(toggle_git_status).with_shortcut("gs"),
        internal(toggle_root_fs).with_shortcut("rfs"),
        internal(toggle_hidden).with_shortcut("h"),
        #[cfg(unix)]
        internal(toggle_perm).with_shortcut("perm"),
        internal(toggle_sizes).with_shortcut("sizes"),
        internal(toggle_trim_root),
        internal(total_search).with_control_key('s'),
        internal(up_tree).with_shortcut("up"),
    ]
}
