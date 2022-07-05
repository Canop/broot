use {
    termimad::minimad::{TextTemplate, TextTemplateExpander},
};

static MD: &str = r#"

# broot ${version}

**broot** lets you explore directory trees and launch commands.
It's best used when launched as **br**.
See **https://dystroy.org/broot** for a complete guide.

The *esc* key gets you back to the previous state.
The *↑* and *↓* arrow keys can be used to change selection.
The mouse can be used to select (on click) or open (on double-click).

## Search Modes

Type some letters to search the tree and select the most relevant file.
${default-search
For example, ${default-search-example}.
}
Various types of search can be used:
|:-:|:-:|:-
|**prefix**|**search**|**example**|
|-:|:-|:-
${search-mode-rows
|`${search-prefix}`|${search-type}|${search-example}
}
|-
You can combine searches with logical operators.
For example, to search all toml or rs files containing `tomat`, you may type `(${nr-prefix}toml/|${nr-prefix}rs$/)&${ce-prefix}tomat`.
For efficiency, place content search last.

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

## Configuration

Verbs, skin, and more, are configured in
${config-files
* **${path}**
}
(hit *enter* to open the main configuration file)

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
    use once_cell::sync::Lazy;
    static TEMPLATE: Lazy<TextTemplate<'static>> = Lazy::new(|| TextTemplate::from(MD));
    TEMPLATE.expander()
}
