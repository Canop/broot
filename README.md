## Broot

[![Tests][s3]][l3] [![MIT][s2]][l2] [![Latest Version][s1]][l1] [![Chat on Miaou][s4]][l4] [![Packaging status][srep]][lrep]

[s1]: https://img.shields.io/crates/v/broot.svg
[l1]: https://crates.io/crates/broot

[s2]: https://img.shields.io/badge/license-MIT-blue.svg
[l2]: LICENSE

[s3]: https://github.com/Canop/broot/actions/workflows/tests.yml/badge.svg
[l3]: https://github.com/Canop/broot/actions/workflows/tests.yml

[s4]: https://miaou.dystroy.org/static/shields/room.svg
[l4]: https://miaou.dystroy.org/3490?broot

[srep]: https://repology.org/badge/tiny-repos/broot.svg
[lrep]: https://repology.org/project/broot/versions

A better way to navigate directories

[**Complete Documentation**](https://dystroy.org/broot/) -
[**Installation Instructions**](https://dystroy.org/broot/install/) -
[**Contributing or Getting Help**](https://dystroy.org/blog/contributing/)

## Get an overview of a directory, even a big one

Hit `br -s`

![overview](website/docs/img/20230930-overview.png)

Notice the *unlisted*?

That's what makes it usable where the old `tree` command would produce pages of output.

`.gitignore` files are properly dealt with to put unwanted files out of your way.

As you sometimes want to see gitignored files, or hidden ones, you'll soon get used to the <kbd>alt</kbd><kbd>i</kbd> and <kbd>alt</kbd><kbd>h</kbd> shortcuts to toggle those visibilities.

(you can ignore them though, see [documentation](../navigation/#toggles)).

## Find a directory then `cd` to it

type a few letters

![cd](website/docs/img/20230930-cd.png)

Hit <kbd>alt</kbd><kbd>enter</kbd> and you're back to the terminal in the desired location.

This way, you can navigate to a directory with the minimum amount of keystrokes, even if you don't exactly remember where it is.

Broot is fast and doesn't block (any keystroke interrupts the current search to start the next one).

Most useful keys for this:

* the letters of what you're looking for
* <kbd>enter</kbd> on the root line to go up to the parent (staying in broot)
* <kbd>enter</kbd> to focus a directory (staying in broot)
* <kbd>esc</kbd> to get back to the previous state or clear your search
* <kbd class=b>↓</kbd> and <kbd class=b>↑</kbd> may be used to move the selection
* <kbd>alt</kbd><kbd>enter</kbd> to get back to the shell having `cd` to the selected directory
* <kbd>alt</kbd><kbd>h</kbd> to toggle showing hidden files (the ones whose name starts with a dot)
* <kbd>alt</kbd><kbd>i</kbd> to toggle showing gitignored files
* `:q` if you just want to quit (you can use <kbd>ctrl</kbd><kbd>q</kbd> if you prefer)

## Never lose track of file hierarchy while you search

![search](website/docs/img/20230930-gccran.png)

Broot tries to select the most relevant file. You can still go from one match to another one using <kbd>tab</kbd> or arrow keys.

You may also search with a regular expression. To do this, add a `/` before the pattern.

And you have [other types of searches](input/#the-filtering-pattern), for example searching on file content (start with `c/`):

![content search](website/docs/img/20230930-content-memm.png)

You may also apply logical operators or combine patterns, for example searching `test` in all files except json ones could be `!/json$/&c/test` and searching `carg` both in file names and file contents would be `carg|c/carg`.

Once the file you want is selected you can

* hit <kbd>enter</kbd> (or double-click) to open it in your system's default program
* hit <kbd>alt</kbd><kbd>enter</kbd> to open it in your system's default program and close broot
* hit <kbd>ctrl</kbd><kbd>→</kbd> to preview it (and then a second time to go inside the preview)
* type a verb. For example `:e` opens the file in your preferred editor (which may be a terminal one)

[blog: a broot content search workflow](https://dystroy.org/blog/broot-c-search/)

## Manipulate your files

Most often, when not using broot, you move your files in the blind. You do a few `ls` before, then your manipulation, and maybe you check after.

You can instead do it without losing the view of the file hierarchy.

![mv](website/docs/img/20230930-mv.png)

Move, copy, rm, mkdir, are built in and you can add your own shortcuts.

Here's chmod:

![chmod](website/docs/img/20230930-chmod.png)

## Manage files with panels

When a directory is selected, do <kbd>ctrl</kbd><kbd>→</kbd> and you open another panel (you may open other ones, or navigate between them, with <kbd>ctrl</kbd><kbd>←</kbd> and <kbd>ctrl</kbd><kbd>→</kbd>).

![custom colors tree](website/docs/img/20230930-colored-panels.png)

(yes, colors are fully customizable)

You can for example copy or move elements between panels:

![cpp](website/docs/img/20230930-cpp.png)

If you like you may do it Norton Commander style by binding `:copy_to_panel` to <kbd>F5</kbd> and `:move_to_panel` to <kbd>F6</kbd>.

## Preview files

Hit <kbd>ctrl</kbd><kbd>→</kbd> when a file is selected and the preview panel appears.

![preview](website/docs/img/20230930-preview.png)

![preview](website/docs/img/20230930-preview-image.png)

The preview panel stays synchronized with the selection in tree panels.

Broot displays images in high resolution when the terminal supports Kitty's graphics protocol
(compatible terminals: [Kitty](https://sw.kovidgoyal.net/kitty/index.html), [WezTerm](https://wezfurlong.org/wezterm/)):

![kitty preview](website/docs/img/20201127-kitty-preview.png)

## Apply a standard or personal command to a file

![size](website/docs/img/20230930-edit.png)

Just find the file you want to edit with a few keystrokes, type `:e`, then <kbd>enter</kbd>.

You can add verbs or configure the existing ones; see [documentation](conf_file/#verbs-shortcuts-and-keys).

And you can add shortcuts, for example a <kbd>ctrl</kbd> sequence or a function key

## Apply commands on several files

Add files to the [staging area](staging-area) then execute any command on all of them.

![staging mv](website/docs/img/20230930-staging-mv.png)

## Replace `ls` (and its clones):

If you want to display *sizes*, *dates* and *permissions*, do `br -sdp` which gets you this:

![replace ls](website/docs/img/20230930-sdp.png)

You may also toggle options with a few keystrokes while inside broot. For example hitting a space, a <kbd>d</kbd> then <kbd>enter</kbd> shows you the dates. Or hit <kbd>alt</kbd><kbd>h</kbd> and you see hidden files.

## Sort, see what takes space:

You may sort by launching broot with `--sort-by-size` or `--sort-by-date`. Or you may, inside broot, type a space, then `sd`, and <kbd>enter</kbd> and you toggled the `:sort_by_date` mode.

When sorting, the whole content of directories is taken into account. So if you want to find on Monday morning the most recently modified files, launch `br --sort-by-date ~`.

If you start broot with the `--whale-spotting` option (or its shortcut `-w`), you get a mode tailored to "whale spotting" navigation, making it easy to determine what files or folders take space.

Sizes, dates, files counts, are computed in the background, you don't have to wait for them when you navigate.

![size](website/docs/img/20230930-whale-spotting.png)

And you keep all broot tools, like filtering or the ability to delete or open files and directories.

If you hit `:fs`, you can check the usage of all filesystems, so that you focus on cleaning the full ones.

![fs](website/docs/img/20230930-fs.png)


## Check git statuses:

Use `:gf` to display the statuses of files (what are the new ones, the modified ones, etc.), the current branch name and the change statistics.

![size](website/docs/img/20230930-git.png)

And if you want to see *only* the files which would be displayed by the `git status` command, do `:gs`. From there it's easy to edit, or diff, selected files.

![size](img/20230930-gg.png)

From there it's easy to edit, diff,  or revert selected files.

[blog: use broot and meld to diff before commit](https://dystroy.org/blog/gg/)


## Further Reading
See **[Broot's web site](https://dystroy.org/broot)** for instructions regarding installation and usage.
