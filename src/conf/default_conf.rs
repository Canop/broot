
/// the content a the conf.toml file broot creates
/// when there's none. It features some default
/// configuration and many completable
/// or uncommentable sections to help the user
/// define her own configuration.
pub const DEFAULT_CONF_FILE: &str = r#"
###############################################################
# This configuration file lets you
# - define new commands
# - change the shortcut or triggering keys of built-in verbs
# - change the colors
# - set default values for flags
# - set special behaviors on specific paths
#
# Configuration documentation is available at
#     https://dystroy.org/broot
###############################################################

###############################################################
# Default flags
# You can set up flags you want broot to start with by
# default, for example `default_flags="ihp"` if you usually want
# to see hidden and gitignored files and the permissions (then
# if you don't want the hidden files you can launch `br -H`)
# A popular flag is the `g` one which displays git related info.
#
default_flags = ""

###############################################################
# Special paths
# If some paths must be handled specially, uncomment (and change
# this section as per the examples
#
# [special-paths]
# "/media/slow-backup-disk" = "no-enter"
# "/home/dys/useless" = "hide"
# "/home/dys/my-link-I-want-to-explore" = "enter"

###############################################################
# Date/Time format
# If you want to change the format for date/time, uncomment the
# following line and change it according to
# https://docs.rs/chrono/0.4.11/chrono/format/strftime/index.html
#
# date_time_format = "%Y/%m/%d %R"

###############################################################
# Column order
# cols_order, if specified, must be a permutation of "gbpdscn"
# where every char denotes a column:
#  g : Git file info
#  b : branch (shows the depth and parent in the tree)
#  p : permissions (mode, user, group)
#  d : last modification date
#  s : size (with size bar when sorting)
#  c : count, number of files in directories
#  n : file name
#
# cols_order = "gbdscn"

###############################################################
# Verbs and shortcuts
# You can define your own commands which would be applied to
# the selection.
#
# Exemple 1: launching `tail -n` on the selected file (leaving broot)
# [[verbs]]
# name = "tail_lines"
# invocation = "tl {lines_count}"
# execution = "tail -f -n {lines_count} {file}"
#
# Exemple 2: creating a new file without leaving broot
# [[verbs]]
# name = "touch"
# invocation = "touch {new_file}"
# execution = "touch {directory}/{new_file}"
# leave_broot = false

# If $EDITOR isn't set on your computer, you should either set it using
#  something similar to
#   export EDITOR=/usr/bin/nvim
#  or just replace it with your editor of choice in the 'execution'
#  pattern.
# Example:
#  execution = "/usr/bin/nvim {file}"
[[verbs]]
invocation = "edit"
key = "F2"
shortcut = "e"
execution = "$EDITOR {file}"
leave_broot = false

[[verbs]]
invocation = "create {subpath}"
execution = "$EDITOR {directory}/{subpath}"
leave_broot = false

[[verbs]]
invocation = "git_diff"
shortcut = "gd"
leave_broot = false
execution = "git difftool -y {file}"

# If $PAGER isn't set on your computer, you should either set it
#  or just replace it with your viewer of choice in the 'execution'
#  pattern.
# Example:
#  execution = "less {file}"
[[verbs]]
name = "view"
invocation = "view"
execution = "$PAGER {file}"
leave_broot = false

# A popular set of shorctuts for going up and down:
#
# [[verbs]]
# key = "ctrl-j"
# execution = ":line_down"
#
# [[verbs]]
# key = "ctrl-k"
# execution = ":line_up"
#
# [[verbs]]
# key = "ctrl-d"
# execution = ":page_down"
#
# [[verbs]]
# key = "ctrl-u"
# execution = ":page_up"
# [[verbs]]
# key = "home"
# execution = ":select_first"
# [[verbs]]
# key = "end"
# execution = ":select_last"

# If you develop using git, you might like to often switch
# to the "git status" filter:
# [[verbs]]
# key = "ctrl-g"
# execution = ":toggle_git_status"

# You can reproduce the bindings of Norton Commander
# on copying or moving to the other panel:
#
# [[verbs]]
# key = "F5"
# execution = ":copy_to_panel"
#
# [[verbs]]
# key = "F6"
# execution = ":move_to_panel"


###############################################################
# Skin
# If you want to change the colors of broot,
# uncomment the following bloc and start messing
# with the various values.
#
# [skin]
# default = "gray(23) none / gray(20) none"
# tree = "ansi(94) None / gray(3) None"
# file = "gray(20) None / gray(15) None"
# directory = "ansi(208) None Bold / ansi(172) None bold"
# exe = "Cyan None"
# link = "Magenta None"
# pruning = "gray(12) None Italic"
# perm__ = "gray(5) None"
# perm_r = "ansi(94) None"
# perm_w = "ansi(132) None"
# perm_x = "ansi(65) None"
# owner = "ansi(138) None"
# group = "ansi(131) None"
# count = "ansi(136) gray(3)"
# dates = "ansi(66) None"
# sparse = "ansi(214) None"
# content_extract = "ansi(29) None"
# content_match = "ansi(34) None"
# git_branch = "ansi(229) None"
# git_insertions = "ansi(28) None"
# git_deletions = "ansi(160) None"
# git_status_current = "gray(5) None"
# git_status_modified = "ansi(28) None"
# git_status_new = "ansi(94) None Bold"
# git_status_ignored = "gray(17) None"
# git_status_conflicted = "ansi(88) None"
# git_status_other = "ansi(88) None"
# selected_line = "None gray(5) / None gray(4)"
# char_match = "Yellow None"
# file_error = "Red None"
# flag_label = "gray(15) None"
# flag_value = "ansi(208) None Bold"
# input = "White None / gray(15) gray(2)"
# status_error = "gray(22) ansi(124)"
# status_job = "ansi(220) gray(5)"
# status_normal = "gray(20) gray(3) / gray(2) gray(2)"
# status_italic = "ansi(208) gray(3) / gray(2) gray(2)"
# status_bold = "ansi(208) gray(3) Bold / gray(2) gray(2)"
# status_code = "ansi(229) gray(3) / gray(2) gray(2)"
# status_ellipsis = "gray(19) gray(1) / gray(2) gray(2)"
# purpose_normal = "gray(20) gray(2)"
# purpose_italic = "ansi(178) gray(2)"
# purpose_bold = "ansi(178) gray(2) Bold"
# purpose_ellipsis = "gray(20) gray(2)"
# scrollbar_track = "gray(7) None / gray(4) None"
# scrollbar_thumb = "gray(22) None / gray(14) None"
# help_paragraph = "gray(20) None"
# help_bold = "ansi(208) None Bold"
# help_italic = "ansi(166) None"
# help_code = "gray(21) gray(3)"
# help_headers = "ansi(208) None"
# help_table_border = "ansi(239) None"
# preview = "gray(20) gray(1) / gray(18) gray(2)"
# preview_line_number = "gray(12) gray(3)"
# preview_match = "None ansi(29)"
# hex_null = "gray(11) None"
# hex_ascii_graphic = "gray(18) None"
# hex_ascii_whitespace = "ansi(143) None"
# hex_ascii_other = "ansi(215) None"
# hex_non_ascii = "ansi(167) None"

# You may find explanations and other skins on
#  https://dystroy.org/broot/skins
# for example a skin suitable for white backgrounds


###############################################################
# File Extension Colors
#
# uncomment and modify the next section if you want to color
# file name depending on their extension
#
# [ext-colors]
# png = "rgb(255, 128, 75)"
# rs = "yellow"


"#;
