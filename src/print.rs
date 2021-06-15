//! fonctions printing a tree or a path

use {
    crate::{
        app::*,
        display::{DisplayableTree, Screen},
        errors::ProgramError,
        launchable::Launchable,
        skin::{ExtColorMap, PanelSkin, StyleMap},
        tree::Tree,
    },
    crossterm::tty::IsTty,
    pathdiff,
    std::{
        fs::OpenOptions,
        io::{self, Write, stdout},
        path::Path,
    },
};

fn print_string(string: String, con: &AppContext) -> io::Result<CmdResult> {
    Ok(
        if let Some(ref output_path) = con.launch_args.file_export_path {
            // an output path was provided, we write to it
            let f = OpenOptions::new()
                .create(true)
                .append(true)
                .open(output_path)?;
            writeln!(&f, "{}", string)?;
            CmdResult::Quit
        } else {
            // no output path provided. We write on stdout, but we must
            // do it after app closing to have the desired stdout (it may
            // be the normal terminal or a file, or other output)
            CmdResult::from(Launchable::printer(string))
        }
    )
}

pub fn print_paths(sel_info: &SelInfo, con: &AppContext) -> io::Result<CmdResult> {
    let string = match sel_info {
        SelInfo::None => "".to_string(), // better idea ?
        SelInfo::One(sel) => sel.path.to_string_lossy().to_string(),
        SelInfo::More(stage) => {
            let mut string = String::new();
            for path in stage.paths().iter() {
                string.push_str(&path.to_string_lossy());
                string.push('\n');
            }
            string
        }
    };
    print_string(string, con)
}

fn relativize_path(path: &Path, con: &AppContext) -> io::Result<String> {
    let relative_path = match pathdiff::diff_paths(path, &con.launch_args.root) {
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Cannot relativize {:?}", path), // does this happen ? how ?
            ));
        }
        Some(p) => p,
    };
    Ok(
        if relative_path.components().next().is_some() {
            relative_path.to_string_lossy().to_string()
        } else {
            ".".to_string()
        }
    )
}

pub fn print_relative_paths(sel_info: &SelInfo, con: &AppContext) -> io::Result<CmdResult> {
    let string = match sel_info {
        SelInfo::None => "".to_string(),
        SelInfo::One(sel) => relativize_path(sel.path, con)?,
        SelInfo::More(stage) => {
            let mut string = String::new();
            for path in stage.paths().iter() {
                string.push_str(&relativize_path(path, con)?);
                string.push('\n');
            }
            string
        }
    };
    print_string(string, con)
}

fn print_tree_to_file(
    tree: &Tree,
    screen: Screen,
    file_path: &str,
    ext_colors: &ExtColorMap,
) -> Result<CmdResult, ProgramError> {
    let no_style_skin = StyleMap::no_term();
    let dp = DisplayableTree::out_of_app(
        tree,
        &no_style_skin,
        ext_colors,
        screen.width,
        (tree.lines.len() as u16).min(screen.height),
    );
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)?;
    dp.write_on(&mut f)?;
    Ok(CmdResult::Quit)
}

pub fn print_tree(
    tree: &Tree,
    screen: Screen,
    panel_skin: &PanelSkin,
    con: &AppContext,
) -> Result<CmdResult, ProgramError> {
    if let Some(ref output_path) = con.launch_args.file_export_path {
        // an output path was provided, we write to it
        print_tree_to_file(tree, screen, output_path, &con.ext_colors)
    } else {
        // no output path provided. We write on stdout, but we must
        // do it after app closing to have the normal terminal
        let show_color = con.launch_args
            .color
            .unwrap_or_else(|| stdout().is_tty());
        let styles = if show_color {
            panel_skin.styles.clone()
        } else {
            StyleMap::no_term()
        };
        Ok(CmdResult::from(Launchable::tree_printer(
            tree,
            screen,
            styles,
            con.ext_colors.clone(),
        )))
    }
}
