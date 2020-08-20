# Introduction

This document describes using broot as a global `ls` replacement and file navigator. This is a beta feature.

# Motivation

Often the first thing one does after entering a directory in a terminal is running ls (and sometimes running ls <child-item>) to familiarize oneself with the directory. Broot can remove the need for ls. 

# Setup 

## Broot 

This setup works best with either multiple monitors or some other way to have a window always visible.

* Build broot with the 'client-server' feature. `cargo build --release --features "client-server"`
* Open a broot instance with the `listen` argument, thus-  `broot --listen GLOBAL_FM`. 
* Move this broot instance to either a secondary monitor or somewhere it is always visible (some window managers allow a pinned window in all workspaces).


## Shell


### ls replacement

In your shell of choice, add a hook for cd. 

Hooks for some popular shells:

#### zsh

```
chpwd(){ ( broot --send global "$PWD" & ) > /dev/null 2>&1 }
```

#### Powershell

```
Remove-Alias cd
function cd()
{
	Set-Location @args
	broot --send global (Get-Location)
}
```

From here on, all `cd` invocations (in a new shell) would update the global broot instance for an instant overview.

### File navigator replacement

`broot` also supports a `--get-root` parameter which queries 
the currently open directory from broot. 
A workflow would be to navigate to your directory of choice
inside the global instance and then issuing 
`cd $(broot/target/release/broot --send global --get-root)` in the shell (zsh/bash).
If your shell supports shortcut keys, you could map this command to a shortcut.


# Current limitations 

Broot doesn't update live. Careful if you expect a file to be created and you don't see it in broot. A workaround is to invoke a `cd .`.

# References

[Documentation](https://github.com/Canop/broot/blob/master/client-server.md)

