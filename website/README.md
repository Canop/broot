
Broot's website is live at https://dystroy.org/broot

It's built using [mkdocs](https://www.mkdocs.org/)

The current version of the site uses mkdocs version 1.0.4 and doesn't properly render on versions 1.1.x (this should be changed)

To test it locally, cd to the website directory then

	mkdocs serve

To build it, do

	mkdocs build

The broot_theme theme is taken from mkdocs standard theme "mkdocs" from mkdocs version 1.0.4 then adapted.

The reason I'm not using anymore the theme by name is that the theme changes with mkdocs version changes.

So in order to keep a constant theme between versions of mkdocs, I had to extract it.
