

<p align=center style="max-width:600px">
<img src="img/vache.svg" height=140px>
</p>

# Get an overview of a directory, even a big one

![overview](img/20190128-overview.png)

Notice the "unlisted"? That's what makes it usable where the old `tree` command would produce pages of output.

`.gitignore` files are properly dealt with to put unwanted files out of your way (you can ignore them though, see documentation).

# Find a directory then `cd` to it

![cd](img/20190128-cd.png)

This way, you can navigate to a directory with the minimum amount of keystrokes, even if you don't exactly remember where it is.

broot is fast and never blocks, even when you make it search a big slow disk (any keystroke interrupts the current search to start the next one).

Most useful keys for this:

* the letters of what you're looking for
* `<enter>` to select a directory (staying in broot)
* `<esc>` to get back to the previous state or clear your search
* `<alt><enter>` to get back to the shell having `cd` to the selected directory
* `:q` if you just want to quit (`<esc>` works too)

# Never lose track of file hierarchy while you search

![size](img/20190212-mycnf.png)

broot tries to select the most relevant file. You can still go from one match to another one using `<tab>` or arrow keys.

You may also search with a regular expression. To do this, add a `/` before or after the pattern.

# See what takes space

![size](img/20190128-only-folders-with-size.png)

To toggle size display, type `:s`. Sizes are computed in the background, you don't have to wait for them when you navigate.

# Apply a personal shortcut to a file

![size](img/20190128-edit.png)

Just find the file you want to edit with a few keystrokes, type `:e`, then `<enter>` (you should define your preferred editor, see [documentation](documentation/usage.md#verbs)).

# More...

See the complete [Documentation](documentation/usage.md).

