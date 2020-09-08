

broot defines two optional features which may be applied on compilation:

* client-server
* clipboard

## The "client-server" feature

This feature allows an instance of broot to be remotely controlled.

It's described in [client-server.md](client-server.md).

This feature could potentially be made standard if there's some demand.

## The "clipboard" feature

This feature allows the `:copy_path` verb which copies the currently selected path into the clipboard.

Limits:

- the feature doesn't compile right now on some platforms (for example Raspberry)
- on some platforms the content leaves the clipboard when you quit broot (so you must paste while broot is still running)
