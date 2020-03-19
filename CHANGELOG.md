### master
- replace ~ in path arguments with user home dir
- use $XDG_CONFIG_HOME/git/ignore when the normal core.excludesFile git setting is missing

<a name="v0.13.4"></a>
### v0.13.4 - 2020-03-13
- support for an arg made of an optional group - Fix #210

<a name="v0.13.3"></a>
### v0.13.3 - 2020-02-27
- fix a compilation problem related to dependency (termimad) versio

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
