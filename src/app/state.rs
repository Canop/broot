use {
    super::*,
    crate::{
        command::{Command, TriggerType},
        display::{Areas, Screen, W},
        errors::ProgramError,
        flag::Flag,
        pattern::Pattern,
        selection_type::SelectionType,
        skin::PanelSkin,
        task_sync::Dam,
        verb::*,
    },
    std::path::{Path, PathBuf},
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

    fn on_pattern(
        &mut self,
        _pat: Pattern,
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
        areas: &Areas,
        screen: &mut Screen,
        panel_skin: &PanelSkin,
        con: &AppContext,
        panel_purpose: PanelPurpose,
    ) -> Result<AppStateCmdResult, ProgramError>;

    /// change the state, does no rendering
    fn on_command(
        &mut self,
        cmd: &Command,
        other_path: &Option<PathBuf>,
        areas: &Areas,
        screen: &mut Screen,
        panel_skin: &PanelSkin,
        con: &AppContext,
        panel_purpose: PanelPurpose,
    ) -> Result<AppStateCmdResult, ProgramError> {
        self.clear_pending();
        match cmd {
            Command::Click(x, y) => self.on_click(*x, *y, screen, con),
            Command::DoubleClick(x, y) => self.on_double_click(*x, *y, screen, con),
            Command::PatternEdit(parts) => {
                let search_mode = con.search_modes.search_mode(&parts.mode);
                match search_mode.and_then(|sm| Pattern::new(sm, &parts.pattern, &parts.flags)) {
                    Ok(pattern) => self.on_pattern(pattern, con),
                    Err(e) => Ok(AppStateCmdResult::DisplayError(format!("{}", e))),
                }
            }
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
                        areas,
                        screen,
                        panel_skin,
                        con,
                        panel_purpose,
                    ),
                    VerbExecution::External(external) => external.to_cmd_result(
                        self.selected_path(),
                        other_path,
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
                areas,
                screen,
                panel_skin,
                con,
                panel_purpose,
            ),
            Command::VerbInvocate(invocation) => match con.verb_store.search(&invocation.name) {
                PrefixSearchResult::Match(_, verb) => {
                    if let Some(err) = verb.check_args(invocation, other_path) {
                        Ok(AppStateCmdResult::DisplayError(err))
                    } else {
                        match &verb.execution {
                            VerbExecution::Internal(internal_exec) => self.on_internal(
                                internal_exec,
                                Some(invocation),
                                TriggerType::Input,
                                areas,
                                screen,
                                panel_skin,
                                con,
                                panel_purpose,
                            ),
                            VerbExecution::External(external) => {
                                external.to_cmd_result(
                                    self.selected_path(),
                                    other_path,
                                    &invocation.args,
                                    con,
                                )
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

    fn do_pending_task(
        &mut self,
        screen: &mut Screen,
        con: &AppContext,
        dam: &mut Dam,
    );

    fn get_pending_task(&self) -> Option<&'static str>;

    fn display(
        &mut self,
        w: &mut W,
        screen: &Screen,
        state_area: Area,
        skin: &PanelSkin,
        con: &AppContext,
    ) -> Result<(), ProgramError>;

    fn get_status(
        &self,
        cmd: &Command,
        other_path: &Option<PathBuf>,
        con: &AppContext,
    ) -> Status;

    /// return the flags to display
    fn get_flags(&self) -> Vec<Flag>;

    fn get_starting_input(&self, _con: &AppContext) -> String {
        String::new()
    }
}
