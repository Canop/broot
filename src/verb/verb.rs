// Verbs are the engines of broot commands, and apply
/// - to the selected file (if user-defined, then must contain {file}, {parent} or {directory})
/// - to the current app state
use {
    crate::{
        errors::{ConfError, ProgramError},
        keys,
        display::{
            Screen,
            Status,
        },
        selection_type::SelectionType,
    },
    crossterm::event::{
        KeyCode,
        KeyEvent,
    },
    minimad::Composite,
    std::{
        io::Write,
        path::PathBuf,
    },
    super::{
        External,
        ExternalExecutionMode,
        Internal,
        VerbExecution,
        VerbInvocation,
    },
};

/// what makes a verb.
///
/// There are two types of verbs executions:
/// - external programs or commands (cd, mkdir, user defined commands, etc.)
/// - internal behaviors (focusing a path, going back, showing the help, etc.)
#[derive(Debug, Clone)]
pub struct Verb {

    // a name, like "cd", "focus", "focus_tab"
    pub name: String,

    pub keys: Vec<KeyEvent>,

    /// description of the optional keyboard key(s) triggering that verb
    pub keys_desc: String,

    /// a shortcut, eg "c"
    pub shortcut: Option<String>,

    /// How the verb will be executed
    pub execution: VerbExecution,

    /// a description
    pub description: Option<String>,

    pub selection_condition: SelectionType,
}

impl From<External> for Verb {
    fn from(external: External) -> Self {
        Self::new(
            external.name().to_string(),
            VerbExecution::External(external),
        )
    }
}

impl Verb {

    fn new(
        name: String,
        execution: VerbExecution,
    ) -> Self {
        Self {
            name,
            keys: Vec::new(),
            keys_desc: "".to_string(),
            shortcut: None,
            execution,
            description: None,
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
        let name = invocation.complete_name();
        let execution = VerbExecution::Internal {
            internal,
            bang,
        };
        Self::new(name, execution)
    }

    pub fn external(
        invocation_str: &str,
        execution_str: &str,
        exec_mode: ExternalExecutionMode,
    ) -> Result<Self, ConfError> {
        let external = External::new(
            invocation_str,
            execution_str,
            exec_mode,
        )?;
        let name = external.name().to_string();
        let execution = VerbExecution::External(external);
        Ok(Self::new(name, execution))
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

    /// Assuming the verb has been matched, check whether the arguments
    /// are OK according to the regex. Return none when there's no problem
    /// and return the error to display if arguments don't match
    pub fn match_error(&self, invocation: &VerbInvocation) -> Option<String> {
        match &self.execution {
            VerbExecution::Internal{ .. } => if invocation.args.is_some() {
                Some(format!("{} doesn't take arguments", invocation.name))
            } else {
                None
            }
            VerbExecution::External(external) => external.match_error(invocation)
        }
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
                    VerbExecution::External(external) => {
                        exec_desc = external.shell_exec_string(&path, &invocation.args);
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

}

