
The current version of broot works on linux, mac and windows (win 10+).

While the linux version is quite well tested the other versions are lacking testers and maybe involved developpers.

Precompiled binaries and the crates.io repository are updated at the same time, with tested releases.

If you prefer to get the very last version, even when not tagged, you may compile from the sources available on GitHub.

# From precompiled binaries

The last one is always made available at:

* [x86_64-linux](https://dystroy.org/broot/download/x86_64-linux/broot)
* [x86_64-pc-windows-gnu](https://dystroy.org/broot/download/x86_64-pc-windows-gnu/broot.exe)

You may download previous releases on [GitHub releases](https://github.com/Canop/broot/releases).

# From crates.io

You'll need to have the [Rust development environment](https://www.rust-lang.org/tools/install) installed.

Once it's installed, use cargo to install broot:

    cargo install broot

# From source

You'll need to have the [Rust development environment](https://www.rust-lang.org/tools/install) installed.

Fetch the Canop/broot repository, move to the broot directory, then run

    cargo install --path .

!!! Note
	Windows users: broot may need additional rights at first use in order to write its configuration file

# Homebrew

If you're using [homebrew](https://brew.sh/), you can use the `brew install` command:

    brew install broot

# Installation Completion : the `br` shell function

broot is convenient to find a directory then `cd` to it, which is done using `<alt><enter>` or `:cd`.

But broot needs a companion function in the shell in order to be able to change directory.

When you start broot, it checks whether the `br` shell function seems to have been installed (or
to have been refused). If needed, and if the used shell seems compatible (supported shells today are bash, zsh and fish),
then broot asks the permission to register this shell function.

If you have messed with the configuration files, you might want to have the shell function reinstalled.

In order to do this, either remove all broot config files, or launch `broot --install`.

When it's done, you can do just `br` to launch broot, and typing `<alt><enter>` will cd for you.


