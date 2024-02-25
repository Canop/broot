# Nerd Fonts Icons

## Requirements

[Nerd Fonts](https://github.com/ryanoasis/nerd-fonts) installed either through a patched font or available as a fallback font.

## Configuration

In broot config file, set

```
icon_theme: nerdfont
```

## Limitations

These icons are limited by availability of symbols in Nerd Fonts, so this feature can only support a subset of filetypes available in `vscode` theme.

## Editing the Icon for a File:
If you want to find an icon for a file: go to https://www.nerdfonts.com/cheat-sheet and search for:
- a icon name like "file", which should return the multiple file icon results. Pick one you like and copy the icon code "ea7b". Copy it into the corresponding mapping prefixed with "0x" in ./data/*.rs. ( "default_file", 0xf15b ), // 
- a icon code like "0xf15b" without the "0x" prefix. This should return the corresponding "" icon.


## Tips on editing these files in vi

1. Open ./icon_name_to_icon_code_point_map.rs
   then in the same session, switch to file you want to edit
   use C-n and C-y in edit mode

2. This plugin currently searches for lowercase, make everything so

3. Remember to run :Tabularize over ')' and ','. <a href="https://github.com/godlygeek/tabular?tab=readme-ov-file">The tabular Plugin</a>

4. :'<,'>!sort

5. `cargo run` in debug mode should figure out some problems.
