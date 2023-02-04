//! functions printing a tree or a path

use {
    crate::{
        app::*,
        display::Screen,
        errors::ProgramError,
        launchable::Launchable,
        skin::{PanelSkin, StyleMap},
        tree::Tree,
    },
    crokey::crossterm::tty::IsTty,
    pathdiff,
    std::{
        io::{self, stdout},
        path::Path,
    },
};

fn print_string(string: String, _con: &AppContext) -> io::Result<CmdResult> {
    Ok(
        // We write on stdout, but we must do it after app closing
        // to have the desired stdout (it may be the normal terminal
        // or a file, or other output)
        CmdResult::from(Launchable::printer(string))
    )
}

pub fn print_paths(sel_info: SelInfo, con: &AppContext) -> io::Result<CmdResult> {
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
    let relative_path = match pathdiff::diff_paths(path, &con.initial_root) {
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Cannot relativize {path:?}"), // does this happen ? how ?
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

pub fn print_relative_paths(sel_info: SelInfo, con: &AppContext) -> io::Result<CmdResult> {
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

pub fn print_tree(
    tree: &Tree,
    screen: Screen,
    panel_skin: &PanelSkin,
    con: &AppContext,
) -> Result<CmdResult, ProgramError> {
    // We write on stdout, but we must do it after app closing to have the normal terminal
    let show_color = con.launch_args.color.unwrap_or_else(|| stdout().is_tty());
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
