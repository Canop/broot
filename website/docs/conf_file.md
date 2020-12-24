
# Opening the configuration file

Two formats are allowed: [TOML](https://github.com/toml-lang/toml) and [Hjson](https://hjson.github.io/).

The configuration file is called either `conf.toml` or `conf.hjson`.

This default file's location follows the XDG convention, which depends on your system settings. This location in your case can be found on the help screen (use <kbd>?</kbd>).

From this screen you can directly open the configuration file in your system's editor by typing `:os` (shortcut for `:open_stay`).

The default configuration file contains several example sections that you may uncomment and modify for your goals.

# Default flags

Broot accepts a few flags at launch (the complete list is available with `broot --help`.

For example, if you want to see hidden files (the ones whose name starts with a dot) and the status of files related to git, you launch broot with

    br -gh

If you almost always want those flags, you may define them as default in the configuration file file, with the `default_flags` setting.

In TOML:

    default_flags = "gh"

In Hjson:

    default_flags: gh

Those flags can still be overridden at launch with the negating ones. For example if you don't want to see hidden files at a specific launch, do

    br -H

# Mouse Capture

Broot usually captures the mouse so that you can click or double click on items. If you want to disable this capture, you may add this:

```toml
capture_mouse = false
```

# Special Paths

You may map special paths to specific behaviors. You may especially want

- to have some link to a directory to always automatically be handled as a normal directory
- to exclude some path because it's on a slow device or non relevant

Example configuration in TOML:

```toml
[special-paths]
"/media/slow-backup-disk" = "no-enter"
"/home/dys/useless" = "hide"
"/home/dys/my-link-I-want-to-explore" = "enter"
```

In Hjson:

```css
special_paths: {
	"/media/slow-backup-disk"		: no-enter
	"/home/dys/useless"			: hide
	"/home/dys/my-link-I-want-to-explore"	: enter
}
```

Be careful that those paths (globs, in fact) are checked a lot when broot builds trees and that defining a lot of paths will impact the overall speed.

# Search Modes

It's possible to redefine the mode mappings, for example if you usually prefer to do exact searches:

```toml
[search-modes]
"<empty>" = "regex name"
"/" = "fuzzy path"
"z/" = "regex path"
```

Note: I'd insist on you not overwriting default mode mappings before you master how broot is used and what those modes exactly work.

# Selection Mark

When the background colors aren't rendered in your terminal, aren't visible enough, or just aren't clear enough for you, you may have the selected lines marked with triangles with

```toml
show_selection_mark = true
```

# Columns order

You may change the order of file attributes in file lists:

*  mark: a small triangle flagging the selected line
*  git : Git file info
*  branch : shows the depth and parent in the tree
*  permission : mode, user, group
*  date : last modification date
*  size : ISO size (and size bar when sorting)
*  count : number of files in directories
*  name : file name

For example, if you prefer to have the branches left of the tree (as was the default in broot prior 0.18.1) you can use

```toml
cols_order = [
	"mark",
	"git",
	"branch",
	"permission",
	"date",
	"size",
	"count",
	"name",
]
```

The name should be kept at end as it's the only one with a variable size.

# Colors by file extension

broot doesn't support `LS_COLORS` which isn't available on all systems and is limited to 16 system dependant colors.

But you can still give a color to files by extension:

```toml
[ext-colors]
png = "rgb(255, 128, 75)"
rs = "yellow"
toml = "ansi(105)"
```

# Syntax Theme

Broot uses [syntect](https://github.com/trishume/syntect) for syntax coloring of previewed files.

It's possible to choose any of the standard themes listed [here](https://docs.rs/syntect/latest/syntect/highlighting/struct.ThemeSet.html#impl):

* base16-ocean.dark
* base16-eighties.dark
* base16-mocha.dark
* base16-ocean.light
* InspiredGitHub
* Solarized (dark)
* Solarized (light)

```toml
syntax_theme = "base16-ocean.light"
```

