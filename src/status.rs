use std::io;

use crate::screens::Screen;
use crate::skin::{self, SkinEntry};
use crossterm::ObjectStyle;

/// the status module manages writing information on the grey line
///  near the bottom of the screen
pub trait Status {
    fn write_status_text(&self, text: &str) -> io::Result<()>;
    fn write_status_err(&self, text: &str) -> io::Result<()>;
}

impl Screen {
    fn write_status(&self, text: &str, skin: &ObjectStyle) -> io::Result<()> {
        let mut text = String::from(text);
        text.truncate(self.w as usize - 3);
        self.goto_clear(2, self.h - 1);
        skin.print_string(" ");
        skin.print_string(&text);
        skin.print_bg();
        self.clear_line();
        skin::reset();
        Ok(())
    }
}

impl Status for Screen {
    fn write_status_err(&self, text: &str) -> io::Result<()> {
        self.write_status(text, &self.skin.status_error)
    }
    fn write_status_text(&self, text: &str) -> io::Result<()> {
        self.write_status(text, &self.skin.status_normal)
    }
}
