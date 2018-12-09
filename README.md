# Broot (pronounce "b-root")

[![Chat on Miaou](https://miaou.dystroy.org/static/shields/room-fr.svg?v=1)](https://miaou.dystroy.org/3?Code_et_Croissants)

A interactive tree view, a fuzzy search, a balanced BFS descent and customizable commands for:

* a clear overview of a directory tree
* a very fast access to any file, without being lost or disoriented
* a personal launcher

[![asciicast](https://asciinema.org/a/vxzgahNhoYfDVpEfp7OmZhLDQ.svg)](https://asciinema.org/a/vxzgahNhoYfDVpEfp7OmZhLDQ)

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

If you want to discuss the code or features of broot, please come to [our chat](https://miaou.dystroy.org/3?Code_et_Croissants).
