use std::{
    env,
    fs::OpenOptions,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};

use opener;
use regex::Regex;

use crate::{
    app::AppStateCmdResult, app_context::AppContext, displayable_tree::DisplayableTree,
    errors::ProgramError, flat_tree::Tree, screens::Screen, skin::Skin,
};

/// description of a possible launch of an external program
/// A launchable can only be executed on end of life of broot.
#[derive(Debug)]
pub enum Launchable {
    Printer {
        // just print something on stdout on end of broot
        to_print: String,
    },
    TreePrinter {
        // print the tree on end of broot
        tree: Box<Tree>,
        skin: Box<Skin>,
        width: u16,
    },
    Program {
        // execute an external program
        exe: String,
        args: Vec<String>,
    },
    SystemOpen {
        // open a path
        path: PathBuf,
    },
}

/// If s starts by a '$', replace it by the environment variable of the same name
fn resolve_env_variable(s: String) -> String {
    if s.starts_with('$') {
        env::var(&s[1..]).unwrap_or(s)
    } else {
        s
    }
}

impl Launchable {
    pub fn opener(path: PathBuf) -> Launchable {
        Launchable::SystemOpen { path }
    }
    pub fn printer(to_print: String) -> Launchable {
        Launchable::Printer { to_print }
    }
    pub fn tree_printer(tree: &Tree, screen: &Screen) -> Launchable {
        Launchable::TreePrinter {
            tree: Box::new(tree.clone()),
            skin: Box::new(screen.skin.clone()),
            width: screen.width,
        }
    }

    pub fn program(mut parts: Vec<String>) -> io::Result<Launchable> {
        let mut parts = parts.drain(0..).map(resolve_env_variable);
        match parts.next() {
            Some(exe) => Ok(Launchable::Program {
                exe,
                args: parts.collect(),
            }),
            None => Err(io::Error::new(io::ErrorKind::Other, "Empty launch string")),
        }
    }

    pub fn execute(&self) -> Result<(), ProgramError> {
        match self {
            Launchable::Printer { to_print } => {
                println!("{}", to_print);
                Ok(())
            }
            Launchable::TreePrinter { tree, skin, width } => {
                let dp = DisplayableTree::out_of_app(&tree, &skin, *width);
                dp.write_on(&mut std::io::stdout())
            }
            Launchable::Program { exe, args } => {
                Command::new(&exe)
                    .args(args.iter())
                    .spawn()
                    .and_then(|mut p| p.wait())
                    .map_err(|source| ProgramError::LaunchError {
                        program: exe.clone(),
                        source,
                    })?;
                Ok(())
            }
            Launchable::SystemOpen { path } => match opener::open(&path) {
                Ok(_) => Ok(()),
                Err(err) => Err(ProgramError::OpenError { err }),
            },
        }
    }
}

// from a path, build a string usable in a shell command, wrapping
//  it in quotes if necessary (and then escaping internal quotes).
// Don't do unnecessary transformation, so that the produced string
//  is prettier on screen.
pub fn escape_for_shell(path: &Path) -> String {
    let path = path.to_string_lossy();
    if regex!(r"^[\w/.-]*$").is_match(&path) {
        path.to_string()
    } else {
        format!("'{}'", &path.replace('\'', r"'\''"))
    }
}

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
            tree, screen,
        )))
    }
}
