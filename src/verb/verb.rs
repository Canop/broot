use {
    super::*,
    crate::{
        app::{Selection, SelectionType, Status},
        errors::ConfError,
        keys,
        path,
        path_anchor::PathAnchor,
    },
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    std::path::PathBuf,
};

/// what makes a verb.
///
/// Verbs are the engines of broot commands, and apply
/// - to the selected file (if user-defined, then must contain {file}, {parent} or {directory})
/// - to the current app state
/// There are two types of verbs executions:
/// - external programs or commands (cd, mkdir, user defined commands, etc.)
/// - internal behaviors (focusing a path, going back, showing the help, etc.)
/// Some verbs are builtins, some other ones are created by configuration.
/// Both builtins and configured vers can be internal or external based.
#[derive(Debug, Clone)]
pub struct Verb {
    /// names (like "cd", "focus", "focus_tab", "c") by which
    /// a verb can be called.
    /// Can be empty if the verb is only called with a key shortcut.
    /// Right now there's no way for it to contain more than 2 elements
    /// but this may change.
    pub names: Vec<String>,

    /// key shortcuts
    pub keys: Vec<KeyEvent>,

    /// description of the optional keyboard key(s) triggering that verb
    pub keys_desc: String,

    /// How the verb will be executed
    pub execution: VerbExecution,

    /// a description
    pub description: VerbDescription,

    /// the type of selection this verb applies to
    pub selection_condition: SelectionType,
}

impl From<ExternalExecution> for Verb {
    fn from(external_exec: ExternalExecution) -> Self {
        let name = Some(external_exec.name().to_string());
        let description = VerbDescription::from_code(external_exec.exec_pattern.to_string());
        let execution = VerbExecution::External(external_exec);
        Self::new(name, execution, description)
    }
}

impl Verb {

    pub fn new(
        name: Option<String>,
        execution: VerbExecution,
        description: VerbDescription,
    ) -> Self {
        let mut names = Vec::new();
        if let Some(name) = name {
            names.push(name);
        }
        Self {
            names,
            keys: Vec::new(),
            keys_desc: "".to_string(),
            execution,
            description,
            selection_condition: SelectionType::Any,
        }
    }

    pub fn internal(internal: Internal) -> Self {
        let name = Some(internal.name().to_string());
        let execution = VerbExecution::Internal(InternalExecution::from_internal(internal));
        let description = VerbDescription::from_text(internal.description().to_string());
        Self::new(name, execution, description)
    }

    pub fn internal_bang(internal: Internal) -> Self {
        let name = None;
        let execution =
            VerbExecution::Internal(InternalExecution::from_internal_bang(internal, true));
        let description = VerbDescription::from_text(internal.description().to_string());
        Self::new(name, execution, description)
    }

    pub fn external(
        invocation_str: &str,
        execution_str: &str,
        exec_mode: ExternalExecutionMode,
    ) -> Result<Self, ConfError> {
        Ok(Self::from(ExternalExecution::new(
            invocation_str,
            execution_str,
            exec_mode,
        )?))
    }

    pub fn with_key(mut self, key: KeyEvent) -> Self {
        self.keys.push(key);
        self.keys_desc = self
            .keys
            .iter()
            .map(|&k| keys::key_event_desc(k))
            .collect::<Vec<String>>() // no way to join an iterator today ?
            .join(", ");
        self
    }
    pub fn with_alt_key(self, chr: char) -> Self {
        self.with_key(KeyEvent {
            code: KeyCode::Char(chr),
            modifiers: KeyModifiers::ALT,
        })
    }
    pub fn with_control_key(self, chr: char) -> Self {
        self.with_key(KeyEvent {
            code: KeyCode::Char(chr),
            modifiers: KeyModifiers::CONTROL,
        })
    }
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = VerbDescription::from_text(description.to_string());
        self
    }
    pub fn with_shortcut(mut self, shortcut: &str) -> Self {
        self.names.push(shortcut.to_string());
        self
    }

    /// Assuming the verb has been matched, check whether the arguments
    /// are OK according to the regex. Return none when there's no problem
    /// and return the error to display if arguments don't match.
    pub fn check_args(
        &self,
        invocation: &VerbInvocation,
        other_path: &Option<PathBuf>,
    ) -> Option<String> {
        match &self.execution {
            VerbExecution::Internal(internal_exec) => internal_exec.check_args(invocation, other_path),
            VerbExecution::External(external_exec) => external_exec.check_args(invocation, other_path),
        }
    }

    pub fn get_status(
        &self,
        sel: Selection<'_>,
        other_path: &Option<PathBuf>,
        invocation: &VerbInvocation,
    ) -> Status {
        if let Some(err) = self.check_args(invocation, other_path) {
            Status::new(err, true)
        } else {
            let name = self.names.get(0).unwrap_or(&invocation.name);
            let markdown = match &self.execution {
                VerbExecution::External(external_exec) => {
                    let exec_desc = external_exec.shell_exec_string(sel, other_path, &invocation.args);
                    format!("Hit *enter* to **{}**: `{}`", name, &exec_desc)
                }
                VerbExecution::Internal(internal_exec) => {
                    let pb;
                    let arg = invocation.args.as_ref().or_else(|| internal_exec.arg.as_ref());
                    let arg_path = if let Some(arg) = arg {
                        pb = path::path_from(sel.path, PathAnchor::Unspecified, arg);
                        &pb
                    } else {
                        sel.path
                    };
                    if let Some(special_desc) = internal_exec.internal.applied_description(arg_path) {
                        format!("Hit *enter* to **{}**: {}", name, special_desc)
                    } else if self.description.code {
                        format!("Hit *enter* to **{}**: `{}`", name, &self.description.content)
                    } else {
                        format!("Hit *enter* to **{}**: {}", name, &self.description.content)
                    }
                }
            };
            Status::new(markdown, false)
        }
    }

    /// in case the verb take only one argument of type path, return
    /// the selection type of this unique argument
    pub fn get_arg_selection_type(&self) -> Option<SelectionType> {
        match &self.execution {
            VerbExecution::External(external) => external.arg_selection_type,
            _ => None,
        }
    }

    pub fn get_arg_anchor(&self) -> PathAnchor {
        match &self.execution {
            VerbExecution::External(external) => external.arg_anchor,
            _ => PathAnchor::Unspecified,
        }
    }

    pub fn get_internal(&self) -> Option<Internal> {
        match &self.execution {
            VerbExecution::Internal(internal_exec) => Some(internal_exec.internal),
            _ => None,
        }
    }

    pub fn set_working_dir(&mut self, b: bool) {
        if let VerbExecution::External(external) = &mut self.execution {
            external.set_working_dir = b;
        }
    }
}
