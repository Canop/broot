
This page defines the optional features which may be applied on compilation:

* clipboard
* trash
* kitty-csi-check

Feature gating is usually temporary: they may be removed when a technical problem is solved, when a feature becomes "mainstream", or when it's dropped because no user mentioned using it.

## The "clipboard" feature

This feature allows the `:copy_path` verb which copies the currently selected path into the clipboard, as well as copy-pasting from,to,within the input.

Limits:

- the feature doesn't compile right now on some platforms (for example Raspberry)
- on some platforms the content leaves the clipboard when you quit broot (so you must paste while broot is still running)

## The "trash" feature

This feature enables commands for managing the system Trash. They are `:open_trash`, `:delete_trashed_file`, `:restore_trashed_file`, `:purge_trash`.

## The "kitty-csi-check" feature

The Kitty graphics protocol allows displaying images in high resolution in broot.

Most terminals don't support it, so support must be verified.

Doing this with CSI escape sequences is a solution, but it involve delays and should only be enabled when this support can't be determined with [environment variables](https://dystroy.org/broot/launch/#environment-variables).

Enabling this feature is thus not recommended unless you use a terminal you know support this protocol and isn't recognized by broot. If this happen, please tell me so that we can update one of the fast checks.

