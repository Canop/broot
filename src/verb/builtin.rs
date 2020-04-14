
use {
    crate::{
        keys::*,
    },
    super::{
        Internal,
        Verb,
    },
};

/// declare the built_in verbs, the ones which are available
/// in standard (they still may be overriden by configuration)
pub fn builtin_verbs() -> Vec<Verb> {
    use {
        Internal::*,
    };
    vec![

        Verb::internal(back),

        Verb::external(
            "cd",
            "cd {directory}",
        ).unwrap()
        .with_description("change directory and quit (mapped to *alt*-*enter*)")
        .with_from_shell(true)
        .with_leave_broot(true),

        #[cfg(unix)]
        Verb::external(
            "chmod {args}",
            "chmod {args} {file}",
        ).unwrap(),

        Verb::external(
            "cp {newpath}",
            "/bin/cp -r {file} {newpath:path-from-parent}",
        ).unwrap(),

        Verb::internal(focus) // hardcoded Enter
        .with_shortcut("goto"),

        Verb::internal_bang(focus, true)
        .with_key(TAB),

        Verb::internal(focus_root),

        Verb::internal(help)
        .with_key(F1)
        .with_shortcut("?"),

        Verb::internal(line_down)
        .with_key(DOWN),

        Verb::internal(line_up)
        .with_key(UP),

        Verb::external(
            "mkdir {subpath}",
            "/bin/mkdir -p {subpath:path-from-directory}",
        ).unwrap()
        .with_shortcut("md"),

        Verb::external(
            "mv {newpath}",
            "/bin/mv {file} {newpath:path-from-parent}",
        ).unwrap(),

        Verb::internal(open_stay)
        .with_shortcut("os"),

        Verb::internal(open_leave)
        .with_shortcut("ol"),

        Verb::internal(parent)
        .with_shortcut("p"),

        Verb::internal(page_down)
        .with_key(PAGE_DOWN),

        Verb::internal(page_up)
        .with_key(PAGE_UP),

        Verb::internal(parent)
        .with_shortcut("p"),

        Verb::internal(print_path)
        .with_shortcut("pp"),

        Verb::internal(print_relative_path)
        .with_shortcut("prp"),

        Verb::internal(print_tree)
        .with_shortcut("pt"),

        Verb::internal(quit)
        .with_key(CTRL_C)
        .with_key(CTRL_Q)
        .with_shortcut("q"),

        Verb::internal(refresh)
        .with_key(F5),

        Verb::external(
            "rm",
            "/bin/rm -rf {file}",
        ).unwrap(),

        Verb::internal(toggle_dates)
        .with_shortcut("dates"),

        Verb::internal(toggle_files)
        .with_shortcut("files"),

        Verb::internal(toggle_git_ignore)
        .with_shortcut("gi"),

        Verb::internal(toggle_git_file_info)
        .with_shortcut("gf"),

        Verb::internal(toggle_git_status)
        .with_shortcut("gs"),

        Verb::internal(toggle_hidden)
        .with_shortcut("h"),

        #[cfg(unix)]
        Verb::internal(toggle_perm)
        .with_shortcut("perm"),

        Verb::internal(toggle_sizes)
        .with_shortcut("sizes"),

        Verb::internal(toggle_trim_root)
        .with_shortcut("t"),

        Verb::internal(total_search)
        .with_key(CTRL_S),

        Verb::internal(up_tree)
        .with_shortcut("up"),

    ]
}
