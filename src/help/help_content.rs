use {
    crate::{app::AppContext, verb::VerbExecution},
    minimad::{Text, TextTemplate},
};

static MD: &str = r#"

# broot ${version}

**broot** lets you explore directory trees and launch commands.
It's best used when launched as **br**.
See **https://dystroy.org/broot** for a complete guide.

The *esc* key gets you back to the previous state.
Typing some letters searches the tree and selects the most relevant file.
To use a regular expression, use a slash at start or end eg `/j(ava|s)$`.
The *↑* and *↓* arrow keys can be used to change selection.
The mouse can be used to select (on click) or open (on double-click).

## Verbs

To execute a verb, type a space or `:` then start of its name or shortcut.
|:-:|:-:|:-:|:-:
|**name**|**shortcut**|**key**|**description**
|-:|:-:|:-:|:-
${verb-rows
|${name}|${shortcut}|${key}|${description}`${execution}`
}
|-:

## Configuration

Verbs and skin can be configured in **${config-path}**.

## Launch Arguments

Some options can be set on launch:
* `-h` or `--hidden` : show hidden files
* `-f` or `--only-folders` : only show folders
* `-s` or `--sizes` : display sizes
* `-d` or `--dates` : display last modified dates
 (for the complete list, run `broot --help`)

## Flags

Flags are displayed at bottom right:
* `h:y` or `h:n` : whether hidden files are shown
* `gi:a`, `gi:y`, `gi:n` : whether gitignore is on `auto`, `yes` or `no`
 When gitignore is auto, .gitignore rules are respected if the displayed root is a git repository or in one.
"#;

/// build the markdown which will be displayed in the help page
pub fn build_text(con: &AppContext) -> Text<'_> {
    lazy_static! {
        // this doesn't really matter, only half a ms is spared
        static ref TEMPLATE: TextTemplate<'static> = TextTemplate::from(MD);
    }
    let mut expander = TEMPLATE.expander();
    expander
        .set("version", env!("CARGO_PKG_VERSION"))
        .set("config-path", &con.config_path);
    for verb in &con.verb_store.verbs {
        let sub = expander
            .sub("verb-rows")
            .set("name", &verb.name)
            .set(
                "shortcut",
                if let Some(sk) = &verb.shortcut {
                    &sk
                } else {
                    ""
                }, // TODO use as_deref when it's available
            )
            .set("key", &verb.keys_desc);
        if let Some(description) = &verb.description {
            sub.set_md("description", description);
            sub.set("execution", "");
        } else {
            match &verb.execution {
                VerbExecution::Internal { internal, .. } => {
                    sub.set_md("description", internal.description());
                    sub.set("execution", "");
                }
                VerbExecution::External(external) => {
                    sub.set("description", "");
                    sub.set("execution", &external.exec_pattern); // we should maybe also show the invoc pattern
                }
            }
        }
    }
    expander.expand()
}
