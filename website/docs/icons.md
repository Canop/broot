# Icons Overview

Broot supports showing icons in terminal if a supported font is installed. 
Currently, two fonts are supported, `nerdfonts` and `vscode`. 

Here is a screenshot, with nerdfont to the left and vscode at right:

![Broot icon comparison](img/20240225-icon-comparison.png)

# Configuration 

First add the appropriate lines to your broot config.

```hjson
icon_theme: vscode
```
```toml
icon_theme = "vscode"
```

or

```hjson
icon_theme: nerdfont
```
```toml
icon_theme = "nerdfont"
```

If the appropriate fonts are already installed correctly on your system then new instances of broot will show icons correctly. 

# Installation and checking for presence

## Checking the font

### VSCode

#### Bash (and compatible):
```bash
echo -e "file_type_rust looks like \U001002D2"
```

#### powershell

```powershell
echo "Rust is `u{1002D2}"
```
If you see a rust gear icon, your terminal is displaying the correct font.

### Nerdfont

#### Bash (and compatible):
```bash
echo -e "file_type_rust looks like \ue7a8"
```

This should display a rust icon

Unless you're really unlucky (there is some font that has the rust gear icon at the exact unicode point as the query but doesn't have other glyphs), you probably have the font installed correctly. If not, move to the installation section. 


## Installation

### VSCode

If the font isn't installed, you may

* take it from `/resources/icons/vscode/vscode.ttf` if you have broot sources
* download it from [https://github.com/Canop/broot/blob/main/resources/icons/vscode/vscode.ttf](https://github.com/Canop/broot/blob/main/resources/icons/vscode/vscode.ttf),
* or take it from the release archive if you installed broot from its zipped archive.

#### Installation on linux:

Copy the `vscode.ttf` file to `~/.local/share/fonts`.

#### Installation on Windows

Double click  the `vscode.ttf` file icon and click on "Install font".


### Nerdfonts

1. In order for the nerdfont setting to work you need to have a <a href="https://github.com/ryanoasis/nerd-fonts" target="_blank">patched nerdfont
</a> installed and set as your font in the terminal emulator of your choice.
2. After a successful installation, you need to add or uncomment the `icon_theme: "nerdfont"` line  in broot's config file
3. You should now be able to see icons when opening broot in your terminal.



## FAQ

**Q:** I don't see icons for my favourite common file type.

**A:** There could be three reasons why you don't see your file type icon

1. Check if your font was setup correctly. See [Installation](icons.md#minstallation) and #checking the font
2. The iconset is limited by available nerdfont icons. Some icons simply don't exist as of yet.
    Eg. a vite or prettier icon. You can find available icons in the nerdfont [cheat-sheet](https://www.nerdfonts.com/cheat-sheet)
3. If both of those are true and you think you found an icon we mapped wrong or is missing, we welcome your contribution! Send us a message in the [Miaou chat](https://miaou.dystroy.org/3490?broot) or create a pull request

**Q:** Why does broot show a generic icon for this very common file type?

**A:** The icon mappings aren't complete. You can help out very easily without any coding knowledge. Go to the github [repository](https://github.com/Canop/broot/tree/main/resources/icons). Enter the directory corresponding to your theme. Inside `data`, edit `extension_to_icon_name_map.rs` and add a line corresponding to your extension. The first field would be the extensions you would like, and the second field should be referred from `icon_name_to_icon_code_point_map.rs`. Submit a PR.


**Q:** Can I set up a totally different set of icons or mappings ?

**A:** Broot can be configured for a different mapping or a different font, but this needs some coding and a new compilation.
To get started, have a look at look at the resources/icons/nerdfont directory and, if necessary, come and chat on miaou.


**Q:** Why isn't the icon mapping configurable?

**A:** For performance reasons, icon mapping is hardcoded. If this looks like a problem, please come and chat with us.


**Q:** How do I remap in nerdfont? 

**A:** If you want to map or remap icons, please go to <a href="https://www.nerdfonts.com/cheat-sheet" target="_blank">nerdfont-cheat-sheet</a> and search for an icon you would like to set in its place.
In order to correctly fix the icon mapping you need a FILE_EXTENSION and a NERDFONT_ICON_CODE. For this example we are remapping the json file extension.

The first thing you need to do is find the "json" icon you want to map to in the cheatsheet and copy its iconCode ("the code inside the red box in the screenshot below").
Once done, find the corresponding file mapping in resources/icons/nerdfont by searching for your filetype in icon_name_to_icon_code_point_map:
In this case its this line:
```
( "file_type_json", 0xe60b ),
```
As you can see: the icon is mapped by "0x" followed by your iconCode. After changing this you should be able to recompile broot and see your new icon. If this is a new file mapping or a missing icon. Please consider contributing!


![nerdfont cheatsheet iconCode](img/20240225-nerdfont-cheatsheet.png)


