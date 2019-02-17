//! the status module manages writing information on the grey line
//!  near the bottom of the screen

use std::io::{self, Write};
use termion;

use crate::screens::Screen;
use crate::skin::Skin;

pub trait Status {
    fn write_status_text(&mut self, text: &str) -> io::Result<()>;
    fn write_status_err(&mut self, text: &str) -> io::Result<()>;
}

impl Status for Screen {
    fn write_status_err(&mut self, text: &str) -> io::Result<()> {
        let y = self.h - 1;
        let mut text = String::from(text);
        text.truncate(self.w as usize - 2);
        write!(
            self.stdout,
            "{}{}{}{} {}{}",
            termion::cursor::Goto(2, y),
            self.skin.status_error.fg,
            self.skin.status_error.bg,
            termion::clear::CurrentLine,
            text,
            self.skin.reset.bg,
        )?;
        self.stdout.flush()?;
        Ok(())
    }
    fn write_status_text(&mut self, text: &str) -> io::Result<()> {
        let y = self.h - 1;
        let mut text = String::from(text);
        text.truncate(self.w as usize - 2);
        write!(
            self.stdout,
            "{}{}{}{} {}{}",
            termion::cursor::Goto(2, y),
            self.skin.status_normal.fg,
            self.skin.status_normal.bg,
            termion::clear::CurrentLine,
            text,
            self.skin.reset.bg,
        )?;
        self.stdout.flush()?;
        Ok(())
    }
}
