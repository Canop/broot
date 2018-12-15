# Broot (pronounce "b-root")

[![Chat on Miaou](https://miaou.dystroy.org/static/shields/room-fr.svg?v=1)](https://miaou.dystroy.org/3?Code_et_Croissants)

An interactive tree view, a fuzzy search, a balanced BFS descent and customizable commands.

Get an overview of a directory, even a big one:

![overview](doc/20181215-overview.png)

See what takes space:

![size](doc/20181215-only-folders-with-size.png)

Never lose track of file hierarchy while you fuzzy search:

![size](doc/20181215-search.png)

Apply a personal shorcut to a file:

![size](doc/20181215-edit.png)

And everything's blazingly fast and nothing never blocks:

[![asciicast](https://asciinema.org/a/IfA8tMykpeQbKIFGe9J3ljeSu.svg)](https://asciinema.org/a/IfA8tMykpeQbKIFGe9J3ljeSu?theme=tango)

## Usage

Launch it (see `broot --help` for launch options).

Type a few letters to fuzzy search files or directories.

Enter brings you to a directory or opens a file.

A command starts with a space or `:` (as you like) and is usually only one letter (for example `:s` to toggle sizes, `:q` to quit, `:p` to go up the tree, etc.).

Type `?` to see the list of commands and the path to their configuration.

## Installation

### From Source

You'll need to have the Rust development environment installed.

Fetch the Canop/broot repository, move to the broot directory, then run

    cargo build --release

The executable is written in the `target/release` directory.


### From up to date precompiled binaries

* [x86_64-linux](https://dystroy.org/broot/x86_64-linux/broot)

## Development

To ease tests during development, a log file can be generated (and followed using tail -f) by using the BROOT_LOG env variable.

For example:

    BROOT_LOG=debug cargo run

If you want to discuss the code or features of broot, please come to [our chat](https://miaou.dystroy.org/3?Code_et_Croissants). If you'd like a new feature, don't hesitate to ask for it.
