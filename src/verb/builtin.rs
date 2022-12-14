use {
    super::*,
    crate::{
        app::SelectionType,
    },
    crokey::*,
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
/// in standard (they still may be overridden by configuration)
pub fn builtin_verbs() -> Vec<Verb> {
    use super::{ExternalExecutionMode::*, Internal::*};
    vec![
        internal(back),

        // input actions, not visible in doc, but available for
        // example in remote control
        internal(input_clear).no_doc(),
        internal(input_del_char_left).no_doc(),
        internal(input_del_char_below).no_doc(),
        internal(input_del_word_left).no_doc(),
        internal(input_del_word_right).no_doc(),
        internal(input_go_to_end).with_key(key!(end)).no_doc(),
        internal(input_go_left).no_doc(),
        internal(input_go_right).no_doc(),
        internal(input_go_to_start).with_key(key!(home)).no_doc(),
        internal(input_go_word_left).no_doc(),
        internal(input_go_word_right).no_doc(),

        // arrow keys bindings
        internal(back).with_key(key!(left)),
        internal(open_stay).with_key(key!(right)),
        internal(line_down).with_key(key!(down)).with_key(key!('j')),
        internal(line_up).with_key(key!(up)).with_key(key!('k')),

        //
        internal(set_syntax_theme),

        // those two operations are mapped on ALT-ENTER, one
        // for directories and the other one for the other files
        internal(open_leave) // calls the system open
            .with_stype(SelectionType::File)
            .with_key(key!(alt-enter))
            .with_shortcut("ol"),
        external("cd", "cd {directory}", FromParentShell)
            .with_stype(SelectionType::Directory)
            .with_key(key!(alt-enter))
            .with_shortcut("ol")
            .with_description("change directory and quit"),

        #[cfg(unix)]
        external("chmod {args}", "chmod {args} {file}", StayInBroot)
            .with_stype(SelectionType::File),
        #[cfg(unix)]
        external("chmod {args}", "chmod -R {args} {file}", StayInBroot)
            .with_stype(SelectionType::Directory),
        internal(open_preview),
        internal(close_preview),
        internal(toggle_preview),
        internal(preview_image)
            .with_shortcut("img"),
        internal(preview_text)
            .with_shortcut("txt"),
        internal(preview_binary)
            .with_shortcut("hex"),
        internal(close_panel_ok),
        internal(close_panel_cancel)
            .with_key(key!(ctrl-w)),
        #[cfg(unix)]
        external(
            "copy {newpath}",
            "cp -r {file} {newpath:path-from-parent}",
            StayInBroot,
        )
            .with_shortcut("cp"),
        #[cfg(windows)]
        external(
            "copy {newpath}",
            "xcopy /Q /H /Y /I {file} {newpath:path-from-parent}",
            StayInBroot,
        )
            .with_shortcut("cp"),
        #[cfg(feature = "clipboard")]
        internal(copy_line)
            .with_key(key!(alt-c)),
        #[cfg(feature = "clipboard")]
        internal(copy_path),
        #[cfg(unix)]
        external(
            "copy_to_panel",
            "cp -r {file} {other-panel-directory}",
            StayInBroot,
        )
            .with_shortcut("cpp"),
        #[cfg(windows)]
        external(
            "copy_to_panel",
            "xcopy /Q /H /Y /I {file} {other-panel-directory}",
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
            .with_key(key!(L))  // hum... why this one ?
            .with_key(key!(ctrl-f)),
        internal(help)
            .with_key(key!(F1))
            .with_shortcut("?"),
        #[cfg(feature="clipboard")]
        internal(input_paste)
            .with_key(key!(ctrl-v)),
        #[cfg(unix)]
        external(
            "mkdir {subpath}",
            "mkdir -p {subpath:path-from-directory}",
            StayInBroot,
        )
            .with_shortcut("md"),
        #[cfg(windows)]
        external(
            "mkdir {subpath}",
            "cmd /c mkdir {subpath:path-from-directory}",
            StayInBroot,
        )
            .with_shortcut("md"),
        #[cfg(unix)]
        external(
            "move {newpath}",
            "mv {file} {newpath:path-from-parent}",
            StayInBroot,
        )
            .with_shortcut("mv"),
        #[cfg(windows)]
        external(
            "move {newpath}",
            "cmd /c move /Y {file} {newpath:path-from-parent}",
            StayInBroot,
        )
            .with_shortcut("mv"),
        #[cfg(unix)]
        external(
            "move_to_panel",
            "mv {file} {other-panel-directory}",
            StayInBroot,
        )
            .with_shortcut("mvp"),
        #[cfg(windows)]
        external(
            "move_to_panel",
            "cmd /c move /Y {file} {other-panel-directory}",
            StayInBroot,
        )
            .with_shortcut("mvp"),
        #[cfg(unix)]
        external(
            "rename {new_filename:file-name}",
            "mv {file} {parent}/{new_filename}",
            StayInBroot,
        )
            .with_auto_exec(false)
            .with_key(key!(f2)),
        #[cfg(windows)]
        external(
            "rename {new_filename:file-name}",
            "cmd /c move /Y {file} {parent}/{new_filename}",
            StayInBroot,
        )
            .with_auto_exec(false)
            .with_key(key!(f2)),
        internal_bang(start_end_panel)
            .with_key(key!(ctrl-p)),
        // the char keys for mode_input are handled differently as they're not
        // consumed by the command
        internal(mode_input)
            .with_key(key!(' '))
            .with_key(key!(':'))
            .with_key(key!('/')),
        internal(previous_match)
            .with_key(key!(shift-backtab))
            .with_key(key!(backtab)),
        internal(next_match)
            .with_key(key!(tab)),
        internal(no_sort)
            .with_shortcut("ns"),
        internal(open_stay)
            .with_key(key!(enter))
            .with_shortcut("os"),
        internal(open_stay_filter)
            .with_shortcut("osf"),
        internal(parent)
            .with_key(key!(h))
            .with_shortcut("p"),
        internal(page_down)
            .with_key(key!(ctrl-d))
            .with_key(key!(pagedown)),
        internal(page_up)
            .with_key(key!(ctrl-u))
            .with_key(key!(pageup)),
        internal(panel_left_no_open)
            .with_key(key!(ctrl-left)),
        internal(panel_right)
            .with_key(key!(ctrl-right)),
        internal(print_path).with_shortcut("pp"),
        internal(print_relative_path).with_shortcut("prp"),
        internal(print_tree).with_shortcut("pt"),
        internal(quit)
            .with_key(key!(ctrl-c))
            .with_key(key!(ctrl-q))
            .with_shortcut("q"),
        internal(refresh).with_key(key!(f5)),
        internal(root_up)
            .with_key(key!(ctrl-up)),
        internal(root_down)
            .with_key(key!(ctrl-down)),
        internal(select_first),
        internal(select_last),
        internal(select),
        internal(clear_stage).with_shortcut("cls"),
        internal(stage)
            .with_key(key!('+')),
        internal(unstage)
            .with_key(key!('-')),
        internal(stage_all_files)
            .with_key(key!(ctrl-a)),
        internal(toggle_stage)
            .with_key(key!(ctrl-g)),
        internal(open_staging_area).with_shortcut("osa"),
        internal(close_staging_area).with_shortcut("csa"),
        internal(toggle_staging_area).with_shortcut("tsa"),
        internal(sort_by_count).with_shortcut("sc"),
        internal(sort_by_date).with_shortcut("sd"),
        internal(sort_by_size).with_shortcut("ss"),
        internal(sort_by_type).with_shortcut("st"),
        #[cfg(unix)]
        external("rm", "rm -rf {file}", StayInBroot),
        #[cfg(windows)]
        external("rm", "cmd /c rmdir /Q /S {file}", StayInBroot)
            .with_stype(SelectionType::Directory),
        #[cfg(windows)]
        external("rm", "cmd /c del /Q {file}", StayInBroot)
            .with_stype(SelectionType::File),
        internal(toggle_counts).with_shortcut("counts"),
        internal(toggle_dates).with_shortcut("dates"),
        internal(toggle_device_id).with_shortcut("dev"),
        internal(toggle_files).with_shortcut("files"),
        internal(toggle_git_ignore)
            .with_key(key!(alt-i))
            .with_shortcut("gi"),
        internal(toggle_git_file_info).with_shortcut("gf"),
        internal(toggle_git_status).with_shortcut("gs"),
        internal(toggle_root_fs).with_shortcut("rfs"),
        internal(toggle_hidden)
            .with_key(key!(alt-h))
            .with_shortcut("h"),
        #[cfg(unix)]
        internal(toggle_perm).with_shortcut("perm"),
        internal(toggle_sizes).with_shortcut("sizes"),
        internal(toggle_trim_root),
        internal(total_search).with_key(key!(ctrl-s)),
        internal(up_tree).with_shortcut("up"),

    ]
}
