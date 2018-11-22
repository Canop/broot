

use std::io::{self, Write};

use app::Screen;
use commands::{Action, Command};

pub trait Input {
    fn writeInput(&mut self, cmd: &Command) -> io::Result<()>;
}

impl Input for Screen {
    fn writeInput(&mut self, cmd: &Command) -> io::Result<()> {
        write!(
            self.stdout,
            "{}{}{}",
            termion::cursor::Goto(1, self.h),
            termion::clear::CurrentLine,
            cmd.raw,
        )?;
        self.stdout.flush()?;
        Ok(())
    }
}

