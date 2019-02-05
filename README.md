# Broot

[![Build Status](https://travis-ci.org/Canop/broot.svg?branch=master)](https://travis-ci.org/Canop/broot)
[![Chat on Miaou](https://miaou.dystroy.org/static/shields/room-en.svg?v=1)](https://miaou.dystroy.org/3490?broot)
[![Chat on Miaou](https://miaou.dystroy.org/static/shields/room-fr.svg?v=1)](https://miaou.dystroy.org/3490?broot)

A better way to navigate directories.

### Get an overview of a directory, even a big one:

![overview](img/20190128-overview.png)

Notice the "unlisted"? That's what makes it usable where the old `tree` command would produce pages of output.

`.gitignore` files are properly dealt with to put unwanted files out of your way (you can ignore them though, see documentation).

### Find a directory then `cd` to it:

![cd](img/20190128-cd.png)

This way, you can navigate to a directory with the minimum amount of keystrokes, even if you don't exactly remember where it is.

broot is fast and never blocks, even when you make it search a big slow disk (any keystroke interrupts the current search to start the next one).

Most useful keys for this:

* the letters of what you're looking for
* `<enter>` to select a directory (staying in broot)
* `<esc>` to get back to the previous state or clear your search
* `<alt><enter>` to get back to the shell having `cd` to the selected directory ([see below](#use-broot-for-navigation))
* `:q` if you just want to quit (`<esc>` works too)

### Never lose track of file hierarchy while you fuzzy search:

![size](img/20190128-search.png)

broot tries to select the most relevant file. You can still go from one match to another one using `<tab>` or arrow keys.

You may also search with a regular expression. To do this, add a `/` before or after the pattern.

Complex regular expression are possible, but you'll probably most often use a regex to do an "exact" search, or search an expression at the start or end of the filename.

For example, assuming you look for your one file whose name contains `abc` in a big directory, you may not see it immediately because of many fuzzy matches. In that case, just add a slash at the end to change you fuzzy search into an exact expression: `abc/`.

And if you look for a filename *ending* in `abc` then you may anchor the regex: `abc$/`.

### See what takes space:

![size](img/20190128-only-folders-with-size.png)

To toggle size display, type `:s`. Sizes are computed in the background, you don't have to wait for them when you navigate.

### Apply a personal shortcut to a file:

![size](img/20190128-edit.png)

Just find the file you want to edit with a few keystrokes, type `:e`, then `<enter>` (you should define your preferred editor, see [documentation](documentation.md#verbs)).

### More...

See the complete [Documentation](documentation.md).

## Installation

### Compile

You'll need to have the [Rust development environment](https://www.rust-lang.org/tools/install) installed.

#### From crates.io

    cargo install broot

#### From Source

Fetch the Canop/broot repository, move to the broot directory, then run

    cargo build --release

The executable is written in the `target/release` directory (you might want to move it to your `/usr/bin`, or to add the release directory to your path).

### From up to date precompiled binaries

* [x86_64-linux](https://dystroy.org/broot/x86_64-linux/broot)

## cd

broot is convenient to find a directory then `cd` to it, which is done using `<alt><enter>` or `:cd`.

But broot needs a companion function in the shell in order to be able to change directory. To enable this feature, add this to your `.bashrc` (or the relevant file for another shell):

	# start broot and let it change directory
	function br {
	    f=$(mktemp)

	    (
		set +e
		broot --out "$f" "$@"
		code=$?
		if [ "$code" != 0 ]; then
		    rm -f "$f"
		    exit "$code"
		fi
	    )
	    code=$?
	    if [ "$code" != 0 ]; then
		return "$code"
	    fi

	    d=$(cat "$f")
	    rm -f "$f"

	    if [ "$(wc -c <(echo -n "$d") | head -c1)" != 0 ]; then
		cd "$d"
	    fi
	}

(You'll have to source the `.bashrc` file or open a new terminal for the function to take effect.)

With this addition, you can do just `br` to launch broot, and typing `<alt><enter>` will cd for you.


## Development

To make tests easier during development, a log file can be generated (and followed using `tail -f`) by using the BROOT_LOG environment variable.

For example:

    BROOT_LOG=debug cargo run

or

    BROOT_LOG=info cargo run

If you want to discuss the code or features of broot, please come to [our chat](https://miaou.dystroy.org/3490?broot). Before you start coding for a PR, it would really be a good idea to come and talk about it.

If you'd like a new feature, don't hesitate to ask for it.
