
# Launch Broot

When the installation is [complete](../install/#installation-completion-the-br-shell-function), you may start broot with either

	broot

or

	br

If your shell is compatible, you should prefer `br` which enables some features like `cd` from broot.

You can pass as argument the path you want to see, for example

	br ~

Broot renders on `stderr` and can be ran in a subshell, which means you can also (on Unix) do things like

	my_unix_command "$(broot some_dir)"

and quit broot with `:pp` on the selected path. But most often you'll more conveniently simply add your command (and maybe a shortcut) to the [config file](../conf_file/#verbs-shortcuts-and-keys).


# Launch Arguments

**broot** and **br** can be passed as argument the path to display, either a directory or a file. When it's a file, it's opened in preview.

They also accept a few other arguments which you can view with `br --help`.

Most of them are display toggles, which may be useful when aliasing the function but which are accessible from inside the application anyway.

Some of them are a little special, though, and are explained below:

## the `--outcmd` launch argument

Some external commands can't be executed from a program.

This is especially the case of `cd`, which isn't a program but a shell function. In order to have any useful effect, it must be called from the parent shell, the one from which broot was launched, and a shell which isn't accessible from broot.

The trick to enable broot to `cd` your shell when you do `alt-enter` is the following one:

* **br** is a shell function. It creates a temp file whose path it gives as argument to **broot** using `--outcmd`
* when you do `alt-enter`, **broot** writes `cd your-selected-path` in this file, then quits
* **br** reads the file, deletes it, then evaluates the command

Most users have no reason to use `--outcmd` on their own, but it can still be used to write an alternative to **br** or to port it to shells which aren't currently supported.

<a name=cmd></a>
## the `--cmd` launch argument

This argument lets you pass commands to broot. Those commands are executed exactly like any command you would type yourself in the application.

Commands must be separated. The default separator is the semicolon (`;`) but another separator may be provided using the `BROOT_CMD_SEPARATOR` environment variable (the separator may be several characters long if needed).

Broot waits for the end of execution of every command.

For example if you launch

    br --cmd cow /

Then broot is launched in the `/` directory and there's a filter typed for you.

If you do

    br --cmd "/^vache;:p"

Then broot looks for a file whose name starts with "vache" and focus its parent.

If you do

    br -c "/mucca$;:cd"

then broot searches for a file whose name ends with "mucca", and `cd` to the closest directory, leaving you on the shell, in your new directory (you may not have the time to notice the broot GUI was displayed).

If you do

	BROOT_CMD_SEPARATOR=@ broot -c ":gi@target@:pp"

then broot toggles the git_ignore filter, searches for `target` then prints the selection path on stdout (when doing it in my broot repository, I get `/home/dys/dev/broot/target`).

The `--cmd` argument may be the basis for many of your own shell functions or programs.

# Environment Variables

Most users don't have to bother with environment variables.

But they come handy in some cases, so here's a complete reference of the variables read by broot.

Variables whose names doesn't start with `BR_` or `BROOT_` aren't specific to broot and may be already present in your system.

variable | usage
-|-
`BROOT_CONFIG_DIR` | Optional path to the config directory. If not set, broot uses the conventions of the system, for example `~/.config/broot`
`BR_INSTALL` | Setting it to `no` prevents broot for installing the `br` shell function
`BROOT_CMD_SEPARATOR` | Set the separator to use in command sequences, e.g. `BROOT_CMD_SEPARATOR=',' br -c 'filter,:pp'`
`BROOT_LOG` | Set up the [log level](../community/#log) (default is none), for example `BROOT_LOG=debug br`
`COLORTERM` | If this conventional variable contains `24bit` or `truecolor`, then broot won't limit itself to a reduced set of colors when rendering images. This may also be set in conf with `true_colors: true`
`TERM` or `TERMINAL` | If one of them contains `kitty`, then broot will use Kitty's [terminal graphics protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/) to render images in high definition
`TERM_PROGRAM` and `TERM_PROGRAM_VERSION` | If the current terminal is [Wezterm](https://wezfurlong.org/wezterm/index.html) with a recent enough version, broot recognizes it with those variables and uses the Kitty's terminal graphics protocol to render images
`COLORFGBG` | This is one of the ways the [terminal-light](https://github.com/Canop/terminal-light) library uses to detect whether your terminal is set in dark or light mode
