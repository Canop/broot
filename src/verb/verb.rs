// Verbs are the engines of broot commands, and apply
/// - to the selected file (if user-defined, then must contain {file}, {parent} or {directory})
/// - to the current app state
use {
    crate::{
        app::{
            AppContext,
            AppStateCmdResult,
        },
        errors::{ConfError, ProgramError},
        external, keys,
        screens::Screen,
        selection_type::SelectionType,
        status::Status,
    },
    crossterm::event::{
        KeyCode,
        KeyEvent,
    },
    directories::UserDirs,
    minimad::Composite,
    regex::{self, Captures, Regex},
    std::{
        collections::HashMap,
        fs::OpenOptions,
        io::Write,
        path::{Path, PathBuf},
    },
    super::{
        Internal,
        VerbExecution,
        VerbInvocation,
    },
};

lazy_static! {
    static ref GROUP: Regex = Regex::new(r"\{([^{}:]+)(?::([^{}:]+))?\}").unwrap();
}

fn make_invocation_args_regex(spec: &str) -> Result<Regex, ConfError> {
    let spec = GROUP.replace_all(spec, r"(?P<$1>.+)");
    let spec = format!("^{}$", spec);
    info!("spec = {:?}", &spec);
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

/// what makes a verb.
///
/// There are two types of verbs executions:
/// - external programs or commands (cd, mkdir, user defined commands, etc.)
/// - internal behaviors (focusing a path, going back, showing the help, etc.)
#[derive(Debug, Clone)]
pub struct Verb {

    // a name with maybe _tab, for the help page
    pub name: String,

    /// how the verb is supposed to be called, may be empty
    pub invocation: VerbInvocation,

    pub keys: Vec<KeyEvent>,

    /// description of the optional keyboard key(s) triggering that verb
    pub keys_desc: String,

    /// the regex used to parse the arguments typed by the user
    /// in the input
    pub args_parser: Option<Regex>,

    /// a shortcut, eg "c"
    pub shortcut: Option<String>,

    /// How the verb will be executed
    pub execution: VerbExecution,

    /// a description
    pub description: Option<String>,

    /// whether it must be launched from the parent shell
    pub from_shell: bool,

    /// leave broot on execution, only defined for external
    pub leave_broot: bool,

    pub selection_condition: SelectionType,
}

impl Verb {

    fn new(
        invocation: VerbInvocation,
        args_parser: Option<Regex>,
        execution: VerbExecution,
    ) -> Self {
        let name = invocation.complete_name();
        Self {
            name,
            invocation,
            keys: Vec::new(),
            keys_desc: "".to_string(),
            args_parser,
            shortcut: None,
            execution,
            description: None,
            from_shell: false,
            leave_broot: false,
            selection_condition: SelectionType::Any,
        }
    }

    pub fn internal(
        internal: Internal,
    ) -> Self {
        Self::internal_bang(internal, false)
    }

    pub fn internal_bang(
        internal: Internal,
        bang: bool,
    ) -> Self {
        let invocation = VerbInvocation {
            name: internal.name().to_string(),
            args: None,
            bang,
        };
        let execution = VerbExecution::Internal {
            internal,
            bang,
        };
        Self::new(invocation, None, execution)
    }

    pub fn external(
        invocation_str: &str,
        execution_str: &str,
    ) -> Result<Self, ConfError> {
        let invocation = VerbInvocation::from(invocation_str);
        let args_parser = invocation
            .args
            .as_ref()
            .map(|args| make_invocation_args_regex(&args))
            .transpose()?;
        let execution = VerbExecution::External(execution_str.to_string());
        Ok(Self::new(invocation, args_parser, execution))
    }

    pub fn with_key(mut self, key: KeyEvent) -> Self {
        self.keys.push(key);
        if key.code==KeyCode::Enter {
            self.selection_condition = SelectionType::File;
        }
        self.keys_desc = self.keys
            .iter()
            .map(|&k| keys::key_event_desc(k))
            .collect::<Vec<String>>() // no way to join an iterator today ?
            .join(", ");
        self
    }
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }
    pub fn with_shortcut(mut self, shortcut: &str) -> Self {
        self.shortcut = Some(shortcut.to_string());
        self
    }
    pub fn with_from_shell(mut self, from_shell: bool) -> Self {
        self.from_shell = from_shell;
        self
    }
    pub fn with_leave_broot(mut self, leave_broot: bool) -> Self {
        self.leave_broot = leave_broot;
        self
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
                    Some(self.invocation.to_string_for_name(&invocation.name))
                }
            }
            (Some(ref s), Some(ref regex)) => {
                if regex.is_match(&s) {
                    None
                } else {
                    Some(self.invocation.to_string_for_name(&invocation.name))
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

    pub fn write_status(
        &self,
        w: &mut impl Write,
        task: Option<&'static str>,
        path: PathBuf,
        invocation: &VerbInvocation,
        screen: &Screen,
    ) -> Result<(), ProgramError> {
        if let Some(err) = self.match_error(invocation) {
            Status::new(task, Composite::from_inline(&err), true).display(w, screen)
        } else {
            let markdown;
            let exec_desc;
            let composite = if let Some(description) = &self.description {
                markdown = format!(
                    "Hit *enter* to **{}**: {}",
                    &self.name,
                    description,
                );
                Composite::from_inline(&markdown)
            } else {
                match &self.execution {
                    VerbExecution::Internal{ internal, .. } => {
                        markdown = format!(
                            "Hit *enter* to **{}**: {}",
                            &self.name,
                            internal.description(),
                        );
                        Composite::from_inline(&markdown)
                    }
                    VerbExecution::External(_) => {
                        exec_desc = self.shell_exec_string(&path, &invocation.args);
                        mad_inline!(
                            "Hit *enter* to **$0**: `$1`",
                            &self.name,
                            &exec_desc,
                        )
                    }
                }
            };
            Status::new(
                task,
                composite,
                false
            ).display(w, screen)
        }
    }

    /// build the cmd result for a verb defined with an exec pattern.
    /// Calling this function on a built-in doesn't make sense
    /// TODO cleaner types so that this isn't callable on internals
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
            let launchable = external::Launchable::program(self.exec_token(file, args)?)?;
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
    /// TODO cleaner types so that this isn't callable on internals
    fn exec_token(&self, file: &Path, args: &Option<String>) -> Result<Vec<String>, ProgramError> {
        match &self.execution {
            VerbExecution::External(external) => {
                let map = self.replacement_map(file, args, false);
                Ok(external
                    .split_whitespace()
                    .map(|token| {
                        GROUP
                            .replace_all(token, |ec: &Captures<'_>| do_exec_replacement(ec, &map))
                            .to_string()
                    })
                    .collect()
                )
            }
            _ => Err(ProgramError::InternalError{ details:"not an external verb".to_string() }),
        }
    }

    /// build a shell compatible command, with escapings
    /// TODO cleaner types so that this isn't callable on internals
    fn shell_exec_string(&self, file: &Path, args: &Option<String>) -> String {
        debug!("shell_exec_string args={:?}", args);
        match &self.execution {
            VerbExecution::External(external) => {
                let map = self.replacement_map(file, args, true);
                GROUP
                    .replace_all(&external, |ec: &Captures<'_>| {
                        do_exec_replacement(ec, &map)
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
            _ => "not an external verb".to_string(), // BUG - FIXME
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum PathSource {
    Directory,
    Parent,
}
impl PathSource {
    fn replacement_map_key(&self) -> &'static str {
        match self {
            Self::Directory => "directory",
            Self::Parent => "parent",
        }
    }
}

/// build a usable path from a user input
fn path_from(
    source: PathSource,
    input: &str,
    replacement_map: &HashMap<String, String>,
) -> String {
    let tilde = regex!(r"^~(/|$)");
    if input.starts_with('/') {
        // if the input starts with a `/`, we use it as is, we don't
        // use the replacement_map
        input.to_string()
    } else if tilde.is_match(input) {
        // if the input starts with `~` as first token, we replace
        // this `~` with the user home directory and  we don't use the
        // replacement map
        tilde.replace(input, |c: &Captures| {
            if let Some(user_dirs) = UserDirs::new() {
                format!(
                    "{}{}",
                    user_dirs.home_dir().to_string_lossy(),
                    &c[1],
                )
            } else {
                warn!("no user dirs found, no expansion of ~");
                c[0].to_string()
            }
        }).to_string()
    } else {
        // we put the input behind the source (the selected directory
        // or its parent) and we normalize so that the user can type
        // paths with `../`
        normalize_path(format!(
            "{}/{}",
            replacement_map.get(source.replacement_map_key()).unwrap(),
            input
        ))
    }
}

/// replace a group in the execution string, using
///  data from the user input and from the selected line
fn do_exec_replacement(
    ec: &Captures<'_>,
    replacement_map: &HashMap<String, String>,
) -> String {
    let name = ec.get(1).unwrap().as_str();
    if let Some(cap) = replacement_map.get(name) {
        let cap = cap.as_str();
        debug!("do_exec_replacement cap={:?} with {:?}", &cap, ec.get(2));
        if let Some(fmt) = ec.get(2) {
            match fmt.as_str() {
                "path-from-directory" => path_from(PathSource::Directory, cap, replacement_map),
                "path-from-parent" => path_from(PathSource::Parent, cap, replacement_map),
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
