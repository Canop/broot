

## General form

The input is the area at the bottom of the focused panel, in which you can type a filter or a command.

Its parts are

* a filtering pattern
* a verb invocation, starting with a space or a colon (`:`)

Both parts are optional.

## The filtering pattern

The search syntax is globally

    <mode><pattern>[/<flags>]

The mode is either nothing (fuzzy name), just a slash (regex name) or some letters followed by a slash.

mode | example query | example match | explanation
-|-|-|-
fuzzy name | `abc` | `abac.txt` | search for "abc" in a fuzzy way in filenames
regex name | `/abc` | `abc.txt` | search for the regular expression `abc` in filenames ("exact search")
regex name | `/[yz]{3}` | `fuzzy.rs` | search for the regular expression `[yz]{3}` in filenames
regex name | `/abc/i` | `aBc.txt` | search for the regular expression `abc` with flag `i` in filenames
fuzzy path | `p/abc` | `a/bac.txt` |  search for "abc" in a fuzzy way in sub-paths from current tree root
regex path | `rp/abc` | `e/abac.txt` |  search for the "abc" regex  in sub-paths from current tree root
content | `c/mask` | `umask = "1.0"` | search for the "umask" string in file contents

It's also possible to [redefine those mode mappings](../conf_file/#search-modes).

To escape characters (for example the space, colon or slash) in the pattern, use a `\` (an antislash is `\\`).

## The verb invocation

The verb invocation is

    :<verb><arguments>

or

    <space><verb><arguments>

where arguments can be empty, depending on the verb's behaviour and invocation pattern.

Verbs are detailled in the [Verbs & Commands](verbs.md) chapter.

## Examples

### A Fuzzy search:

`re`

![fuzzy](img/20200526-input-fuzzy.png)

### A regular expression based search:

`/re`

![fuzzy](img/20200604-input-regex.png)

### A fuzzy path search

`p/coco`

![fuzzy](img/20200604-fuzzy-path.png)

### A search followed by a command without arguments:

`re rm` (which is equivalent to `re:rm`)

This is very natural: You use the search to select your element and you don't need to remove it before typing the command.

![fuzzy](img/20200526-input-fuzzy-rm.png)

### A search followed by a command taking an argument:

`re mv ../regex.rs`

![fuzzy](img/20200526-input-fuzzy-mv.png)

### A full text search

In this case with an escaped space:

`c/two\ p`

![content search](img/20200608-content-search.png)

