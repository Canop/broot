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

fn join_lines<I>(lines: I) -> String
where
    I: IntoIterator<Item = String>,
{
    let mut iter = lines.into_iter();
    let Some(first) = iter.next() else {
        return String::new();
    };
    let mut string = first;
    for line in iter {
        string.push('\n');
        string.push_str(&line);
    }
    string
}

fn print_string(
    string: String,
    _con: &AppContext,
) -> io::Result<CmdResult> {
    Ok(
        // We write on stdout, but we must do it after app closing
        // to have the desired stdout (it may be the normal terminal
        // or a file, or other output)
        CmdResult::from(Launchable::printer(string)),
    )
}

pub fn print_paths(
    sel_info: SelInfo,
    con: &AppContext,
) -> io::Result<CmdResult> {
    let string = match sel_info {
        SelInfo::None => "".to_string(), // better idea ?
        SelInfo::One(sel) => sel.path.to_string_lossy().to_string(),
        SelInfo::More(stage) => join_lines(
            stage
                .paths()
                .iter()
                .map(|path| path.to_string_lossy().to_string()),
        ),
    };
    print_string(string, con)
}

fn relativize_path(
    path: &Path,
    con: &AppContext,
) -> io::Result<String> {
    let relative_path = match pathdiff::diff_paths(path, &con.initial_root) {
        None => {
            return Err(io::Error::other(format!("Cannot relativize {path:?}")));
        }
        Some(p) => p,
    };
    Ok(if relative_path.components().next().is_some() {
        relative_path.to_string_lossy().to_string()
    } else {
        ".".to_string()
    })
}

pub fn print_relative_paths(
    sel_info: SelInfo,
    con: &AppContext,
) -> io::Result<CmdResult> {
    let string = match sel_info {
        SelInfo::None => "".to_string(),
        SelInfo::One(sel) => relativize_path(sel.path, con)?,
        SelInfo::More(stage) => join_lines(
            stage
                .paths()
                .iter()
                .map(|path| relativize_path(path, con))
                .collect::<Result<Vec<_>, _>>()?,
        ),
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

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{launchable::Launchable, stage::Stage},
        std::path::PathBuf,
    };

    #[test]
    fn print_paths_does_not_add_an_empty_line_after_multiple_paths() {
        let mut stage = Stage::default();
        stage.add(PathBuf::from("/tmp/cog.txt"));
        stage.add(PathBuf::from("/tmp/dog.png"));

        let cmd = print_paths(SelInfo::More(&stage), &AppContext::default()).unwrap();

        let CmdResult::Launch(launchable) = cmd else {
            panic!("expected Launch result");
        };
        let Launchable::Printer { to_print } = launchable.as_ref() else {
            panic!("expected printer launchable");
        };

        assert_eq!(to_print, "/tmp/cog.txt\n/tmp/dog.png");
    }

    #[test]
    fn print_relative_paths_does_not_add_an_empty_line_after_multiple_paths() {
        let mut stage = Stage::default();
        stage.add(PathBuf::from("/tmp/broot-print/cog.txt"));
        stage.add(PathBuf::from("/tmp/broot-print/dog.png"));

        let mut con = AppContext::default();
        con.initial_root = PathBuf::from("/tmp/broot-print");

        let cmd = print_relative_paths(SelInfo::More(&stage), &con).unwrap();

        let CmdResult::Launch(launchable) = cmd else {
            panic!("expected Launch result");
        };
        let Launchable::Printer { to_print } = launchable.as_ref() else {
            panic!("expected printer launchable");
        };

        assert_eq!(to_print, "cog.txt\ndog.png");
    }
}
