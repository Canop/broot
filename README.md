# Broot

[![CI][s3]][l3] [![MIT][s2]][l2] [![Latest Version][s1]][l1] [![Chat on Miaou][s4]][l4]

[s1]: https://img.shields.io/crates/v/broot.svg
[l1]: https://crates.io/crates/broot

[s2]: https://img.shields.io/badge/license-MIT-blue.svg
[l2]: LICENSE

[s3]: https://travis-ci.org/Canop/broot.svg?branch=master
[l3]: https://travis-ci.org/Canop/broot

[s4]: https://miaou.dystroy.org/static/shields/room.svg
[l4]: https://miaou.dystroy.org/3490?broot

A better way to navigate directories


Table of Contents
=================

  * [Installation](#installation)
  * [Feature Showcase](#feature-showcase)
    * [Get an overview of a directory, even a big one](#get-an-overview-of-a-directory-even-a-big-one)
    * [Find a directory then `cd` to it](#find-a-directory-then-cd-to-it)
    * [Never lose track of file hierarchy while you search](#never-lose-track-of-file-hierarchy-while-you-search)
    * [Manipulate your files](#manipulate-your-files)
    * [Do it with panels](#do-it-with-panels)
    * [Apply a standard or personal shortcut to a file](#apply-a-standard-or-personal-shortcut-to-a-file)
    * [Replace `ls` (and its clones)](#replace-ls-and-its-clones)
    * [See what takes space](#see-what-takes-space)
    * [Check git statuses](#check-git-statuses)
  * [Further Reading](#further-reading)

## Installation
Full installation documentation can be found [here](https://dystroy.org/broot/documentation/installation/).

## Feature Showcase

### Get an overview of a directory, even a big one

![overview](img/20191112-overview.png)

Notice the *unlisted*?

That's what makes it usable where the old `tree` command would produce pages of output.

`.gitignore` files are properly dealt with to put unwanted files out of your way (you can ignore them though, see [documentation](https://dystroy.org/broot/documentation/usage/#gitignore)).

### Find a directory then `cd` to it

![cd](img/20191112-cd.png)

This way, you can navigate to a directory with the minimum amount of keystrokes, even if you don't exactly remember where it is.

broot is fast and never blocks, even when you make it search a big slow disk (any keystroke interrupts the current search to start the next one).

Most useful keys for this:

* the letters of what you're looking for
* `<enter>` to select a directory (staying in broot)
* `<esc>` to get back to the previous state or clear your search
* `<alt><enter>` to get back to the shell having `cd` to the selected directory
* `:q` if you just want to quit (`<esc>`, or `<ctrl><q>` or `<ctrl><c>` work too)

### Never lose track of file hierarchy while you search

![size](img/20191112-mycnf.png)

broot tries to select the most relevant file. You can still go from one match to another one using `<tab>` or arrow keys.

You may also search with a regular expression. To do this, add a `/` before or after the pattern.

Once the file you want is selected you can

* hit `<enter>` (or double-click) to open it in your system's default program
* hit `<alt><enter>` to open it in your system's default program and close broot
* type a verb. For example `:e` opens the file in your preferred editor (which may be a terminal one)

### Manipulate your files

![mv](img/20191112-mv.png)

Without broot you move your files in the blind. You do a few `ls` before, then your manipulation, and maybe you check after.

You can instead do it without losing the view of the file hierarchy.

`mv`, `cp`, `rm`, `mkdir`, are built in and you can add your own shortcuts.

### Do it with panels

Do `<ctrl><right>` and you open another panel (you may open other ones, or navigate between them, with `<ctrl><left>` and `<ctrl><right>`).

![custom colors tree](website/docs/img/20200525-colored-panels.png)

(yes, colors are fully customizable)

Using two panels, you can for example copy or move elements between them:

![custom colors tree](website/docs/img/20200525-cpp.png)

`:cpp` is an alias for `:copy_to_panel`. If you like you may do it Norton Commander style by binding `:copy_to_panel` to `<F5>` and `:move_to_panel` to `<F6>`.


### Apply a standard or personal shortcut to a file

![size](img/20191112-edit.png)

Just find the file you want to edit with a few keystrokes, type `:e`, then `<enter>`.

You can add verbs or configure the existing ones; see [documentation](https://dystroy.org/broot/documentation/usage/#verbs).

And you can add shortcuts, for example a `ctrl` sequence or a function key

### Replace `ls` (and its clones):

If you want to display *dates* and *permissions*, do `br -dp` which gets you this:

![replace ls](img/20191214-replace-ls.png)

You may also toggle options with a few keystrokes while inside broot. For example hitting a space, a `d` then enter shows you the dates.

You still have all broot features, you can filter, navigate, create directories, copy files, etc.

### See what takes space:

![size](img/20191112-sizes.png)

If you start broot with the `--sizes` option, or if you type `:s` while in broot, you get a mode tailored to "whale spotting" navigation, making it easy to determine what files or folders take space.

And you keep all broot tools, like filtering or the ability to delete or open files and directories.

Sizes are computed in the background, you don't have to wait for them when you navigate.

### check git statuses:

![size](img/20200203-git.png)

Use `:gf` to display the statuses of files (what are the new ones, the modified ones, etc.), the current branch name and the change statistics.

## Further Reading
See **[Broot's web site](https://dystroy.org/broot)** for instructions regarding installation and usage.
