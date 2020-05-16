use {
    super::*,
    crate::{
        command::*,
        display::{status_line, Areas, Screen, W},
        errors::ProgramError,
        keys,
        skin::PanelSkin,
        task_sync::Dam,
        verb::*,
    },
    minimad::{Alignment, Composite},
    std::io::Write,
    termimad::{Event, InputField},
};

pub struct Panel {
    id: PanelId,
    states: Vec<Box<dyn AppState>>, // stack: the last one is current
    pub areas: Areas,
    status: Option<Status>,
    pub purpose: PanelPurpose,
    input_field: InputField,
}

impl Panel {
    pub fn new(
        id: PanelId,
        state: Box<dyn AppState>,
        areas: Areas,
    ) -> Self {
        let input_field = InputField::new(areas.input.clone());
        Self {
            id,
            states: vec![state],
            areas,
            status: None,
            purpose: PanelPurpose::None,
            input_field,
        }
    }

    pub fn set_error(&mut self, text: String) {
        self.status = Some(Status::from_error(text));
    }

    pub fn apply_command(
        &mut self,
        cmd: &Command,
        screen: &mut Screen,
        panel_skin: &PanelSkin,
        con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError> {
        let purpose = self.purpose;
        let result = self.mut_state().on_command(cmd, screen, panel_skin, con, purpose);
        self.status = Some(self.state().get_status(cmd, con));
        debug!("result in panel {:?}: {:?}", &self.id, &result);
        result
    }

    /// execute all the pending tasks until there's none remaining or
    ///  the dam asks for interruption
    pub fn do_pending_tasks(
        &mut self,
        w: &mut W,
        screen: &mut Screen,
        skin: &PanelSkin,
        con: &AppContext,
        dam: &mut Dam,
    ) -> Result<(), ProgramError> {
        while self.mut_state().get_pending_task().is_some() & !dam.has_event() {
            self.mut_state().do_pending_task(screen, dam);
            let is_active = true; // or we wouldn't do pending tasks
            self.display(w, is_active, screen, skin, con)?;
            w.flush()?;
        }
        Ok(())
    }

    /// return a new command
    /// Update the input field but not the command of the panel.
    pub fn add_event(
        &mut self,
        w: &mut W,
        event: Event,
        con: &AppContext,
    ) -> Result<Command, ProgramError> {
        let cmd = event::to_command(event, &mut self.input_field, con, &*self.states[self.states.len()-1]);
        self.input_field.display_on(w)?;
        Ok(cmd)
    }

    pub fn push_state(&mut self, new_state: Box<dyn AppState>) {
        self.states.push(new_state);
    }
    pub fn mut_state(&mut self) -> &mut dyn AppState {
        self.states.last_mut().unwrap().as_mut()
    }
    pub fn state(&self) -> &dyn AppState {
        self.states.last().unwrap().as_ref()
    }

    pub fn clear_input(&mut self) {
        self.input_field.set_content("");
    }

    pub fn set_input_content(&mut self, content: &str) {
        self.input_field.set_content(content);
    }

    pub fn set_input_arg(&mut self, arg: String) {
        let mut command_parts = CommandParts::from(&self.input_field.get_content());
        if let Some(invocation) = &mut command_parts.verb_invocation {
            invocation.args = Some(arg);
            let new_input = format!("{}", command_parts);
            self.input_field.set_content(&new_input);
        }
    }

    pub fn get_input_content(&self) -> String {
        self.input_field.get_content()
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

    pub fn display(
        &mut self,
        w: &mut W,
        active: bool,
        screen: &mut Screen,
        panel_skin: &PanelSkin,
        con: &AppContext,
    ) -> Result<(), ProgramError> {
        let state_area = self.areas.state.clone();
        self.mut_state().display(w, screen, state_area, panel_skin, con)?;
        self.write_status(w, panel_skin, screen)?;
        self.input_field.set_normal_style(panel_skin.styles.input.clone());
        self.input_field.focused = active;
        self.input_field.area = self.areas.input.clone();
        self.input_field.display_on(w)?;
        if active {
            self.write_purpose(w, panel_skin, screen, con)?;
        }
        Ok(())
    }

    fn write_status(
        &self,
        w: &mut W,
        panel_skin: &PanelSkin,
        screen: &Screen,
    ) -> Result<(), ProgramError> {
        let task = self.state().get_pending_task();
        lazy_static! {
            static ref DEFAULT_STATUS: Status = Status::from_message(
                "Hit *esc* to go back, *enter* to go up, *?* for help, or a few letters to search"
            );
        }
        let status = self.status.as_ref().unwrap_or(&*DEFAULT_STATUS);
        status_line::write(w, task, status, &self.areas.status, panel_skin, screen)
    }

    fn write_purpose(
        &self,
        w: &mut W,
        panel_skin: &PanelSkin,
        screen: &Screen,
        con: &AppContext,
    ) -> Result<(), ProgramError> {
        if !self.purpose.is_arg_edition() {
            return Ok(());
        }
        if let Some(area) = &self.areas.purpose {
            let shortcut = con.verb_store.verbs
                .iter()
                .filter(|v| match &v.execution {
                    VerbExecution::Internal(exec) => exec.internal == Internal::start_end_panel,
                    _ => false,
                })
                .filter_map(|v| v.keys.first())
                .map(|&k| keys::key_event_desc(k))
                .next()
                .unwrap_or(":start_end_panel".to_string());

            let md = format!("hit *{}* to fill arg ", shortcut);
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
