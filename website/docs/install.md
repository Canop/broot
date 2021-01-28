
The current version of broot works on linux, mac and windows (win 10+).

!!! Note
	**Windows users:** broot may need additional rights at first use in order to write its configuration file.
	Some users on Windows also report problems with the colon. Remember that a space can be used instead of a colon.
	You should also use the new PowerShell terminal and not the old cmd.exe which isn't supported.

!!! Note
	If you use cargo and there's a compilation error, it usually means you have an old version of the compiler, and you should update it (for example with `rustup update`).

# From precompiled binaries

Binaries are made available at every release in [download](https://dystroy.org/broot/download).

Direct links:

Target|Files
-|-
Linux | [x86_64-linux](https://dystroy.org/broot/download/x86_64-linux/broot)
Linux/musl | [x86_64-unknown-linux-musl](https://dystroy.org/broot/download/x86_64-unknown-linux-musl/broot)
Raspberry | [armv7-unknown-linux-gnueabihf](https://dystroy.org/broot/download/armv7-unknown-linux-gnueabihf/broot)
Windows 10+ | [x86_64-pc-windows-gnu](https://dystroy.org/broot/download/x86_64-pc-windows-gnu/broot.exe)
Shell completion | [completion/](https://dystroy.org/broot/download/completion/)

You may download previous releases on [GitHub releases](https://github.com/Canop/broot/releases).

When you download executable files, you'll have to ensure the shell can find them. An easy solution on linux is for example to put them in `/usr/local/bin`. You may also have to set them executable using `chmod +x broot`.

# From crates.io

You'll need to have the [Rust development environment](https://www.rust-lang.org/tools/install) installed and up to date.

Once it's installed, use cargo to install broot:

    cargo install broot

# From source

You'll need to have the [Rust development environment](https://www.rust-lang.org/tools/install) installed.

Fetch the [Canop/broot](https://github.com/Canop/broot) repository, move to the broot directory, then run

```bash
cargo install --path .
```

If you want a custom compilation, have a look at the [optional features documentation](https://github.com/Canop/broot/blob/master/features.md).

# Third party repositories

Those packages are maintained by third parties and may be less up to date.

## Homebrew

    brew install broot

## Alpine Linux

    apk add broot

*note: broot package is available in Alpine 3.13 and newer*

## APT / Deb

Ubuntu and Debian users may use this apt repository: [https://packages.azlux.fr/](https://packages.azlux.fr/)

## NetBSD

    pkgin install broot

-----------------------------------

Now you should [install the br shell function](../install-br/).
