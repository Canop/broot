
use {
    strict::NonEmptyVec,
    super::{
        AppState,
    },
};


pub struct StatePanel {
    //pub parent_panel_idx: Option<usize>, <- we must find an id
    states: NonEmptyVec<Box<dyn AppState>>, // stack: the last one is current
}

impl StatePanel {
    pub fn new(
        state: Box<dyn AppState>,
    ) -> Self {
        Self {
            states: state.into(),
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
}
