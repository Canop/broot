use {
    crate::{
        command::Command,
        display::{
            Screen,
            Status,
            W,
        },
        errors::ProgramError,
        selection_type::SelectionType,
        task_sync::Dam,
    },
    super::*,
    termimad::Area,
};

/// a whole application state, stackable to allow reverting
///  to a previous one
pub trait AppState {
    fn apply(
        &mut self,
        cmd: &mut Command,
        screen: &mut Screen,
        con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError>;

    //fn can_execute(&self, verb_index: usize, con: &AppContext) -> bool;

    fn selection_type(&self) -> SelectionType;

    fn refresh(
        &mut self,
        screen: &Screen,
        con: &AppContext,
    ) -> Command;

    fn do_pending_task(&mut self, screen: &mut Screen, dam: &mut Dam);

    fn has_pending_task(&self) -> bool;

    fn display(
        &mut self,
        w: &mut W,
        screen: &Screen,
        state_area: Area,
        con: &AppContext,
    ) -> Result<(), ProgramError>;

    fn write_flags(
        &self,
        w: &mut W,
        screen: &mut Screen,
        con: &AppContext,
    ) -> Result<(), ProgramError>;

    fn get_status(
        &self,
        cmd: &Command,
        con: &AppContext,
    ) -> Status;
}
