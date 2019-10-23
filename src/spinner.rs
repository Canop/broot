//! the thing which shows we're waiting for a long task
//! Executed during the do_pending_tasks of the states

use crate::{errors::ProgramError, screens::Screen, skin::SkinEntry};

pub trait Spinner {
    fn write_spinner(&mut self, spinning: bool) -> Result<(), ProgramError>;
}

impl Spinner for Screen {
    fn write_spinner(&mut self, spinning: bool) -> Result<(), ProgramError> {
        self.goto(1, self.h - 1);
        self.skin
            .spinner
            .print_string(if spinning { "⌛" } else { " " });
        Ok(())
    }
}
