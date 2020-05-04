use {
    super::*,
    crate::{
        command::{Command, TriggerType},
        display::{Screen, W},
        errors::ProgramError,
        selection_type::SelectionType,
        task_sync::Dam,
        verb::{InternalExecution, PrefixSearchResult, VerbExecution, VerbInvocation},
    },
    std::path::Path,
    termimad::Area,
};

/// a whole application state, stackable to allow reverting
///  to a previous one
pub trait AppState {
    /// called on start of on_command
    fn clear_pending(&mut self) {}

    fn on_click(
        &mut self,
        _x: u16,
        _y: u16,
        _screen: &mut Screen,
        _con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError> {
        Ok(AppStateCmdResult::Keep)
    }

    fn on_double_click(
        &mut self,
        _x: u16,
        _y: u16,
        _screen: &mut Screen,
        _con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError> {
        Ok(AppStateCmdResult::Keep)
    }

    fn on_fuzzy_pattern_edit(
        &mut self,
        _pat: &str,
        _con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError> {
        Ok(AppStateCmdResult::Keep)
    }

    fn on_regex_pattern_edit(
        &mut self,
        _pat: &str,
        _flags: &str,
        _con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError> {
        Ok(AppStateCmdResult::Keep)
    }

    /// execute the internal with the optional given invocation.
    ///
    /// The invocation comes from the input and may be related
    /// to a different verb (the verb may have been triggered
    /// by a key shorctut)
    fn on_internal(
        &mut self,
        internal_exec: &InternalExecution,
        input_invocation: Option<&VerbInvocation>,
        trigger_type: TriggerType,
        screen: &mut Screen,
        con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError>;

    fn on_command(
        &mut self,
        cmd: &Command,
        screen: &mut Screen,
        con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError> {
        self.clear_pending();
        match cmd {
            Command::Click(x, y) => self.on_click(*x, *y, screen, con),
            Command::DoubleClick(x, y) => self.on_double_click(*x, *y, screen, con),
            Command::FuzzyPatternEdit(pat) => self.on_fuzzy_pattern_edit(pat, con),
            Command::RegexEdit(pat, flags) => self.on_regex_pattern_edit(pat, flags, con),
            Command::VerbTrigger {
                index,
                input_invocation,
            } => {
                let verb = &con.verb_store.verbs[*index];
                match &verb.execution {
                    VerbExecution::Internal(internal_exec) => self.on_internal(
                        internal_exec,
                        input_invocation.as_ref(),
                        TriggerType::Other,
                        screen,
                        con,
                    ),
                    VerbExecution::External(external) => external.to_cmd_result(
                        self.selected_path(),
                        if let Some(inv) = &input_invocation {
                            &inv.args
                        } else {
                            &None
                        },
                        con,
                    ),
                }
            }
            Command::Internal {
                internal,
                input_invocation,
            } => self.on_internal(
                &InternalExecution::from_internal(*internal),
                input_invocation.as_ref(),
                TriggerType::Other,
                screen,
                con,
            ),
            Command::VerbInvocate(invocation) => match con.verb_store.search(&invocation.name) {
                PrefixSearchResult::Match(verb) => {
                    if let Some(err) = verb.check_args(invocation) {
                        Ok(AppStateCmdResult::DisplayError(err))
                    } else {
                        match &verb.execution {
                            VerbExecution::Internal(internal_exec) => self.on_internal(
                                internal_exec,
                                Some(invocation),
                                TriggerType::Input,
                                screen,
                                con,
                            ),
                            VerbExecution::External(external) => {
                                external.to_cmd_result(self.selected_path(), &invocation.args, con)
                            }
                        }
                    }
                }
                _ => Ok(AppStateCmdResult::verb_not_found(&invocation.name)),
            },
            Command::None | Command::VerbEdit(_) => {
                // we do nothing here, the real job is done in get_status
                Ok(AppStateCmdResult::Keep)
            }
        }
    }

    fn selected_path(&self) -> &Path;
    fn selection_type(&self) -> SelectionType;

    fn refresh(&mut self, screen: &Screen, con: &AppContext) -> Command;

    fn do_pending_task(&mut self, screen: &mut Screen, dam: &mut Dam);

    fn get_pending_task(&self) -> Option<&'static str>;

    fn display(
        &mut self,
        w: &mut W,
        screen: &Screen,
        state_area: Area,
        con: &AppContext,
    ) -> Result<(), ProgramError>;

    fn write_flags(
        &self,
        w: &mut W,
        screen: &mut Screen,
        con: &AppContext,
    ) -> Result<(), ProgramError>;

    fn get_status(&self, cmd: &Command, con: &AppContext) -> Status;
}
