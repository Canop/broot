
When you used a toggle, you executed a command in it simplest form: without argument and independant from the current selection.

The simplest verbs are just executed by typing a space (or `:`), then its first letters, then enter.

A verb can be related to the current selection. For example typing `:p` will execute the `:parent` verb, which focuses the parent of the selection (*focusing* a directory means making it the current root).

## Verbs using the selection

The `rm` verb executes the standard `rm` command.

It's defined by this couple (invocation, execution):

```toml
invocation = "rm"
execution = "/bin/rm -rf {file}"
```

Selection based arguments:

name | expanded to
-|-
`{file}` | the complete path of the current selection
`{parent}` | the complete path of the current selection's parent
`{directory}` | the closest directory, either `{file}` or `{parent}`
`{other-panel-file}` | the complete path of the current selection in the other panel
`{other-panel-parent}` | the complete path of the current selection's parent in the other panel
`{other-panel-directory}` | the closest directory, either `{file}` or `{parent}` in the other panel

Several selection based arguments can be used. For example the (built-in) `:copy_to_panel` verb is defined as

```toml
invocation = "copy_to_panel"
execution = "/bin/cp -r {file} {other-panel-directory}"
```

When you type a verb, the execution pattern is completed using the selection(s), the exact command is displayed in the status line:

![rm](../img/20190305-rm.png)

As for filters, hitting <kbd>esc</kbd> clears the command.


## Verbs using user provided arguments

Some commands not only use the selection but also takes one or several argument(s).

For example mkdir is defined as

```toml
invocation = "mkdir {subpath}"
execution = "/bin/mkdir -p {directory}/{subpath}"
```

(it's now a built-in, you won't see it in the config file)

which means that if you type `c/d`, and the file `/a/b/some_file.rs` is selected, then the created directory would be `a/b/c/d`.

Example:

Before you type a subpath, broot tells you, in red, the argument is missing:

![md](../img/20191112-md-missing-subpath.png)

If we type an argument, the command to execute is computed and shown:

![md](../img/20191112-md-list.png)

In this screenshot, we didn't type `mkdir` or its start but `md`. That's because the complete definition of this verb includes this line:

	shortcut = "md"

!!!	Note
	The help screen lists the whole set of available verbs, including the ones coming from the configuration.

## Builtins & external commands, leaving or not

There are two types of verbs, differing by their *execution* pattern (which will be covered in more details in the [configuration page](configuration.md#verbs)):

* buitin features, whose execution starts with `:`, apply internal functions, for example `:toggle_perm` to trigger computation and display of unix file permissions
* external commands, whose execution implies calling an external program, for example `rm -rf {file}`

A command may leave broot (for example to start a program), or not (the tree will be refreshed).

## Adding verbs

You may start with the common set of verbs but you'll very quickly want to define how to edit or create files, and probably have a few personal commands.

That's why should see [how to configure verbs](../conf_file/#verbs).
