### Next Version
Pressing F5 clears the directory sizes cache

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
