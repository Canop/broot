
broot is convenient to find a directory then `cd` to it, which is done using <kbd>alt</kbd><kbd>enter</kbd> or `:cd`.

But broot needs a companion function in the shell in order to be able to change directory.

## Automatic shell function installation

This is normally the easiest solution and it's safe.

When you start broot, it checks whether the `br` shell function seems to have been installed (or
to have been refused). If needed, and if the used shell seems compatible (supported shells today are bash, zsh and fish),
then broot asks the permission to register this shell function.

When it's done, you can do `br` to launch broot, and typing <kbd>alt</kbd><kbd>enter</kbd> will cd for you.

## Retry the automatic installation

If you have messed with the configuration files, you might want to have the shell function reinstalled.

In order to do this, either remove all broot config files, or launch `broot --install`.

You can also use the `--install` argument when you first refused and then decided you want it installed.

## Manual shell function installation

If you prefer to manage the function sourcing yourself, or to automate the installation your way, or if you use an unsupported configuration, you still can get some help of broot:

`broot --print-shell-function bash` (you can replace `bash` with either `zsh` or `fish`) outputs a recommended shell function.

`broot --set-install-state installed` tells broot the `br` function is installed (other possible values are `undefined` and `refused`).

## `br` alias for Nushell

As a shortcut for [Nushell](https://www.nushell.sh/), define the following alias:

    alias br [] { broot | trim | cd $it }

You can bind this command to a key sequence in the [configuration file](../conf_file):

```toml
[[verbs]]
key = "alt-p"
shortcut = "pp"
execution = ":print_path"
```



