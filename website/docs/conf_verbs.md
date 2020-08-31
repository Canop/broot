
The most important part of broot configuration is the `[[verbs]]` sections, which let you define new commands or shortcuts.

## Verb Definition Attributes

You can define a new verb in the TOML configuration file with a `[[verbs]]` section similar to this one:

```toml
[[verbs]]
invocation = "edit"
key = "F2"
shortcut = "e"
apply_to = "file"
execution = "/usr/bin/nvim {file}"
leave_broot = false
```

The possible attributes are:

name | mandatory | role
-|-|-
invocation | no | how the verb is called by the user, with placeholders for arguments
execution | yes | how the verb is executed
key | no | a keyboard key triggering execution
shortcut | no | an alternate way to call the verb (without the arguments part)
leave_broot | no | whether to quit broot on execution (default: `true`)
from_shell | no | whether the verb must be executed from the parent shell (needs `br`, default: `false`). As this is executed after broot closed, this isn't compatible with `leave_broot = false`
apply_to | no | the type of selection this verb applies to, may be `"file"`, `"directory"` or `"any"`. You may declare two verbs with the same key if the first one applies to only files or only directories
set_working_dir | no | whether the working dir of the process must be set to the currenly selected directory (default to false)

!!!	Note
	The `from_shell` attribute exists because some actions can't possibly be useful from a subshell. For example `cd` is a shell builtin which must be executed in the parent shell.

## Shortcuts and Verb search

**broot** looks for the first token following a space or `:` and tries to find the verb you want.

* If what you typed is exactly the shortcut or name of a verb, then this verb is selected: broot explains you what it would do if you were to type `enter`
* If there's exactly one verb whose name or shortcut starts with the characters you typed, then it's selected
* if there are several verbs whose name or shortcut start with the characters you typed, then broot waits for more
* if no verb has a name or shortcut starting with those characters, broot tells you there's a problem

Knowing this algorithm, you may understand the point in the following definition:

```toml
[[verbs]]
invocation = "p"
execution = ":parent"
```

This verb is an alias to the internal builtin already available if you type `:parent`.

Its interest is that if you do `:p`, then `enter`, it is executed even while there are other verbs whose invocation pattern starts with a `p`.

Use shortcuts for verbs you frequently use.

## Keyboard key

The main keys you can use are

* The function keys (for example <kbd>F3</kbd>)
* Ctrl and Alt keys (for example <kbd>ctrl</kbd><kbd>T</kbd>  or <kbd>alt</kbd><kbd>a</kbd>)

It's possible to define a verb just to add a trigger key to an internal verb.

For example you could add those mappings:

```toml
[[verbs]]
invocation = "root"
key = "F9"
execution = ":focus_root"

[[verbs]]
invocation = "home"
key = "ctrl-H"
execution = ":focus_user_home"

[[verbs]]
key = "alt-j"
execution = ":line_down"

[[verbs]]
invocation = "top"
key = "F6"
execution = ":select_first"

[[verbs]]
invocation = "bottom"
key = "F7"
execution = ":select_last"

[[verbs]]
invocation = "open"
key = "crtl-O"
execution = ":open_stay"

[[verbs]]
invocation = "edit"
key = "F2"
shortcut = "e"
execution = "$EDITOR +{line} {file}"
from_shell = true
```

Then,

* when doing <kbd>alt</kbd><kbd>J</kbd>, you would move the selection down (notice we don't need an invocation)
* when doing <kbd>Ctrl-H</kbd>, you would go to you user home (`~` when on linux),
* you would open files (without closing broot) with <kbd>ctrl-O</kbd>,
* <kbd>F7</kbd> would select the last line of the tree,
* and you'd switch to your favorite editor with <kbd>F2</kbd>

Beware that consoles intercept some possible keys. Many keyboard shortcuts aren't available, depending on your configuration. Some keys are also reserved in broot for some uses, for example the <kbd>enter</kbd> key always validate an input command if there's some. The <kbd>Tab</kbd>, <kbd>delete</kbd>, <kbd>backspace</kbd>, <kbd>esc</kbd> keys are reserved too.

### Verbs not leaving broot

If you set `leave_broot = false`, broot won't quit when executing your command, but it will update the tree.

This is useful for commands modifying the tree (like creating or moving files).

## Verb Arguments

The execution of a verb can take one or several arguments.

For example it may be defined as `/usr/bin/vi {file}̀`.

Some arguments are predefined in broot and depends on the current selection:

name | expanded to
-|-
`{file}` | the complete path of the current selection
`{line}` | number of the selected line in the previewed file
`{parent}` | the complete path of the current selection's parent
`{directory}` | the closest directory, either `{file}` or `{parent}`
`{other-panel-file}` | the complete path of the current selection in the other panel
`{other-panel-parent}` | the complete path of the current selection's parent in the other panel
`{other-panel-directory}` | the closest directory, either `{file}` or `{parent}` in the other panel

!!!	Note
	when you're in the help screen, `{file}` is the configuration file, while `{directory}` is the configuration directory.

But you may also define some arguments in the invocation pattern. For example:

```toml
[[verbs]]
invocation = "mkdir {subpath}"
execution = "/bin/mkdir -p {directory}/{subpath}"
```

(this one has now been made standard so you don't have to write it in the configuration file)

In this case the subpath is read from what you type:

![md sub](img/20190306-md.png)

As you see, there's a space in this path, but it works. **broot** tries to determine when to wrap path in quotes and when to escape so that such a command correctly works.

It also normalizes the paths it finds which eases the use of relative paths:

![mv](img/20190306-mv.png)

Here's another example, where the invocation pattern defines two arguments by destructuring:

```toml
[[verbs]]
invocation = "blop {name}\\.{type}"
execution = "/bin/mkdir {parent}/{type} && /usr/bin/nvim {parent}/{type}/{name}.{type}"
from_shell = true
```

And here's how it would look like:

![blop](img/20190306-blop.png)

Notice the `\\.` in the invocation pattern ? That's because it is interpreted as a regular expression
(with just a shortcut for the easy case, enabling `{name}`).


The whole regular expression syntax may be useful for more complex rules.
Let's say we don't want the type to contain dots, then we do this:

```toml
[[verbs]]
invocation = "blop {name}\\.(?P<type>[^.]+)"
execution = "/bin/mkdir {parent}/{type} && /usr/bin/nvim {parent}/{type}/{name}.{type}"
from_shell = true
```

You can override the default behavior of broot by giving your verb the same shortcut or invocation than a default one.

## Built In Verbs

Here's a list of actions you can add an alternate shortcut or keyboard key for:

invocation | default key | default shortcut | behavior / details
-|-|-|-
:back | <kbd>Esc</kbd> | - | back to previous app state (see Usage page) |
:cd | <kbd>alt</kbd><kbd>enter</kbd> | - | leave broot and cd to the selected directory (needs the br shell function)
:chmod {args} | - | - | execute a chmod
:close_preview | - | - | close the preview panel
:copy_path | <kbd>alt</kbd><kbd>c</kbd> | - | copy path
:cp {newpath} | - | - | copy the file or directory to the provided name
:help | <kbd>F1</kbd> | - | open the help page. Help page can also be open with <kbd>?</kbd>
:focus | <kbd>enter</kbd> | - | set the selected directory the root of the displayed tree |
:line_down | <kbd>↓</kbd> | - | scroll one line down or select the next line
:line_up | <kbd>↑</kbd> | - | scroll one line up or select the previous line
:mkdir {subpath} | - | md | create a directory
:mv {newpath} | - | - | move the file or directory to the provided path
:open_stay | <kbd>enter</kbd> | - | open the selected file in the default OS opener, or focus the directory
:open_preview | - | - | open the preview panel
:open_leave | <kbd>alt</kbd><kbd>enter</kbd> | - | open the selected file in the default OS opener and leave broot
:open_stay_filter | - | - | focus the directory but keeping the current filtering pattern
:page_down | <kbd>⇟</kbd> | - | scroll one page down
:page_up | <kbd>⇞</kbd> | - | scroll one page up
:parent | - | - | focus the parent directory
:print_path | - | pp | print path and leave broot
:print_relative_path | - | pp | print relative path and leave broot
:print_tree | - | pt | print tree and leave broot
:quit | <kbd>ctrl</kbd><kbd>q</kbd> | q | quit broot
:refresh | <kbd>F5</kbd> | - | refresh the displayed tree and clears the directory sizes cache
:rm | - | - | remove the selected file or directory. To stay safe, don't define a keyboard key for this action
:select_first | - | - | select the first line
:select_last | - | - | select the last line
:sort_by_count | - | - | sort by count (only one level of the tree is displayed)
:sort_by_date | - | - | sort by date
:sort_by_size | - | - | sort by size
:toggle_counts | - | - | toggle display of total counts of files per directory
:toggle_dates | - | - | toggle display of last modified dates (looking for the most recently changed file, even deep)
:toggle_files | - | - | toggle showing files (or just folders)
:toggle_git_ignore | - | - | toggle git ignore handling (auto, no or yes)
:toggle_git_file_info | - | - | toggle display of git file information
:toggle_git_status | - | - | toggle showing only the file which would show up on `git status`
:toggle_hidden | - | - | toggle display of hidden files (the ones whose name starts with a dot on linux)
:toggle_perm | - | - | toggle display of permissions (not available on Windows)
:toggle_preview | - | - | toggle display of the preview panel
:toggle_sizes | - | - | toggle the size mode
:toggle_trim_root | - | - | toggle trimming of top level files in tree display
:up_tree | - | - | focus the parent of the current root

Note that

- you can always call a verb with its default invocation, you don't *have* to define a shortcut
- verbs whose invocation needs an argument (like `{newpath}`) can't be triggered with just a keyboard key.

## Input related verbs

Some internal actions can be bound to a key shortcut but can't be called explicitly because they directly act on the input field:

name | default binding | behavior
-|-|-
:input_del_char_left | <kbd>del</kbd> | "delete the char left of the cursor",
:input_del_char_below | <kbd>suppr</kbd> | "delete the char left at the cursor's position",
:input_del_word_left | - | "delete the word left of the cursor",
:input_del_word_right | - | "delete the word right of the cursor",
:input_go_to_end | <kbd>end</kbd> | "move the cursor to the end of input",
:input_go_left | <kbd>←</kbd> | "move the cursor to the left",
:input_go_right | <kbd>→</kbd> | "move the cursor to the right",
:input_go_to_start | <kbd>home</kbd> | "move the cursor to the start of input",
:input_go_word_left | - | "move the cursor one word to the left",
:input_go_word_right | - | "move the cursor one word to the right",

You may add this kind of shortcuts:

```toml
[[verbs]]
key = "alt-b"
execution = ":input_go_word_left"

[[verbs]]
key = "alt-f"
execution = ":input_go_word_right"

[[verbs]]
key = "alt-l"
execution = ":input_del_word_left"

[[verbs]]
key = "alt-r"
execution = ":input_del_word_right"
```


## Focus

The `:focus` internal has many uses.

It can be used without explicit argument in which case it takes the selection (for example `:!focus` is equivalent to <kbd>ctrl</kbd><kbd>→</kbd>).

It can be used with an argument, for example you can go to a specific place without leaving broot by typing ` fo /usr/bin` then <kbd>enter</kbd>.

It serves as base for several built-in commands, like `:home` whose execution is `:focus ~` (`~` is interpreted in broot as the user home even on Windows).

And you can add your own ones:

```toml
[[verbs]]
key = "ctrl-up"
execution = ":focus .."

[[verbs]]
key = "ctrl-d"
execution = ":focus ~/dev"
```

