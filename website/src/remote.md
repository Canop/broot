
# Presentation

Broot can also act as client or server, which lets you

* control broot from another process
* query the state of broot from another process

Example use cases:

* synchronize broot with another program (shell, editor, etc.), both ways
* have a viewer automatically display the file selected in broot
* have broot automatically show the content of a directory focused in another program

**Note:**
The implementation is based on unix sockets, or on named pipes on Windows.

# Usage

3 launch arguments are involved:

* `--listen <instance_name>` : listen on a specific socket
* `--send <instance_name>`: send the command(s) to the given server and quit
* `--get-root`: ask the server for its current root (in the active panel)

For example if you start broot with

    br --listen my_broot

broot will run normally but will *also* listen to commands sent from elsewhere (using a unix socket, or a named pipe on Windows).


Now that the "server" is running, try launching a command from another terminal:

    br --send my_broot -c "img;:parent;:focus"

this will make the running "server" search for something like "img" and focus its parent.

If you run

    br --send my_broot --get-root

then the server's current root is printed on stdout.

If you pass neither the `--get-root` nor the `--cmd` (shortened in `-c`) argument, then the server is told to focus the current directory or the path given as argument.

# Hooks

## zsh

`chpwd(){ ( broot --send global_file_viewer "$PWD" & ) > /dev/null 2>&1 }`


