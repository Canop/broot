use {
    crate::app::AppContext,
    minimad::{Text, TextTemplate},
};

static MD: &str = r#"

# broot ${version}

**broot** lets you explore directory trees and launch commands.
It's best used when launched as **br**.
See **https://dystroy.org/broot** for a complete guide.

The *esc* key gets you back to the previous state.
Typing some letters searches the tree and selects the most relevant file.
To use a regular expression, prefix with a slash eg `/j(ava|s)$`.
To search by file content, prefix with `c/` eg `c/TODO`.
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
* `-i` : show files which are normally hidden due to .gitignore rules
* `-d` or `--dates` : display last modified dates
* `-w` : whale-spotting mode
 (for the complete list, run `broot --help`)

## Flags

Flags are displayed at bottom right:
* `h:y` or `h:n` : whether hidden files are shown
* `gi:y`, `gi:n` : whether gitignore rules are active or not

## Special Features

${features-text}
${features
* **${feature-name}:** ${feature-description}
}
"#;

fn determine_features() -> Vec<(&'static str, &'static str)> {
    let mut features: Vec<(&'static str, &'static str)> = Vec::new();

    #[cfg(not(any(target_family="windows",target_os="android")))]
    features.push(("permissions", "allow showing file mode, owner and group"));

    #[cfg(feature="client-server")]
    features.push((
        "client-server",
        "see https://github.com/Canop/broot/blob/master/client-server.md"
    ));

    #[cfg(feature="clipboard")]
    features.push((
        "clipboard",
        ":copy_path (copying the current path), and :input_paste (pasting into the input)"
    ));

    features
}

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
            .set(
                "name",
                if let Some(name) = verb.names.get(0) {
                    &name
                } else {
                    ""
                },
            )
            .set(
                "shortcut",
                if let Some(shortcut) = verb.names.get(1) {
                    &shortcut
                } else {
                    ""
                },
            )
            .set("key", &verb.keys_desc);
        if verb.description.code {
            sub.set("description", "");
            sub.set("execution", &verb.description.content);
        } else {
            sub.set_md("description", &verb.description.content);
            sub.set("execution", "");
        }
    }
    let features = determine_features();
    expander.set(
        "features-text",
        if features.is_empty() {
            "This release was compiled with no optional feature enabled."
        } else {
            "This release was compiled with those optional features enabled:"
        },
    );
    for feature in &features {
        expander.sub("features")
            .set("feature-name", feature.0)
            .set("feature-description", feature.1);
    }
    expander.expand()
}
