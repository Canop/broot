use {
    crate::{
        app::AppContext,
        display::{
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
    crokey::crossterm::{
        QueueableCommand,
        cursor,
        event::{
            DisableMouseCapture,
            EnableMouseCapture,
        },
        terminal::{
            self,
            EnterAlternateScreen,
            LeaveAlternateScreen,
        },
    },
    opener,
    std::{
        env,
        io::{
            self,
            Write,
        },
        path::PathBuf,
        path::Path,
        process::Command,
        process::Stdio,
    },
    which::which,
};

/// Description of a task to execute on end of broot, like printing something on stdout, or
/// executing an external program.
///
/// A launchable can only be executed on end of life of broot, and after the normal terminal has
/// been restored, so it can do things that are not possible while broot is running, like printing
/// on stdout or executing a program that needs the terminal.
#[derive(Debug)]
pub enum Launchable {
    /// just print something on stdout on end of broot
    ///
    /// No newline is added, so the string should end with a newline if needed.
    Printer { to_print: String },

    /// print the tree on end of broot
    TreePrinter {
        tree: Box<Tree>,
        skin: Box<StyleMap>,
        ext_colors: ExtColorMap,
        width: u16,
        height: u16,
    },

    /// execute an external program
    Program {
        exe: String,
        args: Vec<String>,
        working_dir: Option<PathBuf>,
        switch_terminal: bool,
        capture_mouse: bool,
        keyboard_enhanced: bool,
    },

    /// open a path
    SystemOpen { path: PathBuf },
}

/// If a part starts with a '$', replace it by the environment variable of the same name.
/// This part is split too (because of <https://github.com/Canop/broot/issues/114>)
fn resolve_env_variables(parts: Vec<String>) -> Vec<String> {
    let mut resolved = Vec::new();
    for part in parts {
        if let Some(var_name) = part.strip_prefix('$') {
            if let Ok(val) = env::var(var_name) {
                resolved.extend(val.split(' ').map(ToString::to_string));
                continue;
            }
            if var_name == "EDITOR" {
                debug!("Env var $EDITOR not set, looking at editor command for fallback");
                if let Ok(editor) = which("editor") {
                    if let Some(editor) = editor.to_str() {
                        debug!("Using editor solved as {editor:?}");
                        resolved.push(editor.to_string());
                        continue;
                    }
                }
            }
        }
        resolved.push(part);
    }
    resolved
}

fn detach_standard_streams(command: &mut Command) {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(unix)]
    fn detached_background_program_with_stdin_reader_does_not_hang() {
        let mut command = Command::new("cat");
        detach_standard_streams(&mut command);
        let status = command
            .spawn()
            .and_then(|mut child| child.wait())
            .expect("cat should spawn");
        assert!(status.success());
    }

    #[test]
    #[cfg(unix)]
    fn detached_background_program_writing_stdout_succeeds() {
        let mut command = Command::new("echo");
        command.arg("broot");
        detach_standard_streams(&mut command);
        let status = command
            .spawn()
            .and_then(|mut child| child.wait())
            .expect("echo should spawn");
        assert!(status.success());
    }
}

impl Launchable {
    /// Create a launchable to open the given path with the system default application.
    pub fn opener(path: PathBuf) -> Launchable {
        Launchable::SystemOpen { path }
    }
    /// Create a launchable to print the given string on end of broot, without adding a newline.
    pub fn printer(to_print: String) -> Launchable {
        Launchable::Printer { to_print }
    }
    /// Create a launchable to print the given tree on end of broot, with the given skin and
    /// colors.
    pub fn tree_printer(
        tree: &Tree,
        screen: Screen,
        style_map: StyleMap,
        ext_colors: ExtColorMap,
    ) -> Launchable {
        Launchable::TreePrinter {
            tree: Box::new(tree.clone()),
            skin: Box::new(style_map),
            ext_colors,
            width: screen.width,
            height: (tree.lines.len() as u16).min(screen.height - 1),
        }
    }
    /// Create a launchable to execute the given program.
    ///
    /// Parts is a list of strings, the first one being the executable, and the others being the
    /// arguments.
    pub fn program(
        parts: Vec<String>,
        working_dir: Option<PathBuf>,
        switch_terminal: bool,
        con: &AppContext,
    ) -> io::Result<Launchable> {
        let mut parts = resolve_env_variables(parts).into_iter();
        match parts.next() {
            Some(exe) => Ok(Launchable::Program {
                exe,
                args: parts.collect(),
                working_dir,
                switch_terminal,
                capture_mouse: con.capture_mouse,
                keyboard_enhanced: con.keyboard_enhanced,
            }),
            None => Err(io::Error::other("Empty launch string")),
        }
    }

    /// Execute the launchable, writing on the given writer if needed (for the tree printer).
    pub fn execute(
        &self,
        mut w: Option<&mut W>,
    ) -> Result<(), ProgramError> {
        match self {
            Launchable::Printer { to_print } => {
                print!("{to_print}");
                Ok(())
            }
            Launchable::TreePrinter {
                tree,
                skin,
                ext_colors,
                width,
                height,
            } => {
                let dp = DisplayableTree::out_of_app(tree, skin, ext_colors, *width, *height);
                dp.write_on(&mut std::io::stdout())
            }
            Launchable::Program {
                working_dir,
                switch_terminal,
                exe,
                args,
                capture_mouse,
                keyboard_enhanced,
            } => {
                debug!("working_dir: {working_dir:?}");
                debug!("switch_terminal: {switch_terminal:?}");
                if *switch_terminal {
                    // we restore the normal terminal in case the executable
                    // is a terminal application, and we'll switch back to
                    // broot's alternate terminal when we're back to broot
                    if let Some(ref mut w) = &mut w {
                        if *keyboard_enhanced {
                            crokey::pop_keyboard_enhancement_flags()?;
                        }
                        w.queue(cursor::Show)?;
                        w.queue(LeaveAlternateScreen)?;
                        if *capture_mouse {
                            w.queue(DisableMouseCapture)?;
                        }
                        terminal::disable_raw_mode()?;
                        w.flush()?;
                    }
                }
                let mut old_working_dir = None;
                if let Some(working_dir) = working_dir {
                    old_working_dir = std::env::current_dir().ok();
                    if !try_set_current_dir(working_dir) {
                        warn!("Unable to set working dir to {working_dir:?}");
                        old_working_dir = None;
                    }
                }
                let mut command = Command::new(exe);
                if !*switch_terminal {
                    detach_standard_streams(&mut command);
                }
                let exec_res = command
                    .args(args.iter())
                    .spawn()
                    .and_then(|mut p| p.wait())
                    .map_err(|source| ProgramError::LaunchError {
                        program: exe.clone(),
                        source,
                    });
                if *switch_terminal {
                    if let Some(ref mut w) = &mut w {
                        terminal::enable_raw_mode()?;
                        if *capture_mouse {
                            w.queue(EnableMouseCapture)?;
                        }
                        w.queue(EnterAlternateScreen)?;
                        w.queue(cursor::Hide)?;
                        w.flush()?;
                        if *keyboard_enhanced {
                            crokey::push_keyboard_enhancement_flags()?;
                        }
                    }
                }
                if let Some(old_working_dir) = old_working_dir {
                    if !try_set_current_dir(&old_working_dir) {
                        warn!("Unable to restore working dir to {old_working_dir:?}");
                    }
                }
                exec_res?; // we trigger the error display after restoration
                Ok(())
            }
            Launchable::SystemOpen { path } => {
                opener::open(path)?;
                Ok(())
            }
        }
    }
}

/// Try set the current dir to the given path, and if it fails, try to climb the path until an
/// existing folder is found. Return true if the current dir has been changed, false otherwise.
pub fn try_set_current_dir(mut dir: &Path) -> bool {
    loop {
        if std::env::set_current_dir(dir).is_ok() {
            debug!("Working dir set to {dir:?}");
            return true;
        }
        let Some(parent_dir) = dir.parent() else {
            return false;
        };
        dir = parent_dir;
    }
}
