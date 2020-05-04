use {
    super::{ExternalExecutionMode, VerbInvocation},
    crate::{
        app::*,
        errors::{ConfError, ProgramError},
        launchable::Launchable,
        path,
        selection_type::SelectionType,
    },
    regex::{Captures, Regex},
    std::{collections::HashMap, fs::OpenOptions, io::Write, path::Path},
};

fn path_to_string(path: &Path, for_shell: bool) -> String {
    if for_shell {
        path::escape_for_shell(path)
    } else {
        path.to_string_lossy().to_string()
    }
}

lazy_static! {
    static ref GROUP: Regex = Regex::new(r"\{([^{}:]+)(?::([^{}:]+))?\}").unwrap();
}

fn make_invocation_args_regex(spec: &str) -> Result<Regex, ConfError> {
    let spec = GROUP.replace_all(spec, r"(?P<$1>.+)");
    let spec = format!("^{}$", spec);
    Regex::new(&spec.to_string())
        .or_else(|_| Err(ConfError::InvalidVerbInvocation { invocation: spec }))
}

fn determine_arg_selection_type(args: &str) -> Option<SelectionType> {
    GROUP
        .find(args)
        .filter(|m| {
            info!(" m: {:?}", m);
            m.start() == 0 && m.end() == args.len()
        })
        .map(|_| SelectionType::Any)
}

/// Definition of how the user input should be interpreted
/// to be executed in an external command.
#[derive(Debug, Clone)]
pub struct ExternalExecution {
    /// pattern of how the command is supposed to be typed in the input
    invocation_pattern: VerbInvocation,

    /// a regex to read the arguments in the user input
    args_parser: Option<Regex>,

    /// the pattern which will result in an exectuable string when
    /// completed with the args
    pub exec_pattern: String,

    /// how the external process must be launched
    pub exec_mode: ExternalExecutionMode,

    /// contain the type of selection in case there's only one arg
    /// and it's a path (when it's not None, the user can type Tab
    /// to select the argument in another panel)
    pub arg_selection_type: Option<SelectionType>,
}

impl ExternalExecution {
    pub fn new(
        invocation_str: &str,
        execution_str: &str,
        exec_mode: ExternalExecutionMode,
    ) -> Result<Self, ConfError> {
        let invocation_pattern = VerbInvocation::from(invocation_str);
        let arg_selection_type = invocation_pattern
            .args
            .as_ref()
            .and_then(|args| determine_arg_selection_type(&args));
        let args_parser = invocation_pattern
            .args
            .as_ref()
            .map(|args| make_invocation_args_regex(&args))
            .transpose()?;
        Ok(Self {
            invocation_pattern,
            args_parser,
            exec_pattern: execution_str.to_string(),
            exec_mode,
            arg_selection_type,
        })
    }

    pub fn name(&self) -> &str {
        &self.invocation_pattern.name
    }

    /// Assuming the verb has been matched, check whether the arguments
    /// are OK according to the regex. Return none when there's no problem
    /// and return the error to display if arguments don't match
    pub fn check_args(&self, invocation: &VerbInvocation) -> Option<String> {
        match (&invocation.args, &self.args_parser) {
            (None, None) => None,
            (None, Some(ref regex)) => {
                if regex.is_match("") {
                    None
                } else {
                    Some(self.invocation_pattern.to_string_for_name(&invocation.name))
                }
            }
            (Some(ref s), Some(ref regex)) => {
                if regex.is_match(&s) {
                    None
                } else {
                    Some(self.invocation_pattern.to_string_for_name(&invocation.name))
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
        debug!("building repmap, args_parser={:?}", &self.args_parser);
        let default_args;
        let args = match args {
            Some(s) => s,
            None => {
                // empty args are useful when the args_parser contains
                // an optional group
                default_args = String::new();
                &default_args
            }
        };
        if let Some(r) = &self.args_parser {
            if let Some(input_cap) = r.captures(&args) {
                for name in r.capture_names().flatten() {
                    if let Some(c) = input_cap.name(name) {
                        map.insert(name.to_string(), c.as_str().to_string());
                    }
                }
            }
        }
        map
    }

    pub fn to_cmd_result(
        &self,
        file: &Path,
        args: &Option<String>,
        con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError> {
        if self.exec_mode.from_shell() {
            self.exec_from_shell_cmd_result(file, args, con)
        } else {
            self.exec_cmd_result(file, args)
        }
    }

    /// build the cmd result as an executable which will be called from shell
    fn exec_from_shell_cmd_result(
        &self,
        file: &Path,
        args: &Option<String>,
        con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError> {
        if let Some(ref export_path) = con.launch_args.cmd_export_path {
            // Broot was probably launched as br.
            // the whole command is exported in the passed file
            let f = OpenOptions::new().append(true).open(export_path)?;
            writeln!(&f, "{}", self.shell_exec_string(file, args))?;
            Ok(AppStateCmdResult::Quit)
        } else if let Some(ref export_path) = con.launch_args.file_export_path {
            // old version of the br function: only the file is exported
            // in the passed file
            let f = OpenOptions::new().append(true).open(export_path)?;
            writeln!(&f, "{}", file.to_string_lossy())?;
            Ok(AppStateCmdResult::Quit)
        } else {
            Ok(AppStateCmdResult::DisplayError(
                "this verb needs broot to be launched as `br`. Try `broot --install` if necessary."
                    .to_string(),
            ))
        }
    }

    /// build the cmd result as an executable which will be called in a process
    /// launched by broot
    fn exec_cmd_result(
        &self,
        file: &Path,
        args: &Option<String>,
    ) -> Result<AppStateCmdResult, ProgramError> {
        let launchable = Launchable::program(self.exec_token(file, args))?;
        if self.exec_mode.leave_broot() {
            Ok(AppStateCmdResult::from(launchable))
        } else {
            info!("Executing not leaving, launchable {:?}", launchable);
            let execution = launchable.execute();
            match execution {
                Ok(()) => {
                    debug!("ok");
                    Ok(AppStateCmdResult::RefreshState { clear_cache: true })
                }
                Err(e) => {
                    warn!("launchable failed : {:?}", e);
                    Ok(AppStateCmdResult::DisplayError(e.to_string()))
                }
            }
        }
    }

    /// build the token which can be used to launch en executable.
    /// This doesn't make sense for a built-in.
    fn exec_token(&self, file: &Path, args: &Option<String>) -> Vec<String> {
        let map = self.replacement_map(file, args, false);
        self.exec_pattern
            .split_whitespace()
            .map(|token| {
                GROUP
                    .replace_all(token, |ec: &Captures<'_>| {
                        path::do_exec_replacement(ec, &map)
                    })
                    .to_string()
            })
            .collect()
    }

    /// build a shell compatible command, with escapings
    pub fn shell_exec_string(&self, file: &Path, args: &Option<String>) -> String {
        let map = self.replacement_map(file, args, true);
        GROUP
            .replace_all(&self.exec_pattern, |ec: &Captures<'_>| {
                path::do_exec_replacement(ec, &map)
            })
            .to_string()
            .split_whitespace()
            .map(|token| {
                debug!("make path from {:?} token", &token);
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
