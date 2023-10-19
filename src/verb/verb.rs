use {
    super::*,
    crate::{
        app::*,
        errors::ConfError,
        path::PathAnchor,
    },
    crokey::crossterm::event::KeyEvent,
    std::{
        cmp::PartialEq,
        path::PathBuf,
        ptr,
    },
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
///
/// Verbs can't be cloned. Two verbs are equal if they have the same address
/// in memory.
#[derive(Debug)]
pub struct Verb {

    pub id: VerbId,

    /// names (like "cd", "focus", "focus_tab", "c") by which
    /// a verb can be called.
    /// Can be empty if the verb is only called with a key shortcut.
    /// Right now there's no way for it to contain more than 2 elements
    /// but this may change.
    pub names: Vec<String>,

    /// key shortcuts
    pub keys: Vec<KeyEvent>,

    /// how the input must be checked and interpreted
    /// Can be empty if the verb is only called with a key shortcut.
    pub invocation_parser: Option<InvocationParser>,

    /// how the verb will be executed
    pub execution: VerbExecution,

    /// a description
    pub description: VerbDescription,

    /// the type of selection this verb applies to
    pub selection_condition: FileTypeCondition,

    /// extension filtering. If empty, all extensions apply
    pub file_extensions: Vec<String>,

    /// whether the verb needs a selection
    pub needs_selection: bool,

    /// whether we need to have a secondary panel for execution
    /// (which is the case when the execution pattern has {other-panel-file})
    pub needs_another_panel: bool,

    /// if true (default) verbs are directly executed when
    /// triggered with a keyboard shortcut
    pub auto_exec: bool,

    /// whether to show the verb in help screen
    /// (if we show all input related actions, the doc is unusable)
    pub show_in_doc: bool,

    pub panels: Vec<PanelStateType>,
}

impl PartialEq for Verb {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self, other)
    }
}

impl Verb {

    pub fn new(
        id: VerbId,
        invocation_str: Option<&str>,
        execution: VerbExecution,
        description: VerbDescription,
    ) -> Result<Self, ConfError> {
        let invocation_parser = invocation_str.map(InvocationParser::new).transpose()?;
        let mut names = Vec::new();
        if let Some(ref invocation_parser) = invocation_parser {
            names.push(invocation_parser.name().to_string());
        }
        let (
            needs_selection,
            needs_another_panel,
        ) = match &execution {
            VerbExecution::Internal(ie) => (
                ie.needs_selection(),
                false,
            ),
            VerbExecution::External(ee) => (
                ee.exec_pattern.has_selection_group(),
                ee.exec_pattern.has_other_panel_group(),
            ),
            VerbExecution::Sequence(se) => (
                se.sequence.has_selection_group(),
                se.sequence.has_other_panel_group(),
            )
        };
        Ok(Self {
            id,
            names,
            keys: Vec::new(),
            invocation_parser,
            execution,
            description,
            selection_condition: FileTypeCondition::Any,
            file_extensions: Vec::new(),
            needs_selection,
            needs_another_panel,
            auto_exec: true,
            show_in_doc: true,
            panels: Vec::new(),
        })
    }
    pub fn with_key(&mut self, key: KeyEvent) -> &mut Self {
        self.keys.push(key);
        self
    }
    pub fn add_keys(&mut self, keys: Vec<KeyEvent>) {
        for key in keys {
            self.keys.push(key);
        }
    }
    pub fn no_doc(&mut self) -> &mut Self {
        self.show_in_doc = false;
        self
    }
    pub fn with_description(&mut self, description: &str) -> &mut Self {
        self.description = VerbDescription::from_text(description.to_string());
        self
    }
    pub fn with_shortcut(&mut self, shortcut: &str) -> &mut Self {
        self.names.push(shortcut.to_string());
        self
    }
    pub fn with_condition(&mut self, selection_condition: FileTypeCondition) -> &mut Self {
        self.selection_condition = selection_condition;
        self
    }
    pub fn needing_another_panel(&mut self) -> &mut Self {
        self.needs_another_panel = true;
        self
    }
    pub fn with_auto_exec(&mut self, b: bool) -> &mut Self {
        self.auto_exec = b;
        self
    }

    pub fn has_name(&self, searched_name: &str) -> bool {
        self.names.iter().any(|name| name == searched_name)
    }

    /// Assuming the verb has been matched, check whether the arguments
    /// are OK according to the regex. Return none when there's no problem
    /// and return the error to display if arguments don't match.
    pub fn check_args(
        &self,
        sel_info: SelInfo<'_>,
        invocation: &VerbInvocation,
        other_path: &Option<PathBuf>,
    ) -> Option<String> {
        match sel_info {
            SelInfo::None => self.check_sel_args(None, invocation, other_path),
            SelInfo::One(sel) => self.check_sel_args(Some(sel), invocation, other_path),
            SelInfo::More(stage) => {
                stage.paths().iter()
                    .filter_map(|path| {
                        let sel = Selection {
                            path,
                            line: 0,
                            stype: SelectionType::from(path),
                            is_exe: false,
                        };
                        self.check_sel_args(Some(sel), invocation, other_path)
                    })
                    .next()
            }
        }
    }

    fn check_sel_args(
        &self,
        sel: Option<Selection<'_>>,
        invocation: &VerbInvocation,
        other_path: &Option<PathBuf>,
    ) -> Option<String> {
        if self.needs_selection && sel.is_none() {
            Some("This verb needs a selection".to_string())
        } else if self.needs_another_panel && other_path.is_none() {
            Some("This verb needs exactly two panels".to_string())
        } else if let Some(ref parser) = self.invocation_parser {
            parser.check_args(invocation, other_path)
        } else if invocation.args.is_some() {
            Some("This verb doesn't take arguments".to_string())
        } else {
            None
        }
    }

    pub fn get_status_markdown(
        &self,
        sel_info: SelInfo<'_>,
        app_state: &AppState,
        invocation: &VerbInvocation,
    ) -> String {
        let name = self.names.get(0).unwrap_or(&invocation.name);

        // there's one special case: the ̀ :focus` internal. As long
        // as no other internal takes args, and no other verb can
        // have an optional argument, I don't try to build a
        // generic behavior for internal optionally taking args and
        // thus I hardcode the test here.
        if let VerbExecution::Internal(internal_exec) = &self.execution {
            if internal_exec.internal == Internal::focus {
                return internal_focus::get_status_markdown(
                    self,
                    internal_exec,
                    sel_info,
                    invocation,
                    app_state,
                );
            }
        }

        let builder = || {
            ExecutionStringBuilder::with_invocation(
                &self.invocation_parser,
                sel_info,
                app_state,
                invocation.args.as_ref(),
            )
        };
        if let VerbExecution::Sequence(seq_ex) = &self.execution {
            let exec_desc = builder().shell_exec_string(
                &ExecPattern::from_string(&seq_ex.sequence.raw)
            );
            format!("Hit *enter* to **{}**: `{}`", name, &exec_desc)
        } else if let VerbExecution::External(external_exec) = &self.execution {
            let exec_desc = builder().shell_exec_string(&external_exec.exec_pattern);
            format!("Hit *enter* to **{}**: `{}`", name, &exec_desc)
        } else if self.description.code {
            format!("Hit *enter* to **{}**: `{}`", name, &self.description.content)
        } else {
            format!("Hit *enter* to **{}**: {}", name, &self.description.content)
        }
    }

    pub fn get_unique_arg_anchor(&self) -> PathAnchor {
        self.invocation_parser
            .as_ref()
            .map_or(PathAnchor::Unspecified, InvocationParser::get_unique_arg_anchor)
    }

    pub fn get_internal(&self) -> Option<Internal> {
        match &self.execution {
            VerbExecution::Internal(internal_exec) => Some(internal_exec.internal),
            _ => None,
        }
    }

    pub fn is_internal(&self, internal: Internal) -> bool {
        self.get_internal() == Some(internal)
    }

    pub fn is_some_internal(v: Option<&Verb>, internal: Internal) -> bool {
        v.map_or(false, |v| v.is_internal(internal))
    }

    pub fn is_sequence(&self) -> bool {
        matches!(self.execution, VerbExecution::Sequence(_))
    }

    pub fn can_be_called_in_panel(&self, panel_state_type: PanelStateType) -> bool {
        self.panels.is_empty() || self.panels.contains(&panel_state_type)
    }
    pub fn accepts_extension(&self, extension: Option<&str>) -> bool {
        if self.file_extensions.is_empty() {
            true
        } else {
            extension
                .map_or(false, |ext| self.file_extensions.iter().any(|ve| ve == ext))
        }
    }
}
