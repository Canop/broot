
use {
    crate::{
        app::{
            AppContext,
        },
        errors::ProgramError,
        display::{
            Areas,
            Screen,
            W,
        },
    },
    strict::NonEmptyVec,
    super::{
        AppState,
    },
};


pub struct Panel {
    //pub parent_panel_idx: Option<usize>, <- we must find an id
    states: NonEmptyVec<Box<dyn AppState>>, // stack: the last one is current
    pub areas: Areas,
}

impl Panel {
    pub fn new(
        state: Box<dyn AppState>,
        areas: Areas,
    ) -> Self {
        Self {
            states: state.into(),
            areas,
        }
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
        screen: &mut Screen,
        con: &AppContext,
    ) -> Result<(), ProgramError> {
        let state_area = self.areas.state.clone();
        self.mut_state().display(w, screen, state_area, con)
    }
}
