use {
    lazy_static::lazy_static,
    minimad::{TextTemplate, TextTemplateExpander},
};

static MD: &str = r#"

# broot ${version}

**broot** lets you explore directory trees and launch commands.
It's best used when launched as **br**.
See **https://dystroy.org/broot** for a complete guide.

The *esc* key gets you back to the previous state.
The *↑* and *↓* arrow keys can be used to change selection.
The mouse can be used to select (on click) or open (on double-click).

## Configuration

Verbs, skin, and more, are configured in
${config-files
* **${path}**
}
(hit *enter* to open)

## Verbs

To execute a verb, type a space or `:` then start of its name or shortcut.
This table is searchable. Hit a few letters to filter it.
|:-:|:-:|:-:|:-:
|**name**|**shortcut**|**key**|**description**
|-:|:-:|:-:|:-
${verb-rows
|${name}|${shortcut}|${key}|${description}`${execution}`
}
|-:

## Search Modes

Typing some letters searches the tree and selects the most relevant file.
To use a regular expression, prefix with a slash eg `/j(ava|s)$/i`.
To search by file content, prefix with `c/` eg `c/TODO`.
|:-:|:-:
|**prefix**|**search**
|:-:|:-
${search-mode-rows
|${search-prefix}|${search-type}
}
|-:

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

/// build a markdown expander which will need to be
/// completed with data and which then would be used to
/// produce the markdown of the help page
pub fn expander() -> TextTemplateExpander<'static, 'static> {
    lazy_static! {
        // this doesn't really matter, only half a ms is spared
        static ref TEMPLATE: TextTemplate<'static> = TextTemplate::from(MD);
    }
    TEMPLATE.expander()
}
