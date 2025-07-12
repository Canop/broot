
broot is convenient to find a directory then `cd` to it, which is done using <kbd>alt</kbd><kbd>enter</kbd> or `:cd`.

But broot needs a companion function in the shell in order to be able to change directory.

# Automatic shell function installation

This is normally the easiest solution and it's safe.

When you start broot, it checks whether the `br` shell function seems to have been installed (or to have been refused). If needed, and if the used shell seems compatible, then broot asks the permission to register this shell function.

When it's done, you can do `br` to launch broot, and typing <kbd>alt</kbd><kbd>enter</kbd> will cd for you.

Supported shells today are bash, zsh, fish, nushell, and powershell.

!!! Note
	**Mac / Zsh users:** If your shell is zsh and there's no `.zshrc` file yet, broot won't patch it. The solution is to create it with `touch ~/.zshrc` then run `broot --install`

!!! Note
	**Windows users:** broot may need additional rights at first use in order to write its configuration file. You may also have to allow script execution (`set-executionpolicy unrestricted`)

!!! Note
	**Nushell users:** use either `alias` or `def --env` when aliasing the `br` shell function. `def` alone won't enable the shell function to perform `cd`.

# Retry the automatic installation

If you have messed with the configuration files, you might want to have the shell function reinstalled.

In order to do this, either remove all broot config files, or launch `broot --install`.

You can also use the `--install` argument when you first refused and then decided you want it installed.

# Manual shell function installation

If you prefer to manage the function sourcing yourself, or to automate the installation your way, or if you use an unsupported configuration, you still can get some help of broot:

`broot --print-shell-function bash` (you can replace `bash` with `zsh`, `fish`, or `nushell`) outputs a recommended shell function.

`broot --set-install-state installed` tells broot the `br` function is installed (other possible values are `undefined` and `refused`).

# `br` alias for Xonsh shell

The shortcut for [xonsh](https://xon.sh/) shell can be installed with using [xontrib-broot](https://github.com/jnoortheen/xontrib-broot)

