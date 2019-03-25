//! the thing which shows we're waiting for a long task
//! Executed during the do_pending_tasks of the states

use std::io::{self};

use crate::screens::Screen;

pub trait Spinner {
    fn write_spinner(&mut self, spinning: bool) -> io::Result<()>;
}

impl Spinner for Screen {
    fn write_spinner(&mut self, spinning: bool) -> io::Result<()> {
        let y = self.h - 1;
        self.write(&format!(
            "{}{}{}{}",
            termion::cursor::Goto(1, y),
            self.skin.spinner.fg,
            self.skin.spinner.bg,
            if spinning { "⌛" } else { " " },
        ));
        Ok(())
    }
}
