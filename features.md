

This page defines the optional features which may be applied on compilation:

* clipboard

Feature gating is usually temporary: they may be removed when a technical problem is solved, when a feature becomes "mainstream", or when it's dropped because no user mentionned using it.

## The "clipboard" feature

This feature allows the `:copy_path` verb which copies the currently selected path into the clipboard.

Limits:

- the feature doesn't compile right now on some platforms (for example Raspberry)
- on some platforms, such as X11 on Linux, \*BSD, etc the content leaves the clipboard when you quit broot, so you must paste while broot is still running. In the case of X11, this is a limitation of the platform and not something that can ever be handled on an application leve.
