use {
    crate::{
        display::{
            Cols,
            DisplayableTree,
            Screen,
            W,
        },
        errors::ProgramError,
        skin::{
            ExtColorMap,
            StyleMap,
        },
        tree::Tree,
    },
    crossterm::{
        cursor,
        terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
        QueueableCommand,
    },
    open,
    std::{
        env,
        io::{self, Write},
        path::PathBuf,
        process::Command,
    },
};

/// description of a possible launch of an external program
/// A launchable can only be executed on end of life of broot.
#[derive(Debug)]
pub enum Launchable {

    /// just print something on stdout on end of broot
    Printer {
        to_print: String,
    },

    /// print the tree on end of broot
    TreePrinter {
        tree: Box<Tree>,
        skin: Box<StyleMap>,
        cols: Cols,
        ext_colors: ExtColorMap,
        width: u16,
    },

    /// execute an external program
    Program {
        exe: String,
        args: Vec<String>,
        working_dir: Option<PathBuf>,
    },

    /// open a path
    SystemOpen {
        path: PathBuf,
    },
}

/// If a part starts with a '$', replace it by the environment variable of the same name.
/// This part is splitted too (because of https://github.com/Canop/broot/issues/114)
fn resolve_env_variables(parts: Vec<String>) -> Vec<String> {
    let mut resolved = Vec::new();
    for part in parts.into_iter() {
        if part.starts_with('$') {
            if let Ok(val) = env::var(&part[1..]) {
                resolved.extend(val.split(' ').map(|s| s.to_string()));
                continue;
            }
        }
        resolved.push(part);
    }
    resolved
}

impl Launchable {
    pub fn opener(path: PathBuf) -> Launchable {
        Launchable::SystemOpen { path }
    }
    pub fn printer(to_print: String) -> Launchable {
        Launchable::Printer { to_print }
    }
    pub fn tree_printer(
        tree: &Tree,
        screen: Screen,
        style_map: StyleMap,
        cols: Cols,
        ext_colors: ExtColorMap,
    ) -> Launchable {
        Launchable::TreePrinter {
            tree: Box::new(tree.clone()),
            skin: Box::new(style_map),
            cols,
            ext_colors,
            width: screen.width,
        }
    }

    pub fn program(
        parts: Vec<String>,
        working_dir: Option<PathBuf>,
    ) -> io::Result<Launchable> {
        let mut parts = resolve_env_variables(parts).into_iter();
        match parts.next() {
            Some(exe) => Ok(Launchable::Program {
                exe,
                args: parts.collect(),
                working_dir,
            }),
            None => Err(io::Error::new(io::ErrorKind::Other, "Empty launch string")),
        }
    }

    pub fn execute(&self, mut w: Option<&mut W>) -> Result<(), ProgramError> {
        match self {
            Launchable::Printer { to_print } => {
                println!("{}", to_print);
                Ok(())
            }
            Launchable::TreePrinter { tree, skin, cols, ext_colors, width } => {
                let dp = DisplayableTree::out_of_app(&tree, &skin, &cols, &ext_colors, *width);
                dp.write_on(&mut std::io::stdout())
            }
            Launchable::Program { working_dir, exe, args } => {
                // we restore the normal terminal in case the executable
                // is a terminal application, and we'll switch back to
                // broot's alternate terminal when we're back to broot
                // (and this part of the code should be cleaned...)
                if let Some(ref mut w) = &mut w {
                    w.queue(cursor::Show).unwrap();
                    w.queue(cursor::EnableBlinking).unwrap();
                    w.queue(LeaveAlternateScreen).unwrap();
                    terminal::disable_raw_mode().unwrap();
                    w.flush().unwrap();
                }
                let mut old_working_dir = None;
                if let Some(working_dir) = working_dir {
                    old_working_dir = std::env::current_dir().ok();
                    std::env::set_current_dir(working_dir).unwrap();
                }
                Command::new(&exe)
                    .args(args.iter())
                    .spawn()
                    .and_then(|mut p| p.wait())
                    .map_err(|source| ProgramError::LaunchError {
                        program: exe.clone(),
                        source,
                    })?;
                if let Some(ref mut w) = &mut w {
                    terminal::enable_raw_mode().unwrap();
                    w.queue(EnterAlternateScreen).unwrap();
                    w.queue(cursor::DisableBlinking).unwrap();
                    w.queue(cursor::Hide).unwrap();
                    w.flush().unwrap();
                }
                if let Some(old_working_dir) = old_working_dir {
                    std::env::set_current_dir(old_working_dir).unwrap();
                }
                Ok(())
            }
            Launchable::SystemOpen { path } => {
                open::that(&path)?;
                Ok(())
            }
        }
    }
}
