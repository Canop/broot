use regex::{Captures, Regex};
/// Verbs are the engines of broot commands, and apply
/// - to the selected file (if user-defined, then must contain {file}, {parent} or {directory})
/// - to the current app state
use std;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::app::AppStateCmdResult;
use crate::app_context::AppContext;
use crate::errors::ConfError;
use crate::external;
use crate::screens::Screen;
use crate::verb_invocation::VerbInvocation;

// what makes a verb.
//
// There are two types of verbs executions:
// - external programs or commands (cd, mkdir, user defined commands, etc.)
// - built in behaviors (focusing a path, going back, showing the help, etc.)
//
#[derive(Debug, Clone)]
pub struct Verb {
    pub invocation: VerbInvocation,// how the verb is supposed to be called (key may be replaced by shortcut)
    pub args_parser: Option<Regex>,
    pub shortcut: Option<String>,  // a shortcut, eg "c"
    pub execution: String,         // a pattern usable for execution, eg ":quit" or "less {file}"
    pub description: Option<String>, // a description for the user
    pub from_shell: bool, // whether it must be launched from the parent shell (eg because it's a shell function)
    pub leave_broot: bool, // only defined for external
    pub confirm: bool,
}

lazy_static! {
    static ref GROUP: Regex = Regex::new(r"\{([^{}]+)\}").unwrap();
}

pub trait VerbExecutor {
    fn execute_verb(
        &self,
        verb: &Verb,
        invocation: &VerbInvocation,
        screen: &mut Screen,
        con: &AppContext,
    ) -> io::Result<AppStateCmdResult>;
}

fn make_invocation_args_regex(spec: &str) -> Result<Regex, ConfError> {
    let spec = GROUP.replace_all(spec, r"(?P<$1>.+)");
    let spec = format!("^{}$", spec);
    Regex::new(&spec.to_string()).or_else(|_| Err(ConfError::InvalidVerbInvocation{invocation: spec}))
}
fn path_to_string(path: &Path, for_shell: bool) -> String {
    if for_shell {
        external::escape_for_shell(path)
    } else {
        path.to_string_lossy().to_string()
    }
}

impl Verb {
    pub fn create_external(
        invocation_str: &str,
        shortcut: Option<String>,
        execution: String,
        description: Option<String>,
        from_shell: bool,
        leave_broot: bool,
        confirm: bool,
    ) -> Result<Verb, ConfError> {
        let invocation = VerbInvocation::from(invocation_str);
        if invocation.is_empty() {
            return Err(ConfError::InvalidVerbInvocation{
                invocation: invocation_str.to_string()
            });
        }
        let args_parser = match &invocation.args {
            Some(ref args) => Some(make_invocation_args_regex(&args)?),
            None => None,
        };
        Ok(Verb {
            invocation,
            args_parser,
            shortcut,
            execution,
            description,
            from_shell,
            leave_broot,
            confirm,
        })
    }
    // built-ins are verbs offering a logic other than the execution
    //  based on exec_pattern. They mostly modify the appstate
    pub fn create_builtin(
        key: &str,
        shortcut: Option<String>,
        description: &str,
    ) -> Verb {
        Verb {
            invocation: VerbInvocation{
                key:key.to_string(),
                args:None,
            },
            args_parser: None,
            shortcut,
            execution: format!(":{}", key),
            description: Some(description.to_string()),
            from_shell: false,
            leave_broot: true, // ignored
            confirm: false, // ignored
        }
    }

    // the key is assumed to have been already checked
    // (we can't check it here as it depends on the whole set of available verbs)
    pub fn match_error(&self, invocation: &VerbInvocation) -> Option<String> {
        match (&invocation.args, &self.args_parser) {
            (None, None) => None,
            (None, Some(ref regex)) => if regex.is_match("") {
                None
            } else {
                Some(self.invocation.to_string_for_key(&invocation.key))
            }
            (Some(ref s), Some(ref regex)) => if regex.is_match(&s) {
                None
            } else {
                Some(self.invocation.to_string_for_key(&invocation.key))
            }
            (Some(_), None) => Some(format!("{} doesn't take arguments", invocation.key)),
        }
    }

    fn replacement_map(&self, file: &Path, args: &Option<String>, for_shell: bool) -> HashMap<String, String> {
        let mut map = HashMap::new();
        // first we add the replacements computed from the given path
        let parent = file.parent().unwrap();
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
    pub fn description_for(&self, path: PathBuf, args: &Option<String>) -> String {
        if let Some(s) = &self.description {
            s.clone()
        } else {
            self.shell_exec_string(&path, args)
        }
    }
    // build the token which can be used to launch en executable.
    // This doesn't make sense for a built-in.
    pub fn exec_token(&self, file: &Path, args: &Option<String>) -> Vec<String> {
        let map = self.replacement_map(file, args, false);
        self.execution
            .split_whitespace()
            .map(|token| {
                GROUP.replace_all(token, |ec:&Captures| {
                    let name = ec.get(1).unwrap().as_str();
                    if let Some(cap) = map.get(name) {
                        cap.as_str().to_string()
                    } else {
                        format!("{{{}}}", name)
                    }
                }).to_string()
            })
            .collect()
    }
    // build a shell compatible command, with escapings
    pub fn shell_exec_string(&self, file: &Path, args: &Option<String>) -> String {
        let map = self.replacement_map(file, args, true);
        GROUP.replace_all(&self.execution, |ec:&Captures| {
            let name = ec.get(1).unwrap().as_str();
            if let Some(cap) = map.get(name) {
                cap.as_str().to_string()
            } else {
                format!("{{{}}}", name)
            }
        })
        .to_string()
        .split_whitespace()
        .map(|token| {
            let path = Path::new(token);
            if path.exists() {
                if let Ok(path) = path.canonicalize() {
                    if let Some(path) = path.to_str() {
                        return path.to_string();
                    }
                }
            }
            token.to_string()
        })
        .collect::<Vec<String>>()
        .join(" ")
    }
    // build the cmd result for a verb defined with an exec pattern.
    // Calling this function on a built-in doesn't make sense
    pub fn to_cmd_result(
        &self,
        file: &Path,
        args: &Option<String>,
        _screen: &mut Screen,
        con: &AppContext
    ) -> io::Result<AppStateCmdResult> {
        Ok(if self.from_shell {
            if let Some(ref export_path) = con.launch_args.cmd_export_path {
                // new version of the br function: the whole command is exported
                // in the passed file
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
            let launchable = external::Launchable::from(self.exec_token(file, args))?;
            if self.leave_broot {
                AppStateCmdResult::Launch(launchable)
            } else {
                info!("Executing not leaving, launchable {:?}", launchable);
                let execution = launchable.execute();
                match execution {
                    Ok(()) => {
                        info!("ok");
                        AppStateCmdResult::RefreshState
                    },
                    Err(e) => {
                        warn!("launchable failed : {:?}", e);
                        AppStateCmdResult::DisplayError(e.to_string())
                    }
                }
            }
        })
    }
}

