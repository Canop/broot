use {
    crate::{
        app::AppContext,
        pattern::*,
        verb::*,
    },
    minimad::{TextTemplate, TextTemplateExpander},
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

/// find the list of optional features which are enabled
pub fn determine_features() -> Vec<(&'static str, &'static str)> {
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

pub fn expander() -> TextTemplateExpander<'static, 'static> {
    lazy_static! {
        // this doesn't really matter, only half a ms is spared
        static ref TEMPLATE: TextTemplate<'static> = TextTemplate::from(MD);
    }
    TEMPLATE.expander()
}

/// what should be shown for a verb in the help screen, after
/// filtering
pub struct MatchingVerbRow<'v> {
    name: Option<String>,
    shortcut: Option<String>,
    pub verb: &'v Verb,
}

pub fn matching_verb_rows<'v>(
    pat: &Pattern,
    con: &'v AppContext,
) -> Vec<MatchingVerbRow<'v>> {
    let mut rows = Vec::new();
    for verb in &con.verb_store.verbs {
        let mut name = None;
        let mut shortcut = None;
        if pat.is_some() {
            let mut ok = false;
            name = verb.names.get(0)
                .and_then(|s|
                    pat.search_string(s).map(|nm| {
                        ok = true;
                        nm.wrap(s, "**", "**")
                    })
                );
            shortcut = verb.names.get(1)
                .and_then(|s|
                    pat.search_string(s).map(|nm| {
                        ok = true;
                        nm.wrap(s, "**", "**")
                    })
                );
            if !ok {
                continue;
            }
        }
        rows.push(MatchingVerbRow {
            name,
            shortcut,
            verb,
        });
    }
    rows
}

impl MatchingVerbRow<'_> {
    /// the name in markdown (with matching chars in bold if
    /// some filtering occured)
    pub fn name(&self) -> &str {
        // there should be a better way to write this
        self.name.as_deref().unwrap_or_else(|| match self.verb.names.get(0) {
            Some(s) => &s.as_str(),
            _ => " ",
        })
    }
    pub fn shortcut(&self) -> &str {
        // there should be a better way to write this
        self.shortcut.as_deref().unwrap_or_else(|| match self.verb.names.get(1) {
            Some(s) => &s.as_str(),
            _ => " ",
        })
    }
}
