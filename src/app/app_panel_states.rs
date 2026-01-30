use crate::app::{
    PanelId,
    PanelReference,
    PanelState,
    PanelStateType,
};

#[derive(Clone)]
pub struct AppPanelStatesEntry<'a> {
    pub panel_id: PanelId,
    pub state: &'a dyn PanelState,
}

#[derive(Clone)]
pub struct AppPanelStates<'a> {
    pub entries: Vec<AppPanelStatesEntry<'a>>,
    pub active_panel_idx: usize, // guaranteed to be < states.len()
}

impl AppPanelStates<'_> {
    #[must_use]
    pub fn active(&self) -> &dyn PanelState {
        self.entries[self.active_panel_idx].state
    }
    pub fn by_id(
        &self,
        panel_id: PanelId,
    ) -> Option<&dyn PanelState> {
        self.entries
            .iter()
            .find(|&entry| entry.panel_id == panel_id)
            .map(|entry| entry.state)
    }
    pub fn by_type(
        &self,
        state_type: PanelStateType,
    ) -> Option<&dyn PanelState> {
        self.entries
            .iter()
            .find(|&entry| entry.state.get_type() == state_type)
            .map(|entry| entry.state)
    }
    pub fn by_ref(
        &self,
        panel_ref: PanelReference,
    ) -> Option<&dyn PanelState> {
        match panel_ref {
            PanelReference::Active => Some(self.active()),
            PanelReference::Leftest => self.entries.first().map(|e| e.state),
            PanelReference::Rightest => self.entries.last().map(|e| e.state),
            PanelReference::Idx(idx) => self.entries.get(idx).map(|e| e.state),
            PanelReference::Id(id) => self.by_id(id),
            PanelReference::Preview => self.by_type(PanelStateType::Preview),
        }
    }
}
