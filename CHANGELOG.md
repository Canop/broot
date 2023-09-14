
### next
- optional BROOT_CONFIG_DIR env var specifies the config directory

### v1.25.1 - 2023-09-03
<a name="v1.25.1"></a>
- fix shift-char in input extending the selection - Fix #733

### v1.25.0 - 2023-08-19
<a name="v1.25.0"></a>
- allow unescaped '::' in pattern position, experimental (might be removed)
- allow hexa color notation in skins (eg `#fb0` or `#FFD700`)

### v1.24.2 - 2023-07-18
<a name="v1.24.2"></a>
- fix a case of br script installation failing on Windows/Powershell

### v1.24.1 - 2023-07-16
<a name="v1.24.1"></a>
- slightly better `--help`

### v1.24.0 - 2023-07-16
<a name="v1.24.0"></a>
- installer for the powershell br script on windows - Thanks @felixkroemer
- new `--help`, more compact
- allow extra spaces before the verb
- updated man page, now distributed in releases as /man/broot.1

### v1.23.0 - 2023-06-16
<a name="v1.23.0"></a>
- prettier, faster SVG rendering
- reorganize default conf files, with a "skins" subfolder

### v1.22.1 - 2023-05-23
<a name="v1.22.1"></a>
- allow dir computations in /run/media - Fix #704 - Thanks @jinliu
- fix included solarized-dark.hjson skin file

### v1.22.0 - 2023-05-18
<a name="v1.22.0"></a>
- define disk space availability colors in skin - Fix #705
- left elision of path when path/name doesn't fit - Fix #700

### v1.21.3 - 2023-05-02
<a name="v1.21.3"></a>
- `switch_terminal` verb parameter - Thanks @stevenxxiu
- on Windows, when using `-c`, clear events after delay - Fix #699

### v1.21.2 - 2023-03-30
<a name="v1.21.2"></a>
- update dependencies because of some yanked ones

### v1.21.1 - 2023-03-23
<a name="v1.21.1"></a>
- resolve `~` in special paths - Fix #685
- better clipboard support on MacOS - Thanks @bryan824

### v1.21.0 - 2023-03-17
<a name="v1.21.0"></a>
- better nushell integration (no need to quote arguments anymore, fix path extension broken by new version of nushell) - Thanks @stevenxxiu
- don't show modal-only keys in help page when modal mode isn't enabled

### v1.20.2 - 2023-02-19
<a name="v1.20.2"></a>
- fix debug statement printed in some cases (mostly on Windows) - Fix #672

### v1.20.1 - 2023-02-08
<a name="v1.20.1"></a>
- fix status line not always displaying the hint of the input's verb - Fix #665

### v1.20.0 - 2023-02-03
<a name="v1.20.0"></a>
- unless overridden, `/proc` is now `no-enter`, which solves freezes when searching on `/` in some system - See #639
- SVG files now rendered as images in the preview panel
- new version of the nushell function. You should be prompted for an update - Fix #656 - Thanks @FrancescElies and @mediumrarez
- `no-hide` special paths - Thanks @Avlllo
- preview can now be opened on directories, showing their first level - Fix #405
- better determine whether the terminal is white or dark in some (probably rare) cases - See https://github.com/Canop/terminal-light/issues/2

### v1.19.0 - 2023-01-03
<a name="v1.19.0"></a>
- Nushell support - Fix #375 - Thanks @FrancescElies, @mediumrarez, and issue contributors

### v1.18.0 - 2022-12-21
<a name="v1.18.0"></a>
- Hjson configuration file can now omit outside braces (it's "braceless Hjson"), making it much cleaner
- allow opening the help screen with just the `?` key on Windows (as for other systems)
- fix a crash in some cases of input being cleaned with a selection - Fix #643

### v1.17.1 - 2022-12-15
<a name="v1.17.1"></a>
- Windows specific implementation of :cpp

### v1.17.0 - 2022-12-09
<a name="v1.17.0"></a>
- max file size for content search now configurable (default is now 10MB) - Fix #626
- file summing now avoids /proc and /run
- default configuration sets /media as not entered by default (can be commented out, of course)

### v1.16.2 - 2022-11-04
<a name="v1.16.2"></a>
- you can restrict the panels in which verbs apply with the verb configuration `panels` parameter
- fix rm on Windows behaving "recursively" (it was `cmd /c del /Q /S {file}`) - Fix #627

### v1.16.1 - 2022-10-13
<a name="v1.16.1"></a>
- fix ctrl-left not usable anymore in filtered preview to remove filtering

### v1.16.0 - 2022-10-07
<a name="v1.16.0"></a>
- status messages now displayed on toggling (for example showing hidden files)
- upgrade terminal-light to 1.0.1 for better recognition of background color on high precision color terminals
- in default configuration, ctrl-left never opens a panel to the left, as I think this was most often unwanted (one too many hit on cltr-left). It's possible to get the old behavior by binding ctrl-left to `:panel_left` instead of the new `:panel_left_no_open` internal.
- New escaping rules let you skip many `\`, especially when building regexes - See new rules at https://dystroy.org/broot/input/#escaping - Fix #592

### v1.15.0 - 2022-09-24
<a name="v1.15.0"></a>
- with `show_matching_characters_on_path_searches: false`, it's possible to show only file names even when searching paths - Fix #490
- `--sort-by-type-dirs-first` and `--sort-by-type-dirs-last` - Fix #602
- modal: in input mode, uppercase letters don't trigger verbs anymore - Fix #604
- fix `:line_down_no_cycle` which was cycling - Fix #603
- selecting lines up or down with the mouse wheel now wraps in both direction (ie going up when your on top brings you to the bottom, and vice-versa)
- `:select` internal, which can be used to select a visible file when given a path as argument. Experimental

### v1.14.3 - 2022-09-12
<a name="v1.14.3"></a>
- fix crash with token searches - Fix #504 - Thanks @FedericoStra

### v1.14.2 - 2022-07-11
<a name="v1.14.2"></a>
- Terminal background luma determination now works on all tested unixes, including MacOS - Fix #575
- Allow `:focus` based verbs to take a pattern - Fix #389

### v1.14.1 - 2022-07-06
<a name="v1.14.1"></a>
Due to a technical problem, background color based skin selection is disabled on non linux systems.

### v1.14.0 - 2022-07-05
<a name="v1.14.0"></a>
#### Major Feature: imports
A configuration file can now import one or several other ones.
An import can have a condition on the terminal's background color, which makes it possible to import either a dark or a light theme depending on the current terminal settings.
You're also encouraged to split your configuration in several files, as is now done for the default configuration.
### Minor changes
- fix `--cmd` not working (it was accidentally renamed in `--commands`, `-c` was still working) - Fix #570

### v1.13.3 - 2022-06-19
<a name="v1.13.2"></a>
- fix `default_flags` in conf not working anymore - Fix #566

### v1.13.2 - 2022-06-18
<a name="v1.13.2"></a>
- advice to hit alt-i and|or alt-h when no file is visible - Fix #556
- examples on search modes in help screen - Fix #559
- list of syntactic themes in default conf
- the --file-export-path launch argument which was deprecated since broot 1.6 has been removed (redirect the output of broot instead)
- better built-in verbs for Windows - Thanks @Spacelord-XaN
- take the .git/info/exclude file into account for ignoring - Thanks @refi64

Note: The released archive doesn't include an Android build - see https://github.com/Canop/broot/issues/565

### v1.13.1 - 2022-05-30
<a name="v1.13.1"></a>
- fix alt-enter failing to cd to directory

### v1.13.0 - 2022-05-29
<a name="v1.13.0"></a>
- close the staging area when it's emptied with a verb (e.g. on `:rm`)
- format files counts with thousands separator - Fix #549
- try verbs in order allowing some with filters before one without - Fix #552

### v1.12.0 - 2022-05-05
<a name="v1.12.0"></a>
- `:stage_all_files` internal, adding to the staging area all the files verifying the current pattern. Mapped by default to ctrl-a

### v1.11.1 - 2022-04-04
<a name="v1.11.1"></a>
- fix broot not being usable while an image is being opened by hitting enter on linux - Fix #530

### v1.11.0 - 2022-04-02
<a name="v1.11.0"></a>
- sorting by type, with 3 new internals: `:sort_by_type_dirs_first`,  `:sort_by_type_dirs_last`, and `:sort_by_type`. The last one lets you toggle between no sort, sorting by type with directories first, and sorting by type with directories last. - Fix #467

### v1.10.0 - 2022-03-29
<a name="v1.10.0"></a>
- verb filtering on file extension - Fix #508
- don't quit on tiny terminals - Fix #511
- fix the `capture_mouse` config item which was described in documentation but not usable (the non documented `disable_mouse_capture` argument was working and is kept for compatibility)

### v1.9.4 - 2022-03-07
<a name="v1.9.4"></a>
- don't query size of remote filesystems anymore. This fixes some 10 seconds hangs in some cases (e.g. filesystem screen) when a remote filesystem is unreachable

### v1.9.3 - 2022-02-15
<a name="v1.9.3"></a>
- keep same line visible in preview when resizing
- `:previous_dir` and `:next_dir` internals - Fix #502

### v1.9.2 - 2022-01-23
<a name="v1.9.2"></a>
- instead of crashing on syntect panic in code preview, fall back to unstyled text - Fix #485
- fix files in worktree missing from git statuses - Fix #428

### v1.9.1 - 2022-01-07
<a name="v1.9.1"></a>
- fix a few problems of speed, flickering and uncleaned background with high resolution image preview

### v1.9.0 - 2022-01-06
<a name="v1.9.0"></a>
- total search (launched with ctrl-s) shows all matches - This is experimental and might be reversed, opinions welcome
- kitty graphics protocol used for high definition image rendering on recent enough versions of WezTerm - Fix #473
- fix syntaxic preview of Python files broken by comments - Fix #477
- home key bound to :input_go_to_start, end key bound to :input_go_to_end - Fix #475

### v1.8.1 - 2021-12-29
<a name="v1.8.1"></a>
- fix regex pattern automatically built from content pattern when going from a tree search to a file preview isn't escaped - Fix #472

<a name="v1.8.0"></a>
### v1.8.0 - 2021-12-26
- alt-i bound to toggle_git_ignore
- alt-h bound to toggle_hidden
- text previews switches to hexa when there are not printable chars (eg escape sequences)

<a name="v1.7.5"></a>
### v1.7.5 - 2021-12-16
- Make the "clipboard" feature non default again, as it proves to make compilation harder on some platform. I still distribute executables with this feature and you can still try the compilation with `cargo install broot --features "clipboard"`

<a name="v1.7.4"></a>
### v1.7.4 - 2021-12-01
- Fix 1 or 2 characters of the right ASCII column in hex view sometimes lost

<a name="v1.7.3"></a>
### v1.7.3 - 2021-11-19
- Fix rendering artefacts on Windows, like a duplicate input line

<a name="v1.7.2"></a>
### v1.7.2 - 2021-11-18
- include more syntaxes for preview of code files (using the list from the bat project) - Fix #464

<a name="v1.7.1"></a>
### v1.7.1 - 2021-11-07
- fix clipboard filled with dummy value on launch on X11

<a name="v1.7.0"></a>
### v1.7.0 - 2021-10-30
- "clipboard" feature now default (can still be removed at compilation with  `--no-default-features`)
- fix clipboard features not working on some recent linux distributions
- you can now select part of the input with shift arrows or by dragging the mouse cursor
- new internals: input_selection_cut and input_selection_copy (not bound by default)

<a name="v1.6.6"></a>
### v1.6.6 - 2021-10-22
- make it possible to rebind left and right arrow keys without breaking usage in input - Fix #438

<a name="v1.6.5"></a>
### v1.6.5 - 2021-10-01
- improve decision on whether to trim root - Fix #434
- better make the tree's selected line visible

<a name="v1.6.4"></a>
### v1.6.4 - 2021-10-01
- better scrolling behaviors - Fix #419
- fix special-path::Enter for symlinks - Fix #448

<a name="v1.6.3"></a>
### v1.6.3 - 2021-08-02
- hjson: fix bad parsing on tab before colon
- now checks all args of externals are set, doesn't use the raw {arg}

<a name="v1.6.2"></a>
### v1.6.2 - 2021-07-31
- broot reads now both the TERM and TERMINAL env variables to try determine whether the terminal is Kitty
- using `:toggle_device_id`, you can display the device id of files (unix only)
- fix a few problems with filesystems analysis by upgrading lfs-core to 0.4.2 - Fix #420
- a few minor rendering improvements

<a name="v1.6.1"></a>
### v1.6.1 - 2021-06-23
- fix compilation on freeBSD
- fix `:filesystems` view not listing disks whose mount point has a space character
- fix panic on searching `cr/.*` if a file starts with an empty line - Fix #406
- fix preview of linux pseudo-files
- identify "RAM" and "crypted" disks in `:filesystems` view

<a name="v1.6.0"></a>
### v1.6.0 - 2021-06-16
- `{root}` argument (current tree root) can be used in verb patterns - Fix #395
- `working_dir` verb attribute - Fix #396
- client-server mode fixed, no longer feature-gated (but still only available on unix like systems)
- broot tries to keep same selection on option changes
- `:tree_up` and `:tree_down` internals, mapped to ctrl-up and ctrl-down - Fix #399
- better handling of auto color mode: two separate behaviors: for app running and for export when leaving - Fix #397
- remove the deprecated `--no-style` launch argument (use `--color no` instead)
- deprecate the `--out` argument (redirecting the output is the recommended solution)
- fix a few minor bugs

<a name="v1.5.1"></a>
### v1.5.1 - 2021-06-03
- fixed a few problems with the `:del_word_right` internal

<a name="v1.5.0"></a>
### v1.5.0 - 2021-06-02
- new `auto_exec` verb property: a non-auto_exec verb isn't executed directly on a keyboard shortcut but fills the input so that it may be edited before execution on enter key
- add support for backtab key (by default it's bound to :previous_match)
- `:rename` built-in verb, best used with its keyboard shortcut F2
- new standard verb arguments: `{file-stem}`, `{file-extension}`, and `{file-dot-extension}`,
- new `:toggle_second_tree` internal - Fix #388
- total size of staging area computed and displayed if sizes displayed elsewhere
- new `file_sum_threads_count` conf property to define the number of threads used for file summing (size, count, last modified). The goal is to more easily search what's the best value depending on the cpu, OS and disk type/speed
- `:input_clear` internal - Fix #24

<a name="v1.4.0"></a>
### v1.4.0 - 2021-05-11
- the default (non prefixed) search is now "path fuzzy" instead of "name fuzzy". You can still change the default mode and mode bindings in the config. This was done after a survey in chat.
- new "unordered tokens" search type: `t/ab,cd` searches for tokens "ab" and "cd" in any order and case insensitive in the subpath, matches for example `src/dcd/Bab.rs` - Fix #378
- fix search modes configuration removing all default mappings - Fix #383
- conf / quit_on_last_cancel to allow quitting with esc when there's nothing to cancel - Fix #380
- new `parent` skin entry for the part of the sub-path before the file name (visible when you search on subpath)
- when a content search has been done, opening a file with a compatible command (like the standard `:edit`) opens on the first line with a match

<a name="v1.3.1"></a>
### v1.3.1 - 2021-04-30
- fix `:previous_match` not jumping over indirect matches - Fix #377
- fix typing a prefixed pattern then emptying it while keeping the prefix doesn't remove filtering - Fix #379
- fix shifted matching chars highlighting with regex patterns when showing icons - Fix #376

<a name="v1.3.0"></a>
### v1.3.0 - 2021-04-28
#### Minor changes:
- modal mode: revert to command mode on command execution - Fix #372
- modal mode: when in command mode, '/' only enters input mode and is never appended to the input
- better handle failing external programs when not leaving broot
#### Major feature: staging area
You may add files to the staging area then apply a command on all of them. This new feature is described [here](https://dystroy.org/broot/staging-area).
Several verbs have been added. Type "stag" in help to see them and their keyboard shortcuts.

<a name="v1.2.10"></a>
### v1.2.10 - 2021-04-03
- fix shift based key shortcuts - Fix #363
- check there's another panel before executing verbs with other-panel argument - Fix #366

<a name="v1.2.9"></a>
### v1.2.9 - 2021-03-18
- fix panic on `:input_del_word_left` - Fix #361
- remove diacritics and normalize unicode from input on fuzzy search (an unnormalized string with unwanted diacritics most often happen when you paste a string in the input)

<a name="v1.2.8"></a>
### v1.2.8 - 2021-03-11
- it's possible to define several key shortcuts for a verb, using the "keys" property
- improvements of fuzzy matching

<a name="v1.2.7"></a>
### v1.2.7 - 2021-02-28
- don't ask again for installation if no sourcing file has been found

<a name="v1.2.6"></a>
### v1.2.6 - 2021-02-27
- clipboard features (copy and paste verbs) now work on Android/Termux (needs the Termux API to be installed)
- fix a compilation problem on non gnu windows - Thanks @Stargateur
- obey '--color no' even in standard application mode. In that case, automatically enable selection marks or you wouldn't know what line is selected

<a name="v1.2.5"></a>
### v1.2.5 - 2021-02-25
- fix style characters being written in `--no-style` mode - Fix #346
- replace `--no-style` with `--color` taking `yes`, `no` or `auto`, with detection of output being piped in `auto` mode (default). `--no-style` is still usable but it's not documented anymore - Fix #347
- fix wrong version number written in log file - Fix #349
- by default the number of panels is now limited to 2 (can be changed in conf with `max_panels_count`). The goal is to improve the global ergonomics for the most common (universal?) use case - Fix #345

<a name="v1.2.4"></a>
### v1.2.4 - 2021-02-14
- :line_down_no_cycle and :line_up_nocycle. They may be mapped instead of :line_up and :line_down when you don't want to cycle (ie arrive on top when you go down past the end of the tree/list) - Fix #344
- fix selected line number rendering in text preview

<a name="v1.2.3"></a>
### v1.2.3 - 2021-02-06
- special paths in "no-enter" or "hide" aren't counted when summing sizes or dates. It's a compromise: it makes all sums a little slower, especially if you have a lot of special paths or complex ones, but it allows skipping over the very slow disks and thus makes some cases much faster - Fix #331
- br fish shell function uses shell completion of broot
- tree height in `:pt` now applies even when there are more root items (thus truncating the tree) - Fix #341
- fix the F5 and F6 shortcuts (copy and move between panels) in the default configuration

<a name="v1.2.1"></a>
### v1.2.1 - 2021-01-27
- allow dashes instead of underscores in conf property names. This fixes a regression as "special-paths", "ext-colors" and "search-modes" were defined with a dash up to version 1.0.7. Now both spellings are OK - Fix #330
- fix some problems with paths containing spaces (regression since 1.1.11)- Fix #329

<a name="v1.2.0"></a>
### v1.2.0 - 2021-01-14
- experimental "modal mode" (or "vim mode") in broot. See https://dystroy.org/broot/vim_mode/
- fix mouse staying captured during external app execution - Fix #325

<a name="v1.1.11"></a>
### v1.1.11 - 2021-01-07
- fix handling of rules starting with '/' in the global gitignore - Fix #321
- alt-c now mapped to the new :copy_line verb which, when in tree, puts the selected path in the clipboard and, when in text preview, puts the selected text line in the clipboard - Fix #322
- it's possible to define verb execution patterns as arrays instead of simple strings, to avoid having to escape quotes - Fix #319

<a name="v1.1.10"></a>
### v1.1.10 - 2020-12-24
broot now accepts both TOML and Hjson files for configuration. Default is Hjson. I explain the change [here](https://dystroy.org/blog/hjson-in-broot/)

<a name="v1.0.9"></a>
### v1.0.9 - 2020-12-19
- fix handling on quotes in configured verbs - Fix #316

<a name="v1.0.8"></a>
### v1.0.8 - 2020-12-01
- when sizes are displayed (eg on `br -s`), show size of root line and root filesystem info
- modified size cache management makes some size computations faster
- sizes (and dates and counts) are progressively displayed

<a name="v1.0.7"></a>
### v1.0.7 - 2020-11-27
* :previous_same_depth and :next_same_depth internals
* in kitty terminal, image preview is high definition

<a name="v1.0.6"></a>
### v1.0.6 - 2020-11-19
* optional icons, thanks to @asdf8dfafjk (@fiAtcBr on Miaou) - See https://dystroy.org/broot/icons
* dev.log renamed into broot.log
* `:line_up` and `:line_down` accept an optional count as argument - Fix #301

<a name="v1.0.5"></a>
### v1.0.5 - 2020-11-05
* in case of IO error when previewing a file, display the error instead of quitting
* fix regression related to display of texts with characters taking several columns
* preview now supports opening system files with size 0 (eg /proc "files")

<a name="v1.0.4"></a>
### v1.0.4 - 2020-10-22
* don't use absolute paths for built-in verbs
* fix freeze on circular symlink chains
* `:filesystems` (alias `:fs`) display all mounted filesystems in a filtrable view. You can enter to browse at the mount point (unix only for now)
* `:toggle_root_fs` (alias `:rfs`) toogles showing information on the filesystem of the current directory
* filesystem information (mainly size and usage) related to the current filesystem displayed in whale-spotting mode

<a name="v1.0.3"></a>
### v1.0.3 - 2020-10-07
* change the syntax of cols_order in conf
* fix left key moving the cursor to start of input (instead of just one char left)

<a name="v1.0.2"></a>
### v1.0.2 - 2020-10-04
* `cr/` patterns search on file content with regular expressions
* search modes and their prefixes listed in help

<a name="v1.0.1"></a>
### v1.0.1 - 2020-09-30
* don't apply .gitignore files (including the global one) when not in a git repository - Fix #274
* the "clipboard" optional feature adds:
	* the `:copy_path` verb which copies the selected path to the clipboard (mapped to alt-c)
	* the `:input_paste` verb which inserts the clipboard content in the input (mapped to ctrl-v)
* it's now possible to define verbs executing sequences of commands - Fix #277
* fix opening of link of link - Fix #280
* broot is now compatible with Android, you can use it on Termux for example
* help page lists all optional features enabled at compilation
* list of verbs in help page is searchable

<a name="v1.0.0"></a>
### v1.0.0 - 2020-09-01
- nothing new, which is better when you want to call your software stable

<a name="v0.20.3"></a>
### v0.20.3 - 2020-08-23
- fix a few problems with tabulation rendering
- fix a few cases of files being called "huge" while they're only very big

<a name="v0.20.2"></a>
### v0.20.2 - 2020-08-18
- fix esc key not removing the filter in text preview

<a name="v0.20.1"></a>
### v0.20.1 - 2020-08-18
- completion of the "client-server" feature
- the tree tries to keep the selection when you remove a filter using the esc key
- :focus now has a shortcut for when a file is selected too: ctrl-f
- show_selection_mark preference in config (mostly for cases the background isn't clear enough)
- **breaking change:** The working directory of external processes launched by broot isn't set anymore by default.
If you want it to be changed, add `set_working_dir = true` to the verb definition.

<a name="v0.20.0"></a>
### v0.20.0 - 2020-08-16
- it's now possible to launch a terminal as sub process from broot (and be back to broot on exit)
- the selected directory is now the working dir for subprocess launched from broot
- images are previewed as such
- :preview_binary, :preview_text, and :preview_image verbs allow the choice of previewing mode
- fix a possible panic in previewed files on displaying fuzzy pattern matches

<a name="v0.19.4"></a>
### v0.19.4 - 2020-07-31
- don't install the br shell function when --outcmd is set or $BR_INSTALL is "no" - Fix #265
- more relevant status hints - Fix #261

<a name="v0.19.3"></a>
### v0.19.3 - 2020-07-27
- refined search in preview interaction (see blog https://dystroy.org/blog/broot-c-search/)

<a name="v0.19.2"></a>
### v0.19.2 - 2020-07-26
- "client-server" feature (see client-server.md)
- preview's pattern is kept when changing file
- selected line in preview, interesting when removing the pattern (to see what's around a match)
- faster availability of huge files in preview
- search in preview now interrupted by key events (just like the trees)
- a content search in a tree is propagated as a regex in a preview on :panel_right (ctrl-right)
- syntax theme choice in conf.toml
- {line} in a verb execution pattern refers to the line number

<a name="v0.19.1"></a>
### v0.19.1 - 2020-07-17
Force trimming root when searching (trimming root when not searching is no longer the default)

<a name="v0.19.0"></a>
### v0.19.0 - 2020-07-16
#### Major feature: the preview panel
Hit ctrl-right when a file is selected and you get the preview.

<a name="v0.18.6"></a>
### v0.18.6 - 2020-07-10
- `[ext-colors]` section in config
- a few minor fixes and changes

<a name="v0.18.5"></a>
### v0.18.5 - 2020-07-05
- git status takes into account overloading of enter and alt-enter
- a few minor fixes and changes

<a name="v0.18.4"></a>
### v0.18.4 - 2020-07-02
- `--git-status` launch option
- fix rendering on windows

<a name="v0.18.3"></a>
### v0.18.3 - 2020-06-30
Faster rendering (0.18.2 made it slower on some terminals)

<a name="v0.18.2"></a>
### v0.18.2 - 2020-06-29
Remove flickering

<a name="v0.18.1"></a>
### v0.18.1 - 2020-06-28
Column order is now configurable - Fix #127

<a name="v0.18.0"></a>
### v0.18.0 - 2020-06-26

#### Major change: Recursive last modified date computation
The date of directories is now the modification date of the last modified inner file, whatever its depth. This is computed in the background and doesn't slow your navigation.

#### Major change: Sort mode
Size can now be displayed out of sort mode, which concerns either size or dates.

There are new launch arguments:
* `--sort-by-count` : sort by number of files in directories
* `--sort-by-date` : sort by dates, taking content into account (make it easy to find deep recent files)
* `--sort-by-size` : sort by size
* `--whale-spotting` or `-w` : "whale spotting" mode (sort by size and show all files)

The `-s` launch argument now works similarly to -d or -p : it doesn't activate a sort mode but activates showing the sizes. `-s` has been replaced with `-w`.

Similarly new verbs have been defined:
* `:toggle_counts`, with shortcut `counts` shows the number of files in directories
* `:toggle_sizes`, with shortcut `sizes` shows the sizes of files and directories
* `:sort_by_count` has for shortcut `sc`
* `:sort_by_date` has for shortcut `sd`
* `:sort_by_size` has `ss` as shortcut
* `:no_sort` removes the current sort mode, if any

<a name="v0.17.0"></a>
### v0.17.0 - 2020-06-21
#### Major feature: keep broot open behind terminal editors
If you now open vi or emacs from broot with `leave_broot = false` you should
be back in broot after you quit the editor - Fix #34 - Fix #144 - Fix #158
#### Minor changes:
- it's possible to define input edition shortcuts - Fix #235
- MacOS: config directory for new install is ~/.config/broot - Fix #103

<a name="v0.16.0"></a>
### v0.16.0 - 2020-06-20
#### Major feature: composite patterns
It's now possible to use logical operators on patterns.

For example:
* `!/txt$/` : files whose name doesn't end in "txt"
* `carg|c/carg` : files whose name or content has "carg"
* `(json|xml)&c/test` : files containing "test" and whose name fuzzily contains either "json" or "xml"
The document contains other examples and precisions.

<a name="v0.15.1"></a>
### v0.15.1 - 2020-06-12
- fix some problems related to relative paths in built in cp and mv

<a name="v0.15.0"></a>
### v0.15.0 - 2020-06-12
#### Major feature: new input syntax - Breaking Change
New search modes (see https://dystroy.org/broot/input/#the-filtering-pattern) :
	- fuzzy or regex on sub-paths (the path starting from the displayed root)
	- search in file content
- it's possible to configure how search modes are selected in config
- search pattern characters can be escaped with a '\'
#### Minor changes:
- tab goes to next direct match when there's no verb in input - Fix #234
- `:open_stay_filter` to be used if you want to keep the pattern when you navigate - Fix #240
- mouse capture can be disabled with `capture_mouse = false` - Fix #238
- several small fixes

<a name="v0.14.2"></a>
### v0.14.2 - 2020-06-01
- `apply_to` verb property - fix #237

<a name="v0.14.1"></a>
### v0.14.1 - 2020-05-29
- fix uppercase letters ignored in input field

<a name="v0.14.0"></a>
### v0.14.0 - 2020-05-29
#### Major feature: `:focus` verb
This verb can be called, and parameterized, with a path as argument, which makes it possible to have a shortcut to a specific location.
As a result, the specific `:focus_user_home` and `:focus_root` verbs have been removed (`:focus ~` works on all OS).
#### Major feature: panels!
There are three major ways to open a new panel:
- by using ctrl-left or ctrl-right, which can also be used to navigate between panels
- when a verb is edited, by using ctrl-p, which opens a panel which on closure will fill the argument
- by using any verb with a bang. For example `:focus! ~` or `:!help`
When you have two panels, you may use some new verbs like :copy_to_panel which copies the selection to the selected location in the other panel.
Many new verbs and functions are related to panels but broot can still be used exactly as before without using panels.
#### Major feature: autocompletion
Using the Tab key you can complete verbs or paths
#### Major feature: special paths
Some paths can be handled in a specific way. Fix #205 and #208
You can for example decide that some slow disk shouldn't be entered automatically
#### Minor changes:
- date/time format configurable - Fix #229
- esc doesn't quit broot anymore (by popular demand)
It's probably a good idea to remove your existing conf.toml file so that broot creates a brand new one with suggestions of shortcuts.

<a name="v0.13.6"></a>
### v0.13.6 - 2020-04-08
- ignore diacritics in searches - Fix #216

<a name="v0.13.5"></a>
### v0.13.5 - 2020-03-28
- right key open directory, left key gets back (when input is empty) - Fix #179
- replace ~ in path arguments with user home dir - Fix #211
- use $XDG_CONFIG_HOME/git/ignore when the normal core.excludesFile git setting is missing - Fix #212
- add a man page to archive - Fix #165

<a name="v0.13.4"></a>
### v0.13.4 - 2020-03-13
- support for an arg made of an optional group - Fix #210

<a name="v0.13.3"></a>
### v0.13.3 - 2020-02-27
- fix a compilation problem related to dependency (termimad) version

<a name="v0.13.2"></a>
### v0.13.2 - 2020-02-16
- fix -i and -I launch arguments being ignored (fix #202)

<a name="v0.13.1"></a>
### v0.13.1 - 2020-02-08
- fix background not always removed when skin requires no background (Fix #194)

<a name="v0.13.0"></a>
### v0.13.0 - 2020-02-05
#### Major change: git related features
- `:show_git_file_info` compute git repo statistics and file statuses. Statistics are computed in background and cached.
- `:git_diff` verb launching `git diff {file}`
- `:git_status` filter files to show only the ones which are relevant for `git status` (warning: slow on big repositories)
#### Major change: rewamped launch flags
Several new launch flags have been added, mostly doing the opposite of previous ones (eg `-S` negates `-s`) and a new entry in the conf.toml lets you define default flags (which can be overridden by the ones you pass on the command line).
Do `br --help` to view the complete list of flags.
#### Minor changes:
- on refresh or after command, if the previously selected path can't be selected (missing file, probably) then the previous index will be kept if possible
- alt-enter can be rebinded (users should not do that without binding `:cd`, though)

<a name="v0.12.2"></a>
### v0.12.2 - 2020-01-29
- fix Ctrl-J being interpreted as Enter (fix #177)

<a name="v0.12.1"></a>
### v0.12.1 - 2020-01-25
- fix panic on some inputs starting with a `/` (Fix #175)
- TAB key now jumps to direct matches only
- `--conf` arg to launch broot with specific config file(s) (fix #141)

<a name="v0.12.0"></a>
### v0.12.0 - 2020-01-19
- **breaking change:** commands given with `--cmd` must be separated (default separator is `;`)
- fix some cases of terminal let in a bad state on errors (thanks Nathan West)
- bring some changes to the fish shell function and its installation (PR #128)
- consider path `$ZDOTDIR/.zshrc` for zsh shell function sourcing (fix #90)
- don't use .gitignore files of parent repositories
- change default value of the toggle_trim_root to false (fix #106 but might be reverted)
- `:print_relative_path` verb (fix #169, thanks Roshan George)
- `:chmod` verb

<a name="v0.11.9"></a>
### v0.11.9 - 2020-01-15
- fix a case of bad selection after search followed by interrupted search (#147)
- `--set-install-state` can be used in tests or manual installs to set the installation state
- Raspberry now a default target available in installation page
- fix a regression: `br -s` not finishing computing size until receiving an event
- display the real size of sparse files (fix #102)

<a name="v0.11.8"></a>
### v0.11.8 - 2020-01-12
- set different skins for the r, w and x parts of the mode (permission)
- compatibility with freeBSD
- generate shell completion scripts on build (deep into the target directory)
- `--print-shell-function` launch argument to print the shell functions to stdout

<a name="v0.11.7"></a>
### v0.11.7 - 2020-01-11
- fix cancelled verbs possibly executed (fix #104) (major dangerous bug)

<a name="v0.11.6"></a>
### v0.11.6 - 2020-01-10
- backspace was previously bound to :back if not consumed by input. This is removed
- fix unsignificative event interpreted as previous event repetition
- fix wrong background applied on sizes in tree display
- allow env vars used in verb execution to contain parameters (fix #114)
- allow the use of arrow keys as triggers for verbs (fix #121)
- fix scroll adjustment when using the arrow keys (when there's a scrollbar) (fix #112)

<a name="v0.11.5"></a>
### v0.11.5 - 2020-01-10
- keep same path selected when lines are reordered (such as when directory sizes are computed
- changed the skin used before installation so that it works better on white backgrounds

<a name="v0.11.4"></a>
### v0.11.4 - 2020-01-09
- make :open_stay and :open_leave work in help screen (applying on configuration file)
- Mac/fish: use ~/.config/fish even on systems where the config home is usually different
- Mac/bash: add .bash_profile to the list of possible sourcing files
- define ctrl-c as a new way to quit

<a name="v0.11.3"></a>
### v0.11.3 - 2020-01-09
- fix the 'n' answer being ignored when user is asked authorization

<a name="v0.11.2"></a>
### v0.11.2 - 2019-12-30
- fix alt-enter not recognized on some computers

<a name="v0.11.0"></a>
### v0.11.0 - 2019-12-21
New major feature: the `:total_search` verb, normally triggered with *ctrl-s*: done after a search it repeats it but looks at **all** the children, even if it's long and there were a lot of matches

<a name="v0.10.5"></a>
### v0.10.5 - 2019-12-20
- should not panic anymore when opening arbitrary files on server
- allow more keys for verbs. For example you can use `enter` (this one won't apply on directories but only on files)
- display all possible verb completions in status
- don't query the terminal size after start: use the new Resize event of Crossterm

<a name="v0.10.4"></a>
### v0.10.4 - 2019-12-16
* fuzzy search performance improvement
* verb invocation now optional so that a verb can be defined to just introduce a keyboard shortcut
* owner and group separately skinned
* screen redrawn on resize (but tree not recomputed, you may want to do F5 to have the best sized tree)
* changes in br shell function storage and sourcing from fish, bash, and zsh. Fixes #39 and #53.
Note that broot will ask you again to install the br function

<a name="v0.10.3"></a>
### v0.10.3 - 2019-11-27
* fix panic on doing `:rm` on the last child of current root
* refactor help page generation using Termimad templates
* clear help background when terminal was resized between redraws

<a name="v0.10.2"></a>
### v0.10.2 - 2019-11-15
* colored status line
* better handling of errors when opening files externally
* spinner replaced with an explicit text
* `:parent` no longer keeps the filter (this was too confusing)
* new `:up` command, focusing the parent of the current tree root
* `$PAGER` used in default config. Fix #20
* default conf links to the white background skin published on web site
* new "default" entry in skin, to define a global background replacing the terminal's one

<a name="v0.10.1"></a>
### v0.10.1 - 2019-11-04
* incorporate crossterm 0.13.2 to fix a regression in vi launch (see https://github.com/Canop/broot/issues/73)

<a name="v0.10.0"></a>
### v0.10.0 - 2019-11-03
* moved to the crossterm 0.13 and termimad 0.7.1
* broot runs on stderr,
* broot can run in a subshell

Those changes allow tricks like `my_unix_command "$(broot)"` when you do `:pp` to print the path on stdout from broot

<a name="v0.9.6"></a>
### v0.9.6 - 2019-09-20
* smarter cut of the status line when it doesn't fit the console's width
* fix mouse click on the status line crashing broot
* prevent the best match from being hidden inside "unlisted" matches

<a name="v0.9.5"></a>
### v0.9.5 - 2019-09-15
* keyboard keys & shortcuts can be defined for more actions, all built-in verbs documented in website
* paths built from verb arguments are now normalized

<a name="v0.9.4"></a>
### v0.9.4 - 2019-09-13
New internal verbs like :focus_root, :focus_user_home, :refresh, :select_first
You can define triggering keys for verbs.

For example you can add those mappings:

	[[verbs]]
	invocation = "root"
	key = "F9"
	execution = ":focus_root"

	[[verbs]]
	invocation = "home"
	key = "ctrl-H"
	execution = ":focus_user_home"

	[[verbs]]
	invocation = "top"
	key = "F6"
	execution = ":select_first"

	[[verbs]]
	invocation = "bottom"
	key = "F7"
	execution = ":select_last"

Then, when doing <key>Ctrl-H</key>, you would go to you user home (`~` when on linux) and <key>F7</key> would select the last line of the tree.

A few more keys are defined as default, like F1 for `:help` and F5 for `:refresh`.

<a name="v0.9.3"></a>
### v0.9.3 - 2019-08-02
Launching broot with `--sizes` now sets a set of features enabling fast "whale spotting" navigation

<a name="v0.9.2"></a>
### v0.9.2 - 2019-07-31
Fix non consistent builds due to lack of precise versioning in crossterm subcrate versioning

<a name="v0.9.1"></a>
### v0.9.1 - 2019-07-29
#### Major change
* A new syntax allows specifying verbs which can work on relative paths or absolute paths alike.
For example the old definition of `cp` was

	invocation = "cp {newpath}"
	execution = "/bin/cp -r {file} {parent}{newpath}"

and it's now

	invocation = "cp {newpath}"
	execution = "/bin/cp -r {file} {newpath:path-from-parent}"

The :path-from-parent formatting means the token will be interpreted as a path, and if it's
not starting with a / will be prefixed by the parent path.
It's possible to also use `{subpath:path-from-directory}` where directory is parent only if
the selected file isn't a directory itself.
#### Minor changes
- shift-tab selects the previous match
- mouse wheel support (selection in tree, scroll in help)
- the input field handles left/right arrow keys, home/end, click, and delete

<a name="v0.9.0"></a>
### v0.9.0 - 2019-07-19
#### Major change
The logic behind opening has changed to allow easier opening of files in non terminal applications without closing broot.

**Old behavior:**
- in case of enter or double-click
   - on a directory: open that directory, staying in broot
   - on a file: open the file, quitting broot
- in case of alt-enter
   - on a directory: cd to that directory, quitting broot
   - on a file: cd to that file's parent directory, quitting broot

**New behavior:**
- in case of enter or double-click
   - on a directory: open that directory, staying in broot
   - on a file: open that file in default editor, not closing broot
- in case of alt-enter
   - on a directory: cd to that directory, quitting broot
   - on a file: open that file in default editor, quitting broot
#### Minor change
- Hitting `?` more directly opens the help screen, even when executing a verb

<a name="v0.8.6"></a>
### v0.8.6 - 2019-07-03
- Hitting enter when first line is selected, or clicking it, goes up to the parent directory
- detect and color executable files on windows
- new toggle to display dates of files (last modification)
- a few small improvements

<a name="v0.8.5"></a>
### v0.8.5 - 2019-06-20
- minor cosmetic changes (this version was mostly released to ensure consistency with termimad's crate)

<a name="v0.8.4"></a>
### v0.8.4 - 2019-06-17
- apply verbs on link files, not on their targets (rm some_link was dangerous)

<a name="v0.8.3"></a>
### v0.8.3 - 2019-06-16
- mouse support: click to select, double-click to open

<a name="v0.8.2"></a>
### v0.8.2 - 2019-06-15
- fix wrong result of scrolling when help text fits the screen

<a name="v0.8.1"></a>
### v0.8.1 - 2019-06-10
- change default skin to only use highly compatible colors
- allow ANSI colors in skin configuration

<a name="v0.8.0"></a>
### v0.8.0 - 2019-06-07
Half broot has been rewritten to allow Windows compatibility. Termion has been replaced with crossterm.

<a name="v0.7.5"></a>
### v0.7.5 - 2019-04-03
- try to give arguments to verbs executed with --cmd
- Hitting <enter> no longer quits when root is selected (many users found it confusing)

<a name="v0.7.4"></a>
### v0.7.4 - 2019-03-25
- fix verbs crashing broot in /
- fix user displayed in place of user with :perm

<a name="v0.7.3"></a>
### v0.7.3 - 2019-03-22
- :print_tree outputs the tree. See [documentation](https://dystroy.org/broot/documentation/usage/#export-a-tree) for examples of use
- F5 refreshes the tree

<a name="v0.7.2"></a>
### v0.7.2 - 2019-03-15
- env variables usable in verb execution patterns, which makes it possible to use `$EDITOR` in default conf.toml
- ctrl-u and ctrl-d are now alternatives to page-up and page-down
- better error messages regarding faulty configurations
- more precise errors in case of invalid regexes
- use the OS specific file opener instead of xdg-open (concretly it means `open` is now used on MacOS)
Thanks Ophir LOJKINE for his contributions in this release

<a name="v0.7.1"></a>
### v0.7.1 - 2019-03-08
- fix a few problems with the count of "unlisted" files

<a name="v0.7.0"></a>
### v0.7.0 - 2019-03-07
##### Major changes
- verbs can now accept complex arguments. This allows functions like mkdir, mv, cp, etc. and your own rich commands
- custom verbs can be executed without leaving broot (if defined with `leave_broot=false`)
##### Minor changes
- Ctrl-Q shortcut to leave broot
- fix a case of incorrect count of "unlisted" files

<a name="v0.6.3"></a>
### v0.6.3 - 2019-02-23
- `br` installer for the fish shell
- faster directory size computation (using a pool of threads)
- fix alt-enter failing to cd when the path had spaces
- executable files rendered with a different color

<a name="v0.6.2"></a>
### v0.6.2 - 2019-02-18
- all colors can be configured in conf.toml

<a name="v0.6.1"></a>
### v0.6.1 - 2019-02-14
- complete verbs handling in help screen
- faster regex search
- fix missing version in `broot -V`

<a name="v0.6.0"></a>
### v0.6.0 - 2019-02-12
##### Major changes
- broot now installs the **br** shell function itself *(for bash and zsh, help welcome for other shells)*
- new verb `:toggle_trim_root` allows to keep all root children
- verbs can refer to `{directory}` which is the parent dir when a simple file is selected
- user configured verbs can be launched from parent shell too (like is done for `cd {directory}`)
##### Minor changes
- allow page up and page down on help screen
- fuzzy pattern: increase score of match starting after word separator
- better handle errors on a few cases of non suitable root (like passing an invalid path)
- clearer status error on `:cd`. Mentions `<alt><enter>` in help
- add a scrollbar on help screen

<a name="v0.5.2"></a>
### v0.5.2 - 2019-02-04
- More responsive on slow disks
- fix a link to documentation in autogenerated conf

<a name="v0.5.1"></a>
### v0.5.1 - 2019-02-03
- alt-enter now executes `:cd`

<a name="v0.5.0"></a>
### v0.5.0 - 2019-01-30
- patterns can be regexes (add a slash before or after the pattern)
- configuration parsing more robust
- no need to put all verbs in config: builtins are accessible even without being in config
- no need to type the entire verb shortcut: if only one is possible it's proposed
- verbs with {file} usable in help state: they apply to the configuration file
- clear in app error message when calling :cd and not using the br shell function
- bring back jemalloc (it's faster for broot)
- more precise display of file/dir sizes

<a name="0.4.7"></a>
### 0.4.7 - 2019-01-21
- fix some cases of panic on broot quitting
- new `--cmd` program argument allows passing a sequence of commands to be immediately executed (see [updated documentation](https://github.com/Canop/broot/blob/master/documentation.md#passing-commands-as-program-argument))
- better handling of symlink (display type of target, show invalid links, allow verbs on target)
- compiled with rustc 1.32 which brings about 4% improvements in perfs compared to 1.31

<a name="v0.4.6"></a>
### v0.4.6 - 2019-01-12
- fix configured verbs not correctly handling paths with spaces
- fix `:q` not instantly quitting broot when computing size
- hit enter on tree root correctly quits broot

<a name="v0.4.5"></a>
### v0.4.5 - 2019-01-11
- Faster search, mainly

<a name="v0.4.3"></a>
### v0.4.3 - 2019-01-08
- Faster search and directory size computation.

<a name="v0.4.2"></a>
### v0.4.2 - 2019-01-07
- more complete search if time allows
- search pattern kept after verb execution

<a name="v0.4.1"></a>
### v0.4.1 - 2019-01-07
- first public release
