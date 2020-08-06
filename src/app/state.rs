use {
    super::*,
    crate::{
        command::{Command, TriggerType},
        display::{Screen, W},
        errors::ProgramError,
        flag::Flag,
        help::HelpState,
        pattern::*,
        preview::{PreviewMode, PreviewState},
        print,
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
        _pat: InputPattern,
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
        w: &mut W,
        internal_exec: &InternalExecution,
        input_invocation: Option<&VerbInvocation>,
        trigger_type: TriggerType,
        cc: &CmdContext,
        screen: &mut Screen,
    ) -> Result<AppStateCmdResult, ProgramError>;

    /// a generic implementation of on_internal which may be
    /// called by states when they don't have a specific
    /// behavior to execute
    fn on_internal_generic(
        &mut self,
        _w: &mut W,
        internal_exec: &InternalExecution,
        input_invocation: Option<&VerbInvocation>,
        _trigger_type: TriggerType,
        cc: &CmdContext,
        screen: &mut Screen,
    ) -> Result<AppStateCmdResult, ProgramError> {
        let con = &cc.con;
        Ok(match internal_exec.internal {
            Internal::back => AppStateCmdResult::PopState,
            Internal::close_panel_ok => AppStateCmdResult::ClosePanel {
                validate_purpose: true,
                id: None,
            },
            Internal::close_panel_cancel => AppStateCmdResult::ClosePanel {
                validate_purpose: false,
                id: None,
            },
            Internal::help => {
                let bang = input_invocation
                    .map(|inv| inv.bang)
                    .unwrap_or(internal_exec.bang);
                if bang && cc.preview.is_none() {
                    AppStateCmdResult::NewPanel {
                        state: Box::new(HelpState::new(screen, con)),
                        purpose: PanelPurpose::None,
                        direction: HDir::Right,
                    }
                } else {
                    AppStateCmdResult::NewState(Box::new(HelpState::new(screen, con)))
                }
            }
            Internal::open_preview => self.open_preview(None, false, cc),
            Internal::preview_image => self.open_preview(Some(PreviewMode::Image), false, cc),
            Internal::preview_text => self.open_preview(Some(PreviewMode::Text), false, cc),
            Internal::preview_binary => self.open_preview(Some(PreviewMode::Hex), false, cc),
            Internal::toggle_preview => self.open_preview(None, true, cc),
            Internal::close_preview => {
                if let Some(id) = cc.preview {
                    AppStateCmdResult::ClosePanel {
                        validate_purpose: false,
                        id: Some(id),
                    }
                } else {
                    AppStateCmdResult::Keep
                }
            }
            Internal::panel_left => {
                if cc.areas.is_first() {
                    AppStateCmdResult::Keep
                } else {
                    // we ask the app to focus the panel to the left
                    AppStateCmdResult::HandleInApp(Internal::panel_left)
                }
            }
            Internal::panel_right => {
                if cc.areas.is_last() {
                    AppStateCmdResult::Keep
                } else {
                    // we ask the app to focus the panel to the left
                    AppStateCmdResult::HandleInApp(Internal::panel_right)
                }
            }
            Internal::print_path => {
                print::print_path(self.selected_path(), con)?
            }
            Internal::print_relative_path => {
                print::print_relative_path(self.selected_path(), con)?
            }
            Internal::refresh => AppStateCmdResult::RefreshState { clear_cache: true },
            Internal::quit => AppStateCmdResult::Quit,
            _ => AppStateCmdResult::Keep,
        })
    }

    /// change the state, does no rendering
    fn on_command(
        &mut self,
        w: &mut W,
        cc: &CmdContext,
        screen: &mut Screen,
    ) -> Result<AppStateCmdResult, ProgramError> {
        self.clear_pending();
        let con = &cc.con;
        match cc.cmd {
            Command::Click(x, y) => self.on_click(*x, *y, screen, con),
            Command::DoubleClick(x, y) => self.on_double_click(*x, *y, screen, con),
            Command::PatternEdit { raw, expr } => {
                match InputPattern::new(raw.clone(), expr, &cc.con) {
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
                        w,
                        internal_exec,
                        input_invocation.as_ref(),
                        TriggerType::Other,
                        cc,
                        screen,
                    ),
                    VerbExecution::External(external) => external.to_cmd_result(
                        w,
                        self.selection(),
                        &cc.other_path,
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
                w,
                &InternalExecution::from_internal(*internal),
                input_invocation.as_ref(),
                TriggerType::Other,
                cc,
                screen,
            ),
            Command::VerbInvocate(invocation) => match con.verb_store.search(&invocation.name) {
                PrefixSearchResult::Match(_, verb) => {
                    if let Some(err) = verb.check_args(invocation, &cc.other_path) {
                        Ok(AppStateCmdResult::DisplayError(err))
                    } else {
                        match &verb.execution {
                            VerbExecution::Internal(internal_exec) => self.on_internal(
                                w,
                                internal_exec,
                                Some(invocation),
                                TriggerType::Input,
                                cc,
                                screen,
                            ),
                            VerbExecution::External(external) => {
                                external.to_cmd_result(
                                    w,
                                    self.selection(),
                                    &cc.other_path,
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

    fn open_preview(
        &mut self,
        prefered_mode: Option<PreviewMode>,
        close_if_open: bool,
        cc: &CmdContext,
    ) -> AppStateCmdResult {
        if let Some(id) = cc.preview {
            if close_if_open {
                AppStateCmdResult::ClosePanel {
                    validate_purpose: false,
                    id: Some(id),
                }
            } else {
                if prefered_mode.is_some() {
                    // we'll make the preview mode change be
                    // applied on the preview panel
                    AppStateCmdResult::ApplyOnPanel { id }
                } else {
                    AppStateCmdResult::Keep
                }
            }
        } else {
            let path = self.selected_path();
            if path.is_file() {
                AppStateCmdResult::NewPanel {
                    state: Box::new(PreviewState::new(
                        path.to_path_buf(),
                        InputPattern::none(),
                        prefered_mode,
                        &cc.con,
                    )),
                    purpose: PanelPurpose::Preview,
                    direction: HDir::Right,
                }
            } else {
                AppStateCmdResult::DisplayError(
                    "only regular files can be previewed".to_string()
                )
            }
        }
    }

    fn selected_path(&self) -> &Path;

    fn selection(&self) -> Selection<'_>;

    fn refresh(&mut self, screen: &Screen, con: &AppContext) -> Command;

    fn do_pending_task(
        &mut self,
        _screen: &mut Screen,
        _con: &AppContext,
        _dam: &mut Dam,
    ) {
        // no pending task in default impl
        unreachable!();
    }

    fn get_pending_task(&self) -> Option<&'static str> {
        None
    }

    fn display(
        &mut self,
        w: &mut W,
        screen: &Screen,
        state_area: Area,
        skin: &PanelSkin,
        con: &AppContext,
    ) -> Result<(), ProgramError>;

    /// return the flags to display
    fn get_flags(&self) -> Vec<Flag> {
        vec![]
    }

    fn get_starting_input(&self) -> String {
        String::new()
    }

    fn set_selected_path(&mut self, _path: PathBuf, _con: &AppContext) {
        // this function is useful for preview states
    }

    /// return the status which should be used when there's no verb edited
    fn no_verb_status(
        &self,
        has_previous_state: bool,
        con: &AppContext,
    ) -> Status;

    fn get_status(
        &self,
        cmd: &Command,
        other_path: &Option<PathBuf>,
        has_previous_state: bool,
        con: &AppContext,
    ) -> Status {
        match cmd {
            Command::PatternEdit{ .. } => self.no_verb_status(has_previous_state, con),
            Command::VerbEdit(invocation) => {
                if invocation.name.is_empty() {
                    Status::new(
                        "Type a verb then *enter* to execute it (*?* for the list of verbs)",
                        false,
                    )
                } else {
                    match con.verb_store.search(&invocation.name) {
                        PrefixSearchResult::NoMatch => {
                            Status::new("No matching verb (*?* for the list of verbs)", true)
                        }
                        PrefixSearchResult::Match(_, verb) => {
                            let selection = self.selection();
                            verb.get_status(selection, other_path, invocation)
                        }
                        PrefixSearchResult::Matches(completions) => Status::new(
                            format!(
                                "Possible verbs: {}",
                                completions
                                    .iter()
                                    .map(|c| format!("*{}*", c))
                                    .collect::<Vec<String>>()
                                    .join(", "),
                            ),
                            false,
                        ),
                    }
                }
            }
            _ => self.no_verb_status(has_previous_state, con),
        }
    }
}
