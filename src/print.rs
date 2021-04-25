//! fonctions printing a tree or a path

use {
    crate::{
        app::{AppContext, CmdResult},
        display::{DisplayableTree, Screen},
        errors::ProgramError,
        launchable::Launchable,
        skin::{ExtColorMap, PanelSkin, StyleMap},
        tree::Tree,
    },
    pathdiff,
    std::{
        fs::OpenOptions,
        io::{self, Write},
        path::Path,
    },
};

pub fn print_path(path: &Path, con: &AppContext) -> io::Result<CmdResult> {
    let path = path.to_string_lossy().to_string();
    Ok(
        if let Some(ref output_path) = con.launch_args.file_export_path {
            // an output path was provided, we write to it
            let f = OpenOptions::new()
                .create(true)
                .append(true)
                .open(output_path)?;
            writeln!(&f, "{}", path)?;
            CmdResult::Quit
        } else {
            // no output path provided. We write on stdout, but we must
            // do it after app closing to have the normal terminal
            CmdResult::from(Launchable::printer(path))
        },
    )
}

pub fn print_relative_path(path: &Path, con: &AppContext) -> io::Result<CmdResult> {
    let relative_path = match pathdiff::diff_paths(path, &con.launch_args.root) {
        None => {
            return Ok(CmdResult::DisplayError(
                format!("Cannot relativize {:?}", path), // does this happen ? how ?
            ));
        }
        Some(p) => p,
    };
    if relative_path.components().next().is_some() {
        print_path(&relative_path, con)
    } else {
        print_path(Path::new("."), con)
    }
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
        let styles = if con.launch_args.no_style {
            StyleMap::no_term()
        } else {
            panel_skin.styles.clone()
        };
        Ok(CmdResult::from(Launchable::tree_printer(
            tree,
            screen,
            styles,
            con.ext_colors.clone(),
        )))
    }
}
