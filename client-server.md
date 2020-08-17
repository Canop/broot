
# Presentation

`"client-server"` is a feature declared in Cargo.toml.

It's not enabled by default.

If you want to compile with the feature, do

    cargo build --release --features "client-server"

To run in debug mode with the feature, do

    cargo run --features "client-server" --

This feature add 3 launch arguments that you can see with

    cargo run --features "client-server" -- --help

Those arguments are:

* `--listen <instance_name>` : listen on a specific socket
* `--send <instance_name>`: send the command(s) to the given server and quit
* `--get-root`: ask the server for its current root (in the active panel)

For example if you start broot with

    br --listen my_broot

broot will run normally but will *also* listen to commands sent from elsewhere (using linux sockets).

If you want to do the same in debug mode, it's

    cargo run --features "client-server" -- --listen my_broot

Now that the "server" is running, try

    br --send my_broot -c "img;:parent;:focus"

this will make the running "server" search for something like "img" and focus its parent.

If you run

    br --send my_broot --get-root

then the server's current root is printed on stdout.

If you pass neither the `--get-root` nor the `--cmd` (shortened in `-c`) argument, then the server is told to focus the current directory or the path given as argument.

# Development

This feature started here: https://github.com/Canop/broot/issues/225

and is being discussed and developed between @Canop (@dystroy on Miaou) and @SRGOM (@fiAtcBr on Miaou) and you're welcome to contribute on [Miaou](https://miaou.dystroy.org/3490).

# Possible use cases (for users to utilize this).

## Auto-updating file viewer

You can start an instance of broot with `broot --listen global_file_viewer` and add a hook in your shell to update this upon directory change. A hook for zsh would involve adding the following code in your init file `chpwd(){ ( broot --send global_file_viewer "$PWD" & ) > /dev/null 2>&1 }`

# TODO:

- [ ] have convincing use cases implemented
- [ ] make available with TCP localhost sockets on windows ?
- [ ] stop hiding behind a compilation flag (migth want to optimize before)


