use {
    crate::{
        app::*,
        command::{Command, TriggerType},
        display::{CropWriter, Screen, SPACE_FILLING, W},
        errors::ProgramError,
        pattern::*,
        tree::*,
        verb::*,
    },
    crossterm::{
        cursor,
        QueueableCommand,
    },
    std::path::{Path},
};

pub struct StageState {
    tree_options: TreeOptions,
    mode: Mode,
}

impl StageState {

    pub fn new(
        tree_options: TreeOptions,
        con: &AppContext,
    ) -> StageState {
        Self {
            tree_options,
            mode: initial_mode(con),
        }
    }
}

impl PanelState for StageState {

    fn get_type(&self) -> PanelStateType {
        PanelStateType::Stage
    }

    fn get_pending_task(&self) -> Option<&'static str> {
        None
    }

    fn selected_path(&self) -> Option<&Path> {
        None
    }

    fn selection(&self) -> Option<Selection<'_>> {
        None
    }

    fn sel_info<'c>(&'c self, app_state: &'c AppState) -> SelInfo<'c> {
        match app_state.stage.paths.len() {
            0 => SelInfo::None,
            1 => SelInfo::One(Selection {
                path: &app_state.stage.paths[0],
                stype: SelectionType::File,
                is_exe: false,
                line: 0,
            }),
            _ => SelInfo::More(&app_state.stage),
        }
    }

    fn has_at_least_one_selection(&self, app_state: &AppState) -> bool {
        !app_state.stage.is_empty()
    }

    fn tree_options(&self) -> TreeOptions {
        self.tree_options.clone()
    }

    fn with_new_options(
        &mut self,
        screen: Screen,
        change_options: &dyn Fn(&mut TreeOptions),
        in_new_panel: bool,
        con: &AppContext,
    ) -> CmdResult {
        // TODO implement: sorting, etc.
        CmdResult::Keep
    }

    fn on_pattern(
        &mut self,
        pat: InputPattern,
        _con: &AppContext,
    ) -> Result<CmdResult, ProgramError> {
        // TODO implement
        Ok(CmdResult::Keep)
    }

    fn display(
        &mut self,
        w: &mut W,
        disc: &DisplayContext,
    ) -> Result<(), ProgramError> {
        let stage = &disc.app_state.stage;
        let area = &disc.state_area;
        let styles = &disc.panel_skin.styles;
        let line_count = area.height as usize;
        let width = area.width as usize;
        for y in 0..line_count {
            w.queue(cursor::MoveTo(area.left, y as u16 + area.top))?;
            let mut cw = CropWriter::new(w, width);
            if let Some(path) = stage.paths.get(y) {
                cw.queue_g_string(
                    &styles.default,
                    path.to_string_lossy().to_string(),
                )?;
            }
            cw.fill(
                &styles.default,
                &SPACE_FILLING,
            )?;
        }
        Ok(())
    }

    fn refresh(&mut self, _screen: Screen, _con: &AppContext) -> Command {
        Command::empty()
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    fn get_mode(&self) -> Mode {
        self.mode
    }

    fn on_internal(
        &mut self,
        w: &mut W,
        internal_exec: &InternalExecution,
        input_invocation: Option<&VerbInvocation>,
        trigger_type: TriggerType,
        app_state: &mut AppState,
        cc: &CmdContext,
    ) -> Result<CmdResult, ProgramError> {
        Ok(match internal_exec.internal {
            _ => self.on_internal_generic(
                w,
                internal_exec,
                input_invocation,
                trigger_type,
                app_state,
                cc,
            )?,
        })
    }

    /// the stage state deals with a multiple selection, which means
    /// the tests on what can be executed are different, and the verb
    /// display is different too
    fn get_verb_status(
        &self,
        verb: &Verb,
        invocation: &VerbInvocation,
        app_state: &AppState,
        cc: &CmdContext,
    ) -> Status {
        if app_state.stage.paths.len() > 1 {
            if let VerbExecution::External(external) = &verb.execution {
                if external.exec_mode != ExternalExecutionMode::StayInBroot {
                    return Status::new(
                        "only verbs returning to broot on end can be executed on a multi-selection".to_owned(),
                        true,
                    );
                }
            }
            // right now there's no check for sequences but they're inherently dangereous
        }
        if app_state.stage.is_empty() {
            if let Some(err) = verb.check_args(&None, invocation, &cc.app.other_path) {
                return Status::new(err, true);
            }
        } else {
            // we check that the verb applies to all selections
            // TODO make it faster when the verb doesn't need the selection ?
            for path in &app_state.stage.paths {
                let selection = Selection {
                    path,
                    line: 0,
                    stype: SelectionType::from(path),
                    is_exe: false,
                };
                if let Some(err) = verb.check_args(&Some(selection), invocation, &cc.app.other_path) {
                    return Status::new(err, true);
                }
            }
        }
        Status::new(
            verb.get_status_markdown(
                None,
                &cc.app.other_path,
                invocation,
            ),
            false,
        )
    }

    fn execute_external(
        &mut self,
        w: &mut W,
        verb: &Verb,
        external_execution: &ExternalExecution,
        invocation: Option<&VerbInvocation>,
        app_state: &mut AppState,
        cc: &CmdContext,
    ) -> Result<CmdResult, ProgramError> {
        if app_state.stage.paths.len() > 1 {
            if external_execution.exec_mode != ExternalExecutionMode::StayInBroot {
                return Ok(CmdResult::error(
                    "only verbs returning to broot on end can be executed on a multi-selection".to_owned()
                ));
            }
        }
        if app_state.stage.is_empty() {
            // execution on no selection
            let exec_builder = ExecutionStringBuilder::from_invocation(
                &verb.invocation_parser,
                None,
                &cc.app.other_path,
                if let Some(inv) = invocation {
                    &inv.args
                } else {
                    &None
                },
            );
            external_execution.to_cmd_result(w, exec_builder, &cc.app.con)
        } else {
            let mut refresh = false;
            // we apply the verb to all selections
            for path in &app_state.stage.paths {
                let selection = Selection {
                    path,
                    line: 0,
                    stype: SelectionType::from(path),
                    is_exe: false,
                };
                let exec_builder = ExecutionStringBuilder::from_invocation(
                    &verb.invocation_parser,
                    Some(selection),
                    &cc.app.other_path,
                    if let Some(inv) = invocation {
                        &inv.args
                    } else {
                        &None
                    },
                );
                match external_execution.to_cmd_result(w, exec_builder, &cc.app.con)? {
                    CmdResult::Keep => {}
                    CmdResult::RefreshState { .. } => {
                        refresh = true;
                    }
                    cr => {
                        return Ok(CmdResult::error(format!("unexpected execution result: {:?}", cr)));
                    }
                }
            }
            Ok(if refresh {
                CmdResult::RefreshState { clear_cache: true }
            } else {
                CmdResult::Keep
            })
        }
    }

    fn execute_sequence(
        &mut self,
        w: &mut W,
        verb: &Verb,
        seq_ex: &SequenceExecution,
        invocation: Option<&VerbInvocation>,
        app_state: &mut AppState,
        cc: &CmdContext,
    ) -> Result<CmdResult, ProgramError> {
        Ok(CmdResult::error("sequence execution not yet implemented on staging area"))
    }
}

