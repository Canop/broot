

<p align=center style="max-width:600px">
<img src="img/vache.svg" height=140px>
</p>

**[Installation](documentation/installation.md)**

# Get an overview of a directory, even a big one

![overview](img/20191112-overview.png)

Notice the *unlisted*?

That's what makes it usable where the old `tree` command would produce pages of output.

`.gitignore` files are properly dealt with to put unwanted files out of your way (you can ignore them though, see [documentation](documentation/usage/#gitignore)).

# Find a directory then `cd` to it

![cd](img/20191112-cd.png)

This way, you can navigate to a directory with the minimum amount of keystrokes, even if you don't exactly remember where it is.

broot is fast and never blocks, even when you make it search a big slow disk (any keystroke interrupts the current search to start the next one).

Most useful keys for this:

* the letters of what you're looking for
* `<enter>` to select a directory (staying in broot)
* `<esc>` to get back to the previous state or clear your search
* `<alt><enter>` to get back to the shell having `cd` to the selected directory
* `:q` if you just want to quit (you can use `<esc>` or `<ctrl><q>` if you prefer)

# Never lose track of file hierarchy while you search

![size](img/20191112-mycnf.png)

broot tries to select the most relevant file. You can still go from one match to another one using `<tab>` or arrow keys.

You may also search with a regular expression. To do this, add a `/` before or after the pattern.

Once the file you want is selected you can

* hit `<enter>` (or double-click) to open it in your system's default program
* hit `<alt><enter>` to open it in your system's default program and close broot
* type a verb. For example `:e` opens the file in your preferred editor (which may be a terminal one)

# Manipulate your files

![mv](img/20191112-mv.png)

Most often you move your files in the blind. You do a few `ls` before, then your manipulation, and maybe you check after.

You can instead do it without losing the view of the file hierarchy.

Move, copy, rm, mkdir, are built in and you can add your own shortcuts.

# Apply a standard or personal shortcut to a file

![size](img/20191112-edit.png)

Just find the file you want to edit with a few keystrokes, type `:e`, then `<enter>`.

You can add verbs or configure the existing ones; see [documentation](documentation/usage.md#verbs).

And you can add shorcuts, for example a `ctrl` sequence or a function key

# Replace `ls` (and its clones):

If you want to display *dates* and *permissions*, do `br -dp` which gets you this:

![replace ls](img/20191214-replace-ls.png)

You may also toggle options with a few keystrokes while inside broot. For example hitting a space, a `d` then enter shows you the dates. Or a space, then `h` then enter and you show hidden files.

# See what takes space:

![size](img/20191112-sizes.png)

If you start broot with the `--sizes` option, or if you type `:s` while in broot, you get a mode tailored to "whale spotting" navigation, making it easy to determine what files or folders take space.

And you keep all broot tools, like filtering or the ability to delete or open files and directories.

Sizes are computed in the background, you don't have to wait for them when you navigate.

### check git statuses:

![size](img/20200203-git.png)

Use `:gf` to display the statuses of files (what are the new ones, the modified ones, etc.), the current branch name and the change statistics.

And if you want to see *only* the files which would be displayed by the `git status command`, do `:gs`.


# More...

See the complete [Documentation](documentation/usage.md) and [how to install](documentation/installation.md).

