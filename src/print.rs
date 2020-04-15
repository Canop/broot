//! fonctions printing a tree or a path

use {
    crate::{
        app::{
            AppContext,
            AppStateCmdResult,
        },
        display::{
            DisplayableTree,
            Screen,
        },
        errors::ProgramError,
        flat_tree::Tree,
        launchable::Launchable,
        skin::Skin,
    },
    pathdiff,
    std::{
        fs::OpenOptions,
        io::{self, Write},
        path::Path,
    },
};

pub fn print_path(path: &Path, con: &AppContext) -> io::Result<AppStateCmdResult> {
    let path = path.to_string_lossy().to_string();
    Ok(
        if let Some(ref output_path) = con.launch_args.file_export_path {
            // an output path was provided, we write to it
            let f = OpenOptions::new()
                .create(true)
                .append(true)
                .open(output_path)?;
            writeln!(&f, "{}", path)?;
            AppStateCmdResult::Quit
        } else {
            // no output path provided. We write on stdout, but we must
            // do it after app closing to have the normal terminal
            AppStateCmdResult::from(Launchable::printer(path))
        },
    )
}

pub fn print_relative_path(path: &Path, con: &AppContext) -> io::Result<AppStateCmdResult> {
    let relative_path = match pathdiff::diff_paths(path, &con.launch_args.root) {
        None => return Ok(AppStateCmdResult::DisplayError(
            format!("Cannot relativize {:?}", path) // does this happen ? how ?
        )),
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
    screen: &mut Screen,
    file_path: &str,
) -> Result<AppStateCmdResult, ProgramError> {
    let no_style_skin = Skin::no_term();
    let dp = DisplayableTree::out_of_app(tree, &no_style_skin, screen.width);
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)?;
    dp.write_on(&mut f)?;
    Ok(AppStateCmdResult::Quit)
}

pub fn print_tree(
    tree: &Tree,
    screen: &mut Screen,
    con: &AppContext,
) -> Result<AppStateCmdResult, ProgramError> {
    if let Some(ref output_path) = con.launch_args.file_export_path {
        // an output path was provided, we write to it
        print_tree_to_file(tree, screen, output_path)
    } else {
        // no output path provided. We write on stdout, but we must
        // do it after app closing to have the normal terminal
        Ok(AppStateCmdResult::from(Launchable::tree_printer(
            tree,
            screen,
            con.launch_args.no_style,
        )))
    }
}
