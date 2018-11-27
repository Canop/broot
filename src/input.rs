

use std::io::{self, Write};

use app::Screen;
use commands::{Command};

pub trait Input {
    fn write_input(&mut self, cmd: &Command) -> io::Result<()>;
}

impl Input for Screen {
    fn write_input(&mut self, cmd: &Command) -> io::Result<()> {
        write!(
            self.stdout,
            "{}{}{}{} {}",
            termion::cursor::Goto(1, self.h),
            termion::clear::CurrentLine,
            cmd.raw,
            termion::style::Invert,
            termion::style::NoInvert,
        )?;
        self.stdout.flush()?;
        Ok(())
    }
}

