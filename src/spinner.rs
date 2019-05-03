//! the thing which shows we're waiting for a long task
//! Executed during the do_pending_tasks of the states

use std::io::{self};

use crate::screens::Screen;

pub trait Spinner {
    fn write_spinner(&mut self, spinning: bool) -> io::Result<()>;
}

impl Spinner for Screen {
    fn write_spinner(&mut self, spinning: bool) -> io::Result<()> {
        self.goto_clear(1, self.h-1);
        self.write(&format!(
            "{}",
            self.skin.spinner.apply_to(
                if spinning { "⌛" } else { " " }
            ),
        ));
        Ok(())
    }
}
