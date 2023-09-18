This page lists a few problems which were observed more than once and the best current answer.

Please come to [miaou](https://miaou.dystroy.org/3490?broot) if something isn't clear or if you want to propose a change or addition.

# Compilation failure

The common causes are

* an oudated Rust installation. If you're using [rustup](https://rustup.rs), do `rustup update`.
* some tools missing. On Debian like distributions, this can generally be solved with `sudo apt install build-essential libxcb-shape0-dev and libxcb-xfixes0-dev`

# Colors

Broot's initial colors ensure that everything is readable whatever your settings.
But you may want to have something more similar to your usual terminal colors, or maybe to define the whole skin.

* [changing the skin](../skins/)
* [set a transparent background](../skins/#transparent-background)
* [set file extension dependent colors](../conf_file/#colors-by-file-extension)

# Tmux

The first problem you might see is the presence of artifacts. This may happen in other terminal multiplexers too and it seems related to their bad proxying of some style related codes.

* [relevant issue](https://github.com/Canop/broot/issues/248)

A workaround is to create a skin (for example by uncommenting the one in `conf.toml`) and to remove all `Italic` and `Bold`.

Additionally, if backgrounds can't be properly displayed, you may consider [marking selected lines](../conf_file/#selection-mark).

Another problem is the fact the `br` function doesn't set a proper pane name (you'll probably see the name of your shell instead of broot). This may be [solved with a modified shell function](https://github.com/Canop/broot/issues/270).

# Missing shortcut

Most terminals intercept a few keyboard shortcut for their own features. You may need to remap your terminal's default keyboard shortcuts.

I've made a small program which tells you what key combinations are available: [print_key](https://github.com/Canop/print_key).

## Windows Terminal

[Windows Terminal](https://docs.microsoft.com/en-us/windows/terminal/) binds `alt+enter` to the "toggle fullscreen" command by default. To reclaim `alt+enter` for Broot, [add an 'unbound' entry to the actions array in settings.json](https://docs.microsoft.com/en-us/windows/terminal/customize-settings/actions#unbind-keys):

```json
{"command": "unbound", "keys": "alt+enter"}
```

## iTerm2

For Mac users, iTerm2 must also be configured to enable this shortcut:
Go to *Preferences->Profiles->Default->Keys* and add a mapping that maps `⌥Return↩` to `Send Hex Codes: 0x1b 0x0d`. This can be done by clicking the + sign at the bottom to add a mapping, clicking the "Click to Set" area, pressing the desired key combination (⌥Enter a.k.a ⌥Return), choosing the "Send Hex Code" option from the drop-down menu and inserting the following string there: "0x1b 0x0d".

Note that this will change the behavior of `alt+enter` for all terminal windows, and it will no longer send the `return` sequence.

## remap in Broot

If a shortcut isn't available for broot and you can't or don't want to remap the one of your terminal, the solution is to change the shortcut in broot.

* [specific solution for alt-enter](https://github.com/Canop/broot/issues/86#issuecomment-635974557)
* [general shortcut configuration](../conf_verbs/#keyboard-key)


# Slow remote disk

Broot dives into all visible directories to look for the best matches.
This can be a problem if you mount a remote disk.
The solution is to tell broot not to automatically enter the directory. It will still be entered if you focus it yourself.

* [define a special-path in configuration](../conf_file/#special-paths)
* [relevant issue](https://github.com/Canop/broot/issues/251)

# Open files without xdg-open (or equivalent)

In such a case, which isn't rare in server systems, you can rebind <kbd>enter</kbd> to the program of your choice.

* [change standard file opening](../tricks/#change-standard-file-opening)

# Everything feels slow

It's probably your terminal app's fault. You could check that by using any other TUI application.

Most terminal apps are fine but some, made with Electron or worse, or crippled with fancy plugins, take dozens of milliseconds to redraw the screen. You should not use those terminals.

# msysgit or git bash

I have no solution for that. If you know how to tackle the problem, the maintainers of [Crossterm](https://github.com/crossterm-rs/crossterm) would be interested too.

# Windows

Broot isn't as fast or feature complete on Windows.

I'm not a Windows programmer and I don't even have a machine to test. I'd welcome the help of a programmer with the relevant competences and the will to improve broot.

# Windows 9-

Even Microsoft doesn't support versions of Windows before the 10. If you have a cheap solution it's welcome but I don't have any.

# PowerShell Encoding

Some problems in PowerShell are linked to a wrong encoding. You should set it to UTF-8 in your [profile](https://docs.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_profiles?view=powershell-7.1).

This will create a profile if it doesn't exist:
```powershell
if (!(Test-Path -Path $PROFILE)) {
  New-Item -ItemType File -Path $PROFILE -Force
}
```

`notepad $PROFILE` will open the profile config.

Add this line:

```
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
```

Run the following with admin privilege ([source](https://docs.microsoft.com/en-us/powershell/module/microsoft.powershell.security/set-executionpolicy?view=powershell-7.1)):

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
Unblock-File -Path $PROFILE
```

This will ensure your profile is loaded in new terminals/sessions.
You can check it by opening a new terminal then running `[Console]::Out`.
The output should show an encoding of `System.Text.UTF8Encoding`.

# High-Resolution images in Kitty

When using Kitty (and no terminal multiplexer), image preview is normally in high resolution.

If it's not the case, it's probably because the `TERM` environment variable has been redefined. Set either `TERM` or `TERMINAL` to include `kitty`. This can be done several ways, for example by adding `env TERMINAL=xterm-kitty` in your [kitty.conf](https://sw.kovidgoyal.net/kitty/conf/) file

# Edit

The standard `edit` verb, launched with `:e`, starts your favourite terminal editor to edit the selected file.

It works by executing `"$EDITOR +{line} {file}"` which assumes that the `$EDITOR` variable is defined and that your editor takes the line number as argument.

If it doesn't work on your configuration, you should probably just edit this verb definition with a more suitable command, for example `"hx {file}:{line}"` or `"/usr/bin/my-editor --line {line} {file}"`
