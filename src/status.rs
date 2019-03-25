//! the status module manages writing information on the grey line
//!  near the bottom of the screen

use std::io::{self};
use termion;

use crate::screens::Screen;

pub trait Status {
    fn write_status_text(&mut self, text: &str) -> io::Result<()>;
    fn write_status_err(&mut self, text: &str) -> io::Result<()>;
}

impl Screen {
    fn write_status(&mut self, text: &str, error: bool) -> io::Result<()> {
        let skin = if error {
            &self.skin.status_error
        } else {
            &self.skin.status_normal
        };
        let mut text = String::from(text);
        text.truncate(self.w as usize - 2);
        self.write(&format!(
            "{}{}{}{} {}{}",
            termion::cursor::Goto(2, self.h - 1),
            skin.fg,
            skin.bg,
            termion::clear::CurrentLine,
            text,
            self.skin.reset.bg,
        ));
        Ok(())
    }
}

impl Status for Screen {
    fn write_status_err(&mut self, text: &str) -> io::Result<()> {
        self.write_status(text, true)
    }

    fn write_status_text(&mut self, text: &str) -> io::Result<()> {
        self.write_status(text, false)
    }
}
