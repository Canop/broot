use {
    crate::{
        display::{DisplayableTree, Screen},
        errors::ProgramError,
        skin::StyleMap,
        tree::Tree,
    },
    open,
    std::{env, io, path::PathBuf, process::Command},
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
        skin: Box<StyleMap>,
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
        screen: &Screen,
        style_map: StyleMap,
    ) -> Launchable {
        Launchable::TreePrinter {
            tree: Box::new(tree.clone()),
            skin: Box::new(style_map),
            width: screen.width,
        }
    }

    pub fn program(parts: Vec<String>) -> io::Result<Launchable> {
        let mut parts = resolve_env_variables(parts).into_iter();
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
            Launchable::SystemOpen { path } => {
                open::that(&path)?;
                Ok(())
            }
        }
    }
}
