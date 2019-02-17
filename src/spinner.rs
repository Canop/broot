//! the thing which shows we're waiting for a long task
//! Executed during the do_pending_tasks of the states

use std::io::{self, Write};

use crate::screens::Screen;

pub trait Spinner {
    fn write_spinner(&mut self, spinning: bool) -> io::Result<()>;
}

impl Spinner for Screen {
    fn write_spinner(&mut self, spinning: bool) -> io::Result<()> {
        let y = self.h - 1;
        write!(
            self.stdout,
            "{}{}{}{}",
            termion::cursor::Goto(1, y),
            self.skin.spinner.fg,
            self.skin.spinner.bg,
            if spinning { "⌛" } else { " " },
        )?;
        self.stdout.flush()?;
        Ok(())
    }
}
