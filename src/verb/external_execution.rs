//! Special groups:
//! {file}
//! {directory}
//! {parent}
//! {other-panel-file}
//! {other-panel-directory}
//! {other-panel-parent}

use {
    super::{ExternalExecutionMode, VerbInvocation},
    crate::{
        app::*,
        display::W,
        errors::{ConfError, ProgramError},
        launchable::Launchable,
        path,
        path_anchor::PathAnchor,
        selection_type::SelectionType,
    },
    regex::{Captures, Regex},
    std::{
        collections::HashMap,
        fs::OpenOptions,
        io::Write,
        path::{Path, PathBuf},
    },
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
    /// and it's a path (when it's not None, the user can type ctrl-P
    /// to select the argument in another panel)
    pub arg_selection_type: Option<SelectionType>,

    pub arg_anchor: PathAnchor,

    // /// whether we need to have a secondary panel for execution
    // /// (which is the case when an invocation has {other-panel-file})
    pub need_another_panel: bool,
}

impl ExternalExecution {
    pub fn new(
        invocation_str: &str,
        execution_str: &str,
        exec_mode: ExternalExecutionMode,
    ) -> Result<Self, ConfError> {
        let invocation_pattern = VerbInvocation::from(invocation_str);
        let mut args_parser = None;
        let mut arg_selection_type = None;
        let mut arg_anchor = PathAnchor::Unspecified;
        let mut need_another_panel = false;
        if let Some(args) = &invocation_pattern.args {
            let spec = GROUP.replace_all(args, r"(?P<$1>.+)");
            let spec = format!("^{}$", spec);
            args_parser = match Regex::new(&spec) {
                Ok(regex) => Some(regex),
                Err(_) => {
                    return Err(ConfError::InvalidVerbInvocation { invocation: spec });
                }
            };
            if let Some(group) = GROUP.find(args) {
                if group.start() == 0 && group.end() == args.len() {
                    // there's one group, covering the whole args
                    arg_selection_type = Some(SelectionType::Any);
                    let group_str = group.as_str();
                    if group_str.ends_with("path-from-parent}") {
                        arg_anchor = PathAnchor::Parent;
                    } else if group_str.ends_with("path-from-directory}") {
                        arg_anchor = PathAnchor::Directory;
                    }
                }
            }
        }
        for group in GROUP.find_iter(execution_str) {
            if group.as_str().starts_with("{other-panel-") {
                need_another_panel = true;
            }
        }
        Ok(Self {
            invocation_pattern,
            args_parser,
            exec_pattern: execution_str.to_string(),
            exec_mode,
            arg_selection_type,
            arg_anchor,
            need_another_panel,
        })
    }

    pub fn name(&self) -> &str {
        &self.invocation_pattern.name
    }

    /// Assuming the verb has been matched, check whether the arguments
    /// are OK according to the regex. Return none when there's no problem
    /// and return the error to display if arguments don't match
    pub fn check_args(
        &self,
        invocation: &VerbInvocation,
        other_path: &Option<PathBuf>,
    ) -> Option<String> {
        if self.need_another_panel && other_path.is_none() {
            return Some("This verb needs exactly two panels".to_string());
        }
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
        other_file: &Option<PathBuf>,
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
        map.insert("directory".to_string(), dir_str);
        if self.need_another_panel {
            if let Some(other_file) = other_file {
                let other_parent = other_file.parent().unwrap_or(other_file);
                let other_file_str = path_to_string(other_file, for_shell);
                let other_parent_str = path_to_string(other_parent, for_shell);
                map.insert("other-panel-file".to_string(), other_file_str.to_string());
                map.insert("other-panel-parent".to_string(), other_parent_str.to_string());
                let other_dir_str = if other_file.is_dir() { other_file_str } else { other_parent_str };
                map.insert("other-panel-directory".to_string(), other_dir_str);
            }
        }
        // then the ones computed from the user input
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
        w: &mut W,
        file: &Path,
        other_file: &Option<PathBuf>,
        args: &Option<String>,
        con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError> {
        if self.exec_mode.is_from_shell() {
            self.exec_from_shell_cmd_result(file, other_file, args, con)
        } else {
            self.exec_cmd_result(w, file, other_file, args)
        }
    }

    /// build the cmd result as an executable which will be called from shell
    fn exec_from_shell_cmd_result(
        &self,
        file: &Path,
        other_file: &Option<PathBuf>,
        args: &Option<String>,
        con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError> {
        if let Some(ref export_path) = con.launch_args.cmd_export_path {
            // Broot was probably launched as br.
            // the whole command is exported in the passed file
            let f = OpenOptions::new().append(true).open(export_path)?;
            writeln!(&f, "{}", self.shell_exec_string(file, other_file, args))?;
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
        w: &mut W,
        file: &Path,
        other_file: &Option<PathBuf>,
        args: &Option<String>,
    ) -> Result<AppStateCmdResult, ProgramError> {
        let launchable = Launchable::program(self.exec_token(file, other_file, args))?;
        if self.exec_mode.is_leave_broot() {
            Ok(AppStateCmdResult::from(launchable))
        } else {
            info!("Executing not leaving, launchable {:?}", launchable);
            let execution = launchable.execute(Some(w));
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
    fn exec_token(
        &self,
        file: &Path,
        other_file: &Option<PathBuf>,
        args: &Option<String>,
    ) -> Vec<String> {
        let map = self.replacement_map(file, other_file, args, false);
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
    pub fn shell_exec_string(
        &self,
        file: &Path,
        other_file: &Option<PathBuf>,
        args: &Option<String>,
    ) -> String {
        let map = self.replacement_map(file, other_file, args, true);
        GROUP
            .replace_all(&self.exec_pattern, |ec: &Captures<'_>| {
                path::do_exec_replacement(ec, &map)
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
