### next


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
- completion of the "client-server" feature (see client-server.md)
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
- :preview_binay, :preview_text, and :preview_image verbs allow the choice of previewing mode
- fix a possible crash in previewed files on displaying fuzzy pattern matches

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
- git status takes into accout overloading of enter and alt-enter
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
Several new launch flags have been added, mostly doing the opposite of previous ones (eg `-S` negates `-s`) and a new entry in the conf.toml lets you define default flags (which can be overriden by the ones you pass on the command line).
Do `br --help` to view the complete list of flags.
#### Minor changes:
- on refresh or after command, if the previously selected path can't be selected (missing file, probably) then the previous index will be kept if possible
- alt-enter can be rebinded (users should not do that whithout binding `:cd`, though)

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
- diplay the real size of sparse files (fix #102)

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
- fix scroll adjustement when using the arrow keys (when there's a scrollbar) (fix #112)

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
* fix crash on doing `:rm` on the last child of current root
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
Fix non consistent builds due to lack of precise versionning in crossterm subcrate versionning

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
