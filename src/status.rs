

use std::io::{self, Write};

use app::App;

pub trait Status {
    fn write_status(&mut self, text: &str) -> io::Result<()>;
}

impl Status for App {
    fn write_status(&mut self, text: &str) -> io::Result<()> {
        let y = self.h-1;
        write!(
            self.stdout,
            "{}{}{}",
            termion::cursor::Goto(1, y),
            termion::clear::CurrentLine,
            text
        )?;
        Ok(())
    }
}

