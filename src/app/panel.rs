use {
    super::*,
    crate::{
        command::*,
        display::{
            Areas,
            Screen,
            W,
            status_line,
        },
        errors::ProgramError,
        keys::KEY_FORMAT,
        skin::PanelSkin,
        task_sync::Dam,
        verb::*,
    },
    termimad::{
        minimad::{
            Alignment,
            Composite,
        },
    },
};

/// A column on screen containing a stack of states, the top
/// one being visible
pub struct Panel {
    pub id: PanelId,
    states: Vec<Box<dyn PanelState>>, // stack: the last one is current
    pub areas: Areas,
    pub status: Status,
    pub purpose: PanelPurpose,
    pub last_raw_pattern: Option<String>,
}

impl Panel {
    #[must_use]
    pub fn new(
        id: PanelId,
        state: Box<dyn PanelState>,
        areas: Areas,
        con: &AppContext,
    ) -> Self {
        let mut input = PanelInput::new(areas.input.clone());
        input.set_content(&state.get_starting_input());
        let status = state.no_verb_status(false, con, areas.status.width as usize);
        Self {
            id,
            states: vec![state],
            areas,
            status,
            purpose: PanelPurpose::None,
            last_raw_pattern: None,
        }
    }

    pub fn set_error(
        &mut self,
        text: String,
    ) {
        self.status = Status::from_error(text);
    }
    pub fn set_message<S: Into<String>>(
        &mut self,
        md: S,
    ) {
        self.status = Status::from_message(md.into());
    }

    /// apply a command on the current state, with no
    /// effect on screen
    pub fn apply_command<'c>(
        &mut self,
        w: &'c mut W,
        cmd: &'c Command,
        app_state: &mut AppState,
        app_cmd_context: &'c AppCmdContext<'c>,
    ) -> Result<CmdResult, ProgramError> {
        if let Command::PatternEdit { raw, .. } = cmd {
            self.last_raw_pattern = Some(raw.clone());
        }
        let state_idx = self.states.len() - 1;
        let cc = CmdContext {
            cmd,
            app: app_cmd_context,
            panel: PanelCmdContext {
                areas: &self.areas,
                purpose: self.purpose,
            },
        };
        let result = self.states[state_idx].on_command(w, app_state, &cc);
        let has_previous_state = self.states.len() > 1;
        self.status = self.state().get_status(
            app_state,
            &cc,
            has_previous_state,
            self.areas.status.width as usize,
        );
        result
    }

    /// do the next pending task stopping as soon as there's an event
    /// in the dam
    pub fn do_pending_task(
        &mut self,
        app_state: &mut AppState,
        screen: Screen,
        con: &AppContext,
        dam: &mut Dam,
    ) -> Result<(), ProgramError> {
        self.mut_state()
            .do_pending_task(app_state, screen, con, dam)
    }

    #[must_use]
    pub fn has_pending_task(&self) -> bool {
        self.state().get_pending_task().is_some()
    }

    pub fn push_state(
        &mut self,
        new_state: Box<dyn PanelState>,
    ) {
        self.states.push(new_state);
    }
    #[must_use]
    pub fn mut_state(&mut self) -> &mut dyn PanelState {
        #[expect(
            clippy::missing_panics_doc,
            reason = "there's always at least one state"
        )]
        self.states.last_mut().unwrap().as_mut()
    }
    #[must_use]
    pub fn state(&self) -> &dyn PanelState {
        #[expect(
            clippy::missing_panics_doc,
            reason = "there's always at least one state"
        )]
        self.states.last().unwrap().as_ref()
    }

    /// return true when the element has been removed
    pub fn remove_state(&mut self) -> bool {
        if self.states.len() > 1 {
            self.states.pop();
            true
        } else {
            false
        }
    }

    pub fn write_status(
        &self,
        w: &mut W,
        watching: bool,
        panel_skin: &PanelSkin,
        screen: Screen,
    ) -> Result<(), ProgramError> {
        let task = self.state().get_pending_task();
        status_line::write(
            w,
            watching,
            task,
            &self.status,
            &self.areas.status,
            panel_skin,
            screen,
        )
    }

    /// if a panel has a specific purpose (i.e. is here for
    /// editing of the verb argument on another panel), render
    /// a hint of that purpose on screen
    pub fn write_purpose(
        &self,
        w: &mut W,
        panel_skin: &PanelSkin,
        screen: Screen,
        con: &AppContext,
    ) -> Result<(), ProgramError> {
        if !self.purpose.is_arg_edition() {
            return Ok(());
        }
        if let Some(area) = &self.areas.purpose {
            let shortcut = con
                .verb_store
                .verbs()
                .iter()
                .filter(|v| match &v.execution {
                    VerbExecution::Internal(exec) => exec.internal == Internal::start_end_panel,
                    _ => false,
                })
                .filter_map(|v| v.keys.first())
                .map(|&k| KEY_FORMAT.to_string(k))
                .next()
                .unwrap_or_else(|| ":start_end_panel".to_string());

            let md = format!("hit *{shortcut}* to fill arg ");
            // Add verbindex in purpose ?
            screen.goto(w, area.left, area.top)?;
            panel_skin.purpose_skin.write_composite_fill(
                w,
                Composite::from_inline(&md),
                area.width as usize,
                Alignment::Right,
            )?;
        }
        Ok(())
    }

}
