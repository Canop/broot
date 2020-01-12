/// Verbs are the engines of broot commands, and apply
/// - to the selected file (if user-defined, then must contain {file}, {parent} or {directory})
/// - to the current app state
use {
    crate::{
        app_context::AppContext,
        app_state::AppStateCmdResult,
        errors::{ConfError, ProgramError},
        external,
        io::W,
        keys,
        screens::Screen,
        selection_type::SelectionType,
        status::Status,
        verb_invocation::VerbInvocation,
    },
    crossterm::event::{KeyCode, KeyEvent},
    minimad::Composite,
    regex::{self, Captures, Regex},
    std::{
        collections::HashMap,
        fs::OpenOptions,
        io::Write,
        path::{Path, PathBuf},
    },
};

/// what makes a verb.
///
/// There are two types of verbs executions:
/// - external programs or commands (cd, mkdir, user defined commands, etc.)
/// - built in behaviors (focusing a path, going back, showing the help, etc.)
///
#[derive(Debug, Clone)]
pub struct Verb {
    pub invocation: VerbInvocation, // how the verb is supposed to be called, may be empty
    pub key: Option<KeyEvent>,
    pub key_desc: String, // a description of the optional keyboard key triggering that verb
    pub args_parser: Option<Regex>,
    pub shortcut: Option<String>,    // a shortcut, eg "c"
    pub execution: String,           // a pattern usable for execution, eg ":quit" or "less {file}"
    pub description: Option<String>, // a description for the user
    pub from_shell: bool, // whether it must be launched from the parent shell (eg because it's a shell function)
    pub leave_broot: bool, // only defined for external
    pub confirm: bool,    // not yet used...
    pub selection_condition: SelectionType,
}

lazy_static! {
    static ref GROUP: Regex = Regex::new(r"\{([^{}:]+)(?::([^{}:]+))?\}").unwrap();
}

pub trait VerbExecutor {
    fn execute_verb(
        &mut self,
        verb: &Verb,
        invocation: &VerbInvocation,
        screen: &mut Screen,
        con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError>;
}

fn make_invocation_args_regex(spec: &str) -> Result<Regex, ConfError> {
    let spec = GROUP.replace_all(spec, r"(?P<$1>.+)");
    let spec = format!("^{}$", spec);
    Regex::new(&spec.to_string())
        .or_else(|_| Err(ConfError::InvalidVerbInvocation { invocation: spec }))
}

fn path_to_string(path: &Path, for_shell: bool) -> String {
    if for_shell {
        external::escape_for_shell(path)
    } else {
        path.to_string_lossy().to_string()
    }
}

impl Verb {
    /// build a verb using standard configurable behavior.
    /// "external" means not "built-in".
    pub fn create_external(
        invocation_str: &str,
        key: Option<KeyEvent>,
        shortcut: Option<String>,
        execution: String,
        description: Option<String>,
        from_shell: bool,
        leave_broot: bool,
        confirm: bool,
    ) -> Result<Verb, ConfError> {
        let invocation = VerbInvocation::from(invocation_str);
        let args_parser = invocation
            .args
            .as_ref()
            .map(|args| make_invocation_args_regex(&args))
            .transpose()?;
        // we use the selection condition to prevent configured
        // verb execution on enter on directories
        let selection_condition = match key {
            Some(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) => SelectionType::File,
            _ => SelectionType::Any,
        };
        Ok(Verb {
            invocation,
            key_desc: key.map_or("".to_string(), keys::key_event_desc),
            key,
            args_parser,
            shortcut,
            execution,
            description,
            from_shell,
            leave_broot,
            confirm,
            selection_condition,
        })
    }

    /// built-ins are verbs offering a logic other than the execution
    ///  based on exec_pattern. They mostly modify the appstate
    pub fn create_builtin(
        name: &str,
        key: Option<KeyEvent>,
        shortcut: Option<String>,
        description: &str,
    ) -> Verb {
        Verb {
            invocation: VerbInvocation {
                name: name.to_string(),
                args: None,
            },
            key_desc: key.map_or("".to_string(), keys::key_event_desc),
            key,
            args_parser: None,
            shortcut,
            execution: format!(":{}", name),
            description: Some(description.to_string()),
            from_shell: false,
            leave_broot: true, // ignored
            confirm: false,    // ignored
            selection_condition: SelectionType::Any,
        }
    }

    /// Assuming the verb has been matched, check whether the arguments
    /// are OK according to the regex. Return none when there's no problem
    /// and return the error to display if arguments don't match
    pub fn match_error(&self, invocation: &VerbInvocation) -> Option<String> {
        match (&invocation.args, &self.args_parser) {
            (None, None) => None,
            (None, Some(ref regex)) => {
                if regex.is_match("") {
                    None
                } else {
                    Some(self.invocation.to_string_for_name(invocation.name.clone()))
                }
            }
            (Some(ref s), Some(ref regex)) => {
                if regex.is_match(&s) {
                    None
                } else {
                    Some(self.invocation.to_string_for_name(invocation.name.clone()))
                }
            }
            (Some(_), None) => Some(format!("{} doesn't take arguments", invocation.name)),
        }
    }

    /// build the map which will be used to replace braced parts (i.e. like {part}) in
    /// the execution pattern
    fn replacement_map(
        &self,
        file: &Path,
        args: &Option<String>,
        for_shell: bool,
    ) -> HashMap<String, String> {
        let mut map = HashMap::new();
        // first we add the replacements computed from the given path
        let parent = file.parent().unwrap_or(file); // when there's no parent... we take file
        let file_str = path_to_string(file, for_shell);
        let parent_str = path_to_string(parent, for_shell);
        map.insert("file".to_string(), file_str.to_string());
        map.insert("parent".to_string(), parent_str.to_string());
        let dir_str = if file.is_dir() { file_str } else { parent_str };
        map.insert("directory".to_string(), dir_str.to_string());
        // then the ones computed from the user input
        if let Some(args) = args {
            if let Some(r) = &self.args_parser {
                if let Some(input_cap) = r.captures(&args) {
                    for name in r.capture_names().flatten() {
                        if let Some(c) = input_cap.name(name) {
                            map.insert(name.to_string(), c.as_str().to_string());
                        }
                    }
                }
            } else {
                warn!("invocation args given but none expected");
                // maybe tell the user?
            }
        }
        map
    }

    pub fn write_status(
        &self,
        w: &mut W,
        task: Option<&'static str>,
        path: PathBuf,
        invocation: &VerbInvocation,
        screen: &Screen,
    ) -> Result<(), ProgramError> {
        if let Some(err) = self.match_error(invocation) {
            Status::new(task, Composite::from_inline(&err), true).display(w, screen)
        } else {
            let verb_description;
            let markdown;
            let composite = if let Some(description) = &self.description {
                markdown = format!(
                    "Hit *enter* to **{}**: {}",
                    &self.invocation.name, description,
                );
                Composite::from_inline(&markdown)
            } else {
                verb_description = self.shell_exec_string(&path, &invocation.args);
                mad_inline!(
                    "Hit *enter* to **$0**: `$1`",
                    &self.invocation.name,
                    &verb_description,
                )
            };
            Status::new(task, composite, false).display(w, screen)
        }
    }

    /// build the cmd result for a verb defined with an exec pattern.
    /// Calling this function on a built-in doesn't make sense
    pub fn to_cmd_result(
        &self,
        file: &Path,
        args: &Option<String>,
        _screen: &mut Screen,
        con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError> {
        Ok(if self.from_shell {
            if let Some(ref export_path) = con.launch_args.cmd_export_path {
                // Broot was probably launched as br.
                // the whole command is exported in the passed file
                let f = OpenOptions::new().append(true).open(export_path)?;
                writeln!(&f, "{}", self.shell_exec_string(file, args))?;
                AppStateCmdResult::Quit
            } else if let Some(ref export_path) = con.launch_args.file_export_path {
                // old version of the br function: only the file is exported
                // in the passed file
                let f = OpenOptions::new().append(true).open(export_path)?;
                writeln!(&f, "{}", file.to_string_lossy())?;
                AppStateCmdResult::Quit
            } else {
                AppStateCmdResult::DisplayError(
                    "this verb needs broot to be launched as `br`. Try `broot --install` if necessary.".to_string()
                )
            }
        } else {
            let launchable = external::Launchable::program(self.exec_token(file, args))?;
            if self.leave_broot {
                AppStateCmdResult::from(launchable)
            } else {
                info!("Executing not leaving, launchable {:?}", launchable);
                let execution = launchable.execute();
                match execution {
                    Ok(()) => {
                        debug!("ok");
                        AppStateCmdResult::RefreshState { clear_cache: true }
                    }
                    Err(e) => {
                        warn!("launchable failed : {:?}", e);
                        AppStateCmdResult::DisplayError(e.to_string())
                    }
                }
            }
        })
    }

    /// build the token which can be used to launch en executable.
    /// This doesn't make sense for a built-in.
    pub fn exec_token(&self, file: &Path, args: &Option<String>) -> Vec<String> {
        let map = self.replacement_map(file, args, false);
        self.execution
            .split_whitespace()
            .map(|token| {
                GROUP
                    .replace_all(token, |ec: &Captures<'_>| do_exec_replacement(ec, &map))
                    .to_string()
            })
            .collect()
    }

    /// build a shell compatible command, with escapings
    pub fn shell_exec_string(&self, file: &Path, args: &Option<String>) -> String {
        let map = self.replacement_map(file, args, true);
        GROUP
            .replace_all(&self.execution, |ec: &Captures<'_>| {
                do_exec_replacement(ec, &map)
            })
            .to_string()
            .split_whitespace()
            .map(|token| {
                let path = Path::new(token);
                if path.exists() {
                    if let Some(path) = path.to_str() {
                        return path.to_string();
                    }
                }
                token.to_string()
            })
            .collect::<Vec<String>>()
            .join(" ")
    }
}

/// replace a group in the execution string, using
///  data from the user input and from the selected line
fn do_exec_replacement(ec: &Captures<'_>, replacement_map: &HashMap<String, String>) -> String {
    let name = ec.get(1).unwrap().as_str();
    if let Some(cap) = replacement_map.get(name) {
        let cap = cap.as_str();
        if let Some(fmt) = ec.get(2) {
            match fmt.as_str() {
                "path-from-directory" => {
                    if cap.starts_with('/') {
                        cap.to_string()
                    } else {
                        normalize_path(format!(
                            "{}/{}",
                            replacement_map.get("directory").unwrap(),
                            cap
                        ))
                    }
                }
                "path-from-parent" => {
                    if cap.starts_with('/') {
                        cap.to_string()
                    } else {
                        normalize_path(format!(
                            "{}/{}",
                            replacement_map.get("parent").unwrap(),
                            cap
                        ))
                    }
                }
                _ => format!("invalid format: {:?}", fmt.as_str()),
            }
        } else {
            cap.to_string()
        }
    } else {
        format!("{{{}}}", name)
    }
}

/// Improve the path to remove and solve .. token.
///
/// This will be removed when this issue is solved: https://github.com/rust-lang/rfcs/issues/2208
///
/// Note that this operation might be a little too optimistic in some cases
/// of aliases but it's probably OK in broot.
pub fn normalize_path(mut path: String) -> String {
    let mut len_before = path.len();
    loop {
        path = regex!(r"/[^/.\\]+/\.\.").replace(&path, "").to_string();
        let len = path.len();
        if len == len_before {
            return path;
        }
        len_before = len;
    }
}
#[cfg(test)]
mod path_normalize_tests {

    use crate::verbs::normalize_path;

    fn check(before: &str, after: &str) {
        assert_eq!(normalize_path(before.to_string()), after.to_string());
    }

    #[test]
    fn test_path_normalization() {
        check("/abc/test/../thing.png", "/abc/thing.png");
        check("/abc/def/../../thing.png", "/thing.png");
        check("/home/dys/test", "/home/dys/test");
        check("/home/dys/..", "/home");
        check("/home/dys/../", "/home/");
        check("/..", "/..");
        check("../test", "../test");
        check("/home/dys/../../../test", "/../test");
        check(
            "/home/dys/dev/broot/../../../canop/test",
            "/home/canop/test",
        );
    }
}
