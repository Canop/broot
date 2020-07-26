
# Opening the configuration file

The configuration file is called `conf.toml` and is in [TOML](https://github.com/toml-lang/toml).

This file's location follows the XDG convention, which depends on your system settings. This location in your case can be found on the help screen (use <kbd>?</kbd>).

From this screen you can directly open the configuration file in your system's editor by typing `:os` (shorcut for `:open_stay`).

Currently, you can configure

* default flags
* special paths
* verbs and shortcuts
* style

The default configuration file contains several example sections that you may uncomment and modify for your goals.

# Default flags

Broot accepts a few flags at launch (the complete list is available with `broot --help`.

For example, if you want to see hidden files (the ones whose name starts with a dot) and the status of files related to git, you launch broot with

    br -gh

If you almost always want those flags, you may define them as default in the `conf.toml` file, with the `default_flags` setting:

    default_flags = "gh"

Those flags can still be overriden at launch with the negating ones. For example if you don't want to see hidden files at a specific launch, do

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

Example configuration:

```toml
[special-paths]
"/media/slow-backup-disk" = "no-enter"
"/home/dys/useless" = "hide"
"/home/dys/my-link-I-want-to-explore" = "enter"
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

# Columns order

You may change the order of file attributes in file lists.

The `cols_order` property, if specified, must be a permutation of `"gbpdscn"` where every char denotes a column:

*  g : Git file info
*  b : branch (shows the depth and parent in the tree)
*  p : permissions (mode, user, group)
*  d : last modification date
*  s : size (with size bar when sorting)
*  c : count, number of files in directories
*  n : file name

The default value is

```toml
cols_order = "gscpdbn"
```
If you prefer to have the branchs left of the tree (as was the default in broot prior 0.18.1) you can use

```toml
cols_order = "gbpdscn"
```

The `n` column should be kept at end as it's the only one with a variable size.

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

It's possible to choose any of the standard themes listed [here](https://docs.rs/syntect/4.2.0/syntect/highlighting/struct.ThemeSet.html#impl):

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

