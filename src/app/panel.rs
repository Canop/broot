use {
    super::*,
    crate::{
        command::*,
        display::{
            status_line,
            Areas,
            Screen,
            W,
            WIDE_STATUS,
            flags_display,
        },
        errors::ProgramError,
        keys::KEY_FORMAT,
        skin::PanelSkin,
        task_sync::Dam,
        verb::*,
    },
    termimad::{
        minimad::{Alignment, Composite},
        TimedEvent,
    },
};

/// A colon on screen containing a stack of states, the top
/// one being visible
pub struct Panel {
    pub id: PanelId,
    states: Vec<Box<dyn PanelState>>, // stack: the last one is current
    pub areas: Areas,
    status: Status,
    pub purpose: PanelPurpose,
    pub input: PanelInput,
}

impl Panel {

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
            input,
        }
    }

    pub fn set_error(&mut self, text: String) {
        self.status = Status::from_error(text);
    }
    pub fn set_message<S: Into<String>>(&mut self, md: S) {
        self.status = Status::from_message(md.into());
    }

    /// apply a command on the current state, with no
    /// effect on screen
    #[allow(clippy::too_many_arguments)] // a refactory could still be useful
    pub fn apply_command<'c>(
        &mut self,
        w: &'c mut W,
        cmd: &'c Command,
        app_state: &mut AppState,
        app_cmd_context: &'c AppCmdContext<'c>,
    ) -> Result<CmdResult, ProgramError> {
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
        self.status = self.state().get_status(app_state, &cc, has_previous_state, self.areas.status.width as usize);
        result
    }

    /// called on focusing the panel and before the display,
    /// this updates the status from the command read in the input
    pub fn refresh_input_status<'c>(
        &mut self,
        app_state: &AppState,
        app_cmd_context: &'c AppCmdContext<'c>,
    ) {
        let cmd = Command::from_raw(self.input.get_content(), false);
        let cc = CmdContext {
            cmd: &cmd,
            app: app_cmd_context,
            panel: PanelCmdContext {
                areas: &self.areas,
                purpose: self.purpose,
            },
        };
        let has_previous_state = self.states.len() > 1;
        self.status = self.state().get_status(app_state, &cc, has_previous_state, self.areas.status.width as usize);
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
        self.mut_state().do_pending_task(app_state, screen, con, dam)
    }

    pub fn has_pending_task(&self) -> bool {
        self.state().get_pending_task().is_some()
    }

    /// return a new command
    /// Update the input field
    pub fn add_event(
        &mut self,
        w: &mut W,
        event: TimedEvent,
        app_state: &AppState,
        con: &AppContext,
    ) -> Result<Command, ProgramError> {
        let sel_info = self.states[self.states.len() - 1].sel_info(app_state);
        let mode = self.state().get_mode();
        let panel_state_type = self.state().get_type();
        self.input.on_event(w, event, con, sel_info, app_state, mode, panel_state_type)
    }

    pub fn push_state(&mut self, new_state: Box<dyn PanelState>) {
        self.input.set_content(&new_state.get_starting_input());
        self.states.push(new_state);
    }
    pub fn mut_state(&mut self) -> &mut dyn PanelState {
        self.states.last_mut().unwrap().as_mut()
    }
    pub fn state(&self) -> &dyn PanelState {
        self.states.last().unwrap().as_ref()
    }

    pub fn clear_input(&mut self) {
        self.input.set_content("");
    }
    /// remove the verb invocation from the input but keep
    /// the filter if there's one
    pub fn clear_input_invocation(&mut self, con: &AppContext) {
        let mut command_parts = CommandParts::from(self.input.get_content());
        if command_parts.verb_invocation.is_some() {
            command_parts.verb_invocation = None;
            let new_input = format!("{command_parts}");
            self.input.set_content(&new_input);
        }
        self.mut_state().set_mode(con.initial_mode());
    }

    pub fn set_input_content(&mut self, content: &str) {
        self.input.set_content(content);
    }

    pub fn get_input_content(&self) -> String {
        self.input.get_content()
    }

    /// change the argument of the verb in the input, if there's one
    pub fn set_input_arg(&mut self, arg: String) {
        let mut command_parts = CommandParts::from(self.input.get_content());
        if let Some(invocation) = &mut command_parts.verb_invocation {
            invocation.args = Some(arg);
            let new_input = format!("{command_parts}");
            self.input.set_content(&new_input);
        }
    }

    /// return true when the element has been removed
    pub fn remove_state(&mut self) -> bool {
        if self.states.len() > 1 {
            self.states.pop();
            self.input.set_content(&self.state().get_starting_input());
            true
        } else {
            false
        }
    }

    /// render the whole panel (state, status, purpose, input, flags)
    pub fn display(
        &mut self,
        w: &mut W,
        disc: &DisplayContext,
    ) -> Result<(), ProgramError> {
        self.mut_state().display(w, disc)?;
        if disc.active || !WIDE_STATUS {
            self.write_status(w, disc.panel_skin, disc.screen)?;
        }
        let mut input_area = self.areas.input.clone();
        if disc.active {
            self.write_purpose(w, disc.panel_skin, disc.screen, disc.con)?;
            let flags = self.state().get_flags();
            let input_content_len = self.input.get_content().len() as u16;
            let flags_len = flags_display::visible_width(&flags);
            if input_area.width > input_content_len + 1 + flags_len {
                input_area.width -= flags_len + 1;
                disc.screen.goto(w, input_area.left + input_area.width, input_area.top)?;
                flags_display::write(w, &flags, disc.panel_skin)?;
            }
        }
        self.input.display(w, disc.active, self.state().get_mode(), input_area, disc.panel_skin)?;
        Ok(())
    }

    fn write_status(
        &self,
        w: &mut W,
        panel_skin: &PanelSkin,
        screen: Screen,
    ) -> Result<(), ProgramError> {
        let task = self.state().get_pending_task();
        status_line::write(
            w,
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
    fn write_purpose(
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
