use crossterm::KeyEvent::*;

use crate::{
    app_context::AppContext,
    conf::Conf,
};

/// build the markdown which will be displayed in the help page
pub fn build_markdown(con: &AppContext) -> String {
    let mut md = String::new();
    md.push_str(&format!("\n# broot v{}", env!("CARGO_PKG_VERSION")));
    md.push_str(MD_HELP_INTRO);
    md.push_str(MD_VERBS);
    append_verbs_table(&mut md, con);
    append_config_info(&mut md, con);
    md.push_str(MD_HELP_LAUNCH_ARGUMENTS);
    md.push_str(MD_HELP_FLAGS);
    md
}

const MD_HELP_INTRO: &str = r#"

**broot** lets you explore directory trees and launch commands.
It's best used when launched as **br**.
See *https://dystroy.org/broot* for a complete guide.

`<esc>` gets you back to the previous state.
Typing some letters searches the tree and selects the most relevant file.
To use a regular expression, use a slash at start or end eg `/j(ava|s)$`.
The **🡑** and **🡓** arrow keys can be used to change selection.
The mouse can be used to select (on click) or open (on double-click).
"#;

const MD_VERBS: &str = r#"
## Verbs

To execute a verb, type a space or `:` then start of its name or shortcut.
"#;

const MD_HELP_LAUNCH_ARGUMENTS: &str = r#"
## Launch Arguments

Some options can be set on launch:
* `-h` or `--hidden` : show hidden files
* `-f` or `--only-folders` : only show folders
* `-s` or `--sizes` : display sizes
* `-d` or `--dates` : display last modified dates
 (for the complete list, run `broot --help`)
"#;

const MD_HELP_FLAGS: &str = r#"
## Flags

Flags are displayed at bottom right:
* `h:y` or `h:n` : whether hidden files are shown
* `gi:a`, `gi:y`, `gi:n` : whether gitignore is on `auto`, `yes` or `no`
 When gitignore is auto, .gitignore rules are respected if the displayed root is a git repository or in one.

"#;

fn append_verbs_table(md: &mut String, con: &AppContext) {
    md.push_str("|-:\n");
    md.push_str("|**name**|**shortcut**|**key**|**description**\n");
    md.push_str("|-:|:-:|:-:|:-\n");
    for verb in &con.verb_store.verbs {
        md.push_str(&format!(
            "|{}|{}|",
            verb.invocation.key,
            if let Some(sk) = &verb.shortcut {
                &sk
            } else {
                ""
            },
        ));
        if let Some(key) = &verb.key {
            match key {
                F(d) => md.push_str(&format!("F{}", d)),
                Ctrl(c) => md.push_str(&format!("^{}", c)),
                Alt(c) => md.push_str(&format!("alt-{}", c)),
                _ => md.push_str(&format!("{:?}", key)),
            }
        }
        md.push_str("|");
        if let Some(s) = &verb.description {
            md.push_str(&format!("{}\n", &s));
        } else {
            md.push_str(&format!("`{}`\n", &verb.execution));
        }
    }
    md.push_str("|:-|-|:-\n");
}

fn append_config_info(md: &mut String, _con: &AppContext) {
    md.push_str(&format!(
        "\n## Configuration\n\nVerbs and skin can be configured in *{}*.\n",
        Conf::default_location().to_string_lossy()
    ));
}
