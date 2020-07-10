This page lists a few problems which were observed more than once and the best current answer.

Please come to [miaou](https://miaou.dystroy.org/3490?broot) if something isn't clear or if you want to propose a change or addition.

# Those aren't my colors

Broot's initial colors ensure that everything is readable whathever your settings.
But you may want to have something more similar to your usual terminal colors, or maybe to define the whole skin.

* [changing the skin](../skins/)
* [set a transparent background](../skins/#transparent-background)
* [set file extension dependent colors](../conf_file/#colors-by-file-extension)

# There are artifacts on tmux or screen

Most problems with terminal multiplexers seem related to their bad proxying of some style related codes.

* [relevant issue](https://github.com/Canop/broot/issues/248)

A workaround is to create a skin (for example by uncommenting the one in `conf.toml`) and to remove all `Italic` and `Bold`.

# alt-enter (or other shortcut) isn't available

Most terminals intercept a few keybaord shorctut for their own features.

If a shorcut isn't available for broot and you can't or don't want to remap the one of your terminal, the solution is to change the shortcut in broot.

* [specific solution for alt-enter](https://github.com/Canop/broot/issues/86#issuecomment-635974557)
* [general shortcut configuration](../conf_verbs/#keyboard-key)

# Searching is slow when I mount a slow remote disk

Broot dives into all visible directories to look for the best matches.
This can be a problem if you mount a remote disk.
The solution is to tell broot not to automatically enter the directory. It will still be entered if you focus it yourself.

* [define a special-path in configuration](../conf_file/#special-paths)
* [relevant issue](https://github.com/Canop/broot/issues/251)

# My system has neither xdg-open nor any equivalent and doesn't know how to open files

In such a case, which isn't rare in server systems, you can rebind <kbd>enter</kbd> to the program of your choice.

* [change standard file opening](../tricks/#change-standard-file-opening)

# Broot doesn't seem to work on msysgit or git bash

I have no solution for that. If you know how to tackle the problem, the maintainers of [Crossterm](https://github.com/crossterm-rs/crossterm) would be interested too.

# Broot doesn't seem to work correctly on Windows before 10

Even Microsoft doesn't support them anymore. If you have a cheap solution it's welcome but I don't have any.

# Broot doesn't seem as fast or feature complete on Windows

It isn't. I'm not a Windows programmer and I don't even have a machine to test. I'd welcome the help of a programmer with the relevant competences and the will to improve broot.
