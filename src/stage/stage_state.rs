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
    termimad::Area,
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

    /// option changing is unlikely to be done on this state, but
    /// we'll still do it in case a future scenario makes it possible
    /// to open a different state from this state
    fn with_new_options(
        &mut self,
        _screen: Screen,
        change_options: &dyn Fn(&mut TreeOptions),
        in_new_panel: bool,
        con: &AppContext,
    ) -> CmdResult {
        if in_new_panel {
            CmdResult::error("stage can't be displayed in two panels")
        } else {
            let mut new_options= self.tree_options();
            change_options(&mut new_options);
            CmdResult::NewState(Box::new(StageState::new(new_options, con)))
        }
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
        let width = area.width as usize;
        w.queue(cursor::MoveTo(area.left, 0))?;
        let mut cw = CropWriter::new(w, width);
        cw.queue_str(&styles.staging_area_title, "Staging Area")?;
        cw.fill(&styles.staging_area_title, &SPACE_FILLING)?;
        let list_area = Area::new(area.left, area.top + 1, area.width, area.height - 1);
        let list_height = list_area.height as usize;
        for idx in 0..list_height {
            let y = list_area.top + idx as u16;
            let stage_idx = idx; // + scroll
            w.queue(cursor::MoveTo(area.left, y))?;
            let mut cw = CropWriter::new(w, width);
            if let Some(path) = stage.paths.get(stage_idx) {
                cw.queue_g_string(
                    &styles.default,
                    path.to_string_lossy().to_string(),
                )?;
            }
            cw.fill(&styles.default, &SPACE_FILLING)?;
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

    fn execute_sequence(
        &mut self,
        _w: &mut W,
        _verb: &Verb,
        _seq_ex: &SequenceExecution,
        _invocation: Option<&VerbInvocation>,
        _app_state: &mut AppState,
        _cc: &CmdContext,
    ) -> Result<CmdResult, ProgramError> {
        Ok(CmdResult::error("sequence execution not yet implemented on staging area"))
    }
}

