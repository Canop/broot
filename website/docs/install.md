
The current version of broot works on linux, mac and windows (win 10+).

!!! Note
	**Windows users:** broot may need additional rights at first use in order to write its configuration file.
	Some users on Windows also report problems with the colon. Remember that a space can be used instead of a colon.
	You should also use a modern terminal, for example the [new Microsoft one](https://github.com/microsoft/terminal)

# From precompiled binaries

Binaries are made available at every release in [download](https://dystroy.org/broot/download).

The archives there contain all precompiled versions, as well as the licenses and other files. They're the prefered distribution way.

Direct links:

Target|Details|Download
-|-|-
Linux | clipoard support| [x86_64-linux](https://dystroy.org/broot/download/x86_64-linux/broot)
Android | clipoard support | [aarch64-linux-android](https://dystroy.org/broot/download/aarch64-linux-android/broot)
Linux/glibc | no clipoard support, compatible with older GLIBC | [x86_64-unknown-linux-gnu](https://dystroy.org/broot/download/x86_64-unknown-linux-gnu/broot)
Linux/musl | no clipoard support | [x86_64-unknown-linux-musl](https://dystroy.org/broot/download/x86_64-unknown-linux-musl/broot)
Raspberry | no clipoard support | [armv7-unknown-linux-gnueabihf](https://dystroy.org/broot/download/armv7-unknown-linux-gnueabihf/broot)
Windows 10+ | clipoard support | [x86_64-pc-windows-gnu](https://dystroy.org/broot/download/x86_64-pc-windows-gnu/broot.exe)
Shell completion  | | [completion/](https://dystroy.org/broot/download/completion)

You may download previous releases on [GitHub releases](https://github.com/Canop/broot/releases).

When you download executable files, you'll have to ensure the shell can find them. An easy solution on linux is for example to put them in `/usr/local/bin`. You may also have to set them executable using `chmod +x broot`.

# From crates.io

You'll need to have the [Rust development environment](https://www.rustup.rs) installed and up to date.

Once it's installed, use cargo to install broot:

```bash
cargo install --locked broot
```

# From source

You'll need to have the [Rust development environment](https://www.rustup.rs) installed.

Fetch the [Canop/broot](https://github.com/Canop/broot) repository, move to the broot directory, then run

```bash
cargo install --locked --path .
```

If you want a custom compilation, have a look at the [optional features documentation](https://github.com/Canop/broot/blob/master/features.md).

!!! Note
	If there's a compilation error, it most often means either that you're missing some compilation dependency (on ubuntu/debian try `sudo apt install build-essential`) or that you have an old version of the compiler, and you should update it (for example with `rustup update`).

# Third party repositories

Those packages are maintained by third parties and may be less up to date.

## Homebrew

    brew install broot

## MacPorts

    sudo port selfupdate
    sudo port install broot

## Alpine Linux

    apk add broot

*note: broot package is available in Alpine 3.13 and newer*

## APT / Deb

Ubuntu and Debian users may use this apt repository: [https://packages.azlux.fr/](https://packages.azlux.fr/)

## NetBSD

    pkgin install broot

## Gentoo Linux

    emerge broot

-----------------------------------

Now you should [install the br shell function](../install-br/).
