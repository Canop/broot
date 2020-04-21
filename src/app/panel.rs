
use {
    crate::{
        command::Command,
        errors::ProgramError,
        display::{
            Areas,
            Screen,
            Status,
            W,
        },
    },
    strict::NonEmptyVec,
    super::{
        AppContext,
        AppState,
        AppStateCmdResult,
    },
    termimad::{
        Event,
        InputField,
    },
};

pub struct Panel {
    //pub parent_panel_idx: Option<usize>, <- we must find an id
    states: NonEmptyVec<Box<dyn AppState>>, // stack: the last one is current
    pub areas: Areas,
    cmd: Command,
    status: Option<Status>, // FIXME Why an option ?
    input_field: InputField,
}

impl Panel {
    pub fn new(
        state: Box<dyn AppState>,
        areas: Areas,
        cmd: Command,
        screen: &Screen,
    ) -> Self {
        let mut input_field = InputField::new(areas.input.clone());
        input_field.set_normal_style(screen.skin.input.clone());
        Self {
            states: state.into(),
            areas,
            cmd,
            status: None,
            input_field,
        }
    }

    pub fn set_error(&mut self, text: String) {
        self.status = Some(Status::from_error(text));
    }

    pub fn apply_command(
        &mut self,
        mut cmd: Command,
        screen: &mut Screen,
        con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError> {
        let result = self.mut_state().apply(&mut cmd, screen, con);
        self.cmd = cmd;
        result
    }

    /// get a clone of the command which led to the current state
    pub fn get_command(&self) -> Command {
        self.cmd.clone()
    }

    /// return a new command, based on the previous one with the event
    /// added. Update the input field but not the command of the panel.
    pub fn add_event(
        &mut self,
        w: &mut W,
        event: Event,
        con: &AppContext,
    ) -> Result<Command, ProgramError> {
        let mut cmd = self.cmd.clone();
        let selection_type = self.state().selection_type();
        cmd.add_event(event, &mut self.input_field, con, selection_type);
        self.write_input_field(w, &cmd)?;
        Ok(cmd)
    }

    pub fn push(&mut self, new_state: Box<dyn AppState>) {
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
        // rétablir le write input pour les resize
        //self.write_input_field(w, &self.cmd)?;
        self.write_status(w, active, screen)?;
        Ok(())
    }

    /// can be called before the cmd of the panel is updated (for
    /// immediate rendering of the key press)
    pub fn write_input_field(
        &mut self,
        w: &mut W,
        cmd: &Command,
    ) -> Result<(), ProgramError> {
        self.input_field.set_content(&cmd.raw);
        self.input_field.display_on(w)?;
        Ok(())
    }

    fn write_status(
        &self,
        w: &mut W,
        active: bool,
        screen: &Screen,
    ) -> Result<(), ProgramError> {
        if let Some(status) = &self.status {
            status.display(w, &self.areas.status, screen)
        } else if active {
            lazy_static! {
                static ref DEFAULT_STATUS: Status = Status::from_message(
                    "Hit *esc* to go back, *enter* to go up, *?* for help, or a few letters to search"
                );
            }
            DEFAULT_STATUS.display(w, &self.areas.status, screen)
        } else {
            Status::erase(w, &self.areas.status, screen)
        }
    }

}
