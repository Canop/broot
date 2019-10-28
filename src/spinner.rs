//! the thing which shows we're waiting for a long task
//! Executed during the do_pending_tasks of the states

use crate::{
    app::W,
    errors::ProgramError,
    screens::Screen,
};

pub trait Spinner {
    fn write_spinner(&mut self, w: &mut W, spinning: bool) -> Result<(), ProgramError>;
}

impl Spinner for Screen {
    fn write_spinner(&mut self, w: &mut W, spinning: bool) -> Result<(), ProgramError> {
        self.goto(w, 0, self.height - 2)?;
        self.skin.spinner.queue_str(w, if spinning { "⌛" } else { "  " })?;
        Ok(())
    }
}
