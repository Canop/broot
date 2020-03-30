use {
    crate::{
        command::Command,
        errors::ProgramError,
        screens::Screen,
        task_sync::Dam,
    },
    std::io::Write,
    super::*,
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

    fn can_execute(&self, verb_index: usize, con: &AppContext) -> bool;

    fn refresh(&mut self, screen: &Screen, con: &AppContext) -> Command;

    fn do_pending_task(&mut self, screen: &mut Screen, dam: &mut Dam);

    fn has_pending_task(&self) -> bool;

    fn display(
        &mut self,
        w: &mut dyn Write,
        screen: &Screen,
        con: &AppContext,
    ) -> Result<(), ProgramError>;

    fn write_flags(
        &self,
        w: &mut dyn Write,
        screen: &mut Screen,
        con: &AppContext,
    ) -> Result<(), ProgramError>;

    fn write_status(
        &self,
        w: &mut dyn Write,
        cmd: &Command,
        screen: &Screen,
        con: &AppContext,
    ) -> Result<(), ProgramError>;
}
