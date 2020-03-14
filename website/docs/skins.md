[reference on skin configuration](../documentation/configuration/#colors)

# White Background Skin

This skin is suitable if you're used to black on white:

```toml
[skin]
default = "gray(2) gray(23)"
tree = "gray(17) none"
file = "gray(1) none"
directory = "ansi(20) none bold"
exe = "ansi(178) none"
link = "Magenta none"
pruning = "gray(5) none Italic"
perm__ = "gray(19) None"
perm_r = "ansi(92) None"
perm_w = "ansi(192) None"
perm_x = "ansi(69) None"
selected_line = "none gray(20)"
char_match = "ansi(28) none"
file_error = "Red none"
flag_label = "gray(16) none"
flag_value = "ansi(202) none bold"
input = "ansi(0) none"
status_error = "ansi(196) gray(22)"
status_job = "ansi(220) gray(18)"
status_normal = "gray(2) gray(22)"
status_italic = "ansi(202) gray(22)"
status_bold = "ansi(202) gray(22) bold"
status_code = "ansi(17) gray(22)"
status_ellipsis = "gray(1) white"
scrollbar_track = "gray(20) none"
scrollbar_thumb = "ansi(238) none"
help_paragraph = "gray(2) none"
help_bold = "ansi(202) none bold"
help_italic = "ansi(202) none italic"
help_code = "gray(5) gray(22)"
help_headers = "ansi(202) none"
```

![light skin](../img/20191114-light-skin.png)

# Solarized Dark

*contributed by [@danieltrautmann](https://github.com/danieltrautmann)*

The Solarized Dark skin uses RGB values, so it might not work well with some
terminals. It was tested with iTerm2 3.3 on macOS Catalina with the reported
terminal type `xterm-256color`. This can be set via:

Profiles > Your Profile > Terminal > Reported terminal type

The comment next to each setting tells the name of the color from the Solarized
Dark color scheme. The values are taken from
[here](https://github.com/altercation/solarized#the-values). The term `default`
in the comment refers to the skins default setting.

```toml
[skin]
default = "rgb(131, 148, 150) rgb(0, 43, 54)"          # base0 base03
tree = "rgb(88, 110, 117) none"                        # base01 default
file = "none none"                                     # default default
directory = "rgb(38, 139, 210) none bold"              # blue default bold
exe = "rgb(211, 1, 2) none"                            # red default
link = "rgb(211, 54, 130) none"                        # magenta default
pruning = "rgb(88, 110, 117) none italic"              # base01 default italic
perm__ = "rgb(88, 110, 117) none"                      # base01 default
perm_r = "none none"                                   # default default
perm_w = "none none"                                   # default default
perm_x = "none none"                                   # default default
owner = "rgb(88, 110, 117) none"                       # base01 default
group = "rgb(88, 110, 117) none"                       # base01 default
selected_line = "none rgb(7, 54, 66)"                  # default base02
char_match = "rgb(133, 153, 0) none underlined"        # green default underlined
file_error = "rgb(203, 75, 22) none italic"            # orange default italic
flag_label = "none none"                               # default default
flag_value = "rgb(181, 137, 0) none bold"              # yellow default bold
input = "none none"                                    # default default
status_error = "rgb(203, 75, 22) rgb(7, 54, 66)"       # orange base02
status_job = "rgb(108, 113, 196) rgb(7, 54, 66) bold"  # violet base02 bold
status_normal = "none rgb(7, 54, 66)"                  # default base02
status_italic = "rgb(181, 137, 0) rgb(7, 54, 66)"      # yellow base02
status_bold = "rgb(147, 161, 161) rgb(7, 54, 66) bold" # base1 base02 bold
status_code = "rgb(108, 113, 196) rgb(7, 54, 66)"      # violet base02
status_ellipsis = "none rgb(7, 54, 66)"                # default base02
scrollbar_track = "rgb(7, 54, 66) none"                # base02 default
scrollbar_thumb = "none none"                          # default default
help_paragraph = "none none"                           # default default
help_bold = "rgb(147, 161, 161) none bold"             # base1 default bold
help_italic = "rgb(147, 161, 161) none italic"         # base1 default italic
help_code = "rgb(147, 161, 161) rgb(7, 54, 66)"        # base1 base02
help_headers = "rgb(181, 137, 0) none"                 # yellow default
```

**Default View**

![default](img/skins/solarized_dark/default.png)

**Search**

![default](img/skins/solarized_dark/search.png)

**Permissions**

![default](img/skins/solarized_dark/perms.png)

**Sizes**

![default](img/skins/solarized_dark/sizes.png)

# Contribute your own skin

Don't hesitate to contact me on Miaou if you have a skin to propose or to discuss an existing one.

The white background one isn't very good IMO (I'm not used to such configuration mysel) so I'd be happy to replace it.
