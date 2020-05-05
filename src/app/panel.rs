use {
    super::{AppContext, AppState, AppStateCmdResult, PanelId, Status},
    crate::{
        command::{event, Command},
        display::{status_line, Areas, Screen, W},
        errors::ProgramError,
        task_sync::Dam,
    },
    std::io::Write,
    strict::NonEmptyVec,
    termimad::{Event, InputField},
};

pub struct Panel {
    id: PanelId,
    //pub parent_panel_id: Option<usize>,
    states: NonEmptyVec<Box<dyn AppState>>, // stack: the last one is current
    pub areas: Areas,
    status: Option<Status>, // FIXME Why an option ?
    input_field: InputField,
}

impl Panel {
    pub fn new(id: PanelId, state: Box<dyn AppState>, areas: Areas, screen: &Screen) -> Self {
        let mut input_field = InputField::new(areas.input.clone());
        input_field.set_normal_style(screen.skin.input.clone());
        Self {
            id,
            // parent_panel_id: None,
            states: state.into(),
            areas,
            status: None,
            input_field,
        }
    }

    pub fn clear_input(&mut self) {
        self.input_field.set_content("");
    }

    pub fn set_error(&mut self, text: String) {
        self.status = Some(Status::from_error(text));
    }

    pub fn apply_command(
        &mut self,
        cmd: &Command,
        screen: &mut Screen,
        con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError> {
        let result = self.mut_state().on_command(cmd, screen, con);
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
        con: &AppContext,
        dam: &mut Dam,
    ) -> Result<(), ProgramError> {
        while self.mut_state().get_pending_task().is_some() & !dam.has_event() {
            self.mut_state().do_pending_task(screen, dam);
            //self.status = Some(self.state().get_status(&self.cmd, con));
            let is_active = true; // or we wouldn't do pending tasks
            self.display(w, is_active, screen, con)?;
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
        let selection_type = self.state().selection_type();
        let cmd = event::to_command(event, &mut self.input_field, con, selection_type);
        self.input_field.display_on(w)?;
        Ok(cmd)
    }

    pub fn push_state(&mut self, new_state: Box<dyn AppState>) {
        self.states.push(new_state);
    }
    pub fn mut_state(&mut self) -> &mut dyn AppState {
        self.states.last_mut().as_mut()
    }
    pub fn state(&self) -> &dyn AppState {
        self.states.last().as_ref()
    }

    /// return true when the element has been removed
    pub fn remove_state(&mut self) -> bool {
        self.states.pop().is_some()
    }

    pub fn display(
        &mut self,
        w: &mut W,
        active: bool,
        screen: &mut Screen,
        con: &AppContext,
    ) -> Result<(), ProgramError> {
        let state_area = self.areas.state.clone();
        self.mut_state().display(w, screen, state_area, con)?;
        self.input_field.focused = active;
        self.input_field.area = self.areas.input.clone();
        self.input_field.display_on(w)?;
        self.write_status(w, active, screen)?;
        Ok(())
    }

    fn write_status(&self, w: &mut W, _active: bool, screen: &Screen) -> Result<(), ProgramError> {
        let task = self.state().get_pending_task();
        lazy_static! {
            static ref DEFAULT_STATUS: Status = Status::from_message(
                "Hit *esc* to go back, *enter* to go up, *?* for help, or a few letters to search"
            );
        }
        let status = self.status.as_ref().unwrap_or(&*DEFAULT_STATUS);
        status_line::write(w, task, status, &self.areas.status, screen)
    }
}
