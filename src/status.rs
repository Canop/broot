//! the status module manages writing information on the grey line
//!  near the bottom of the screen

use std::io::{self};

use crossterm::{Attribute::{self, Reset}, Color::{self, *}, Colored, Color::AnsiValue};

use crate::screens::Screen;
use crate::skin::{self, SkinEntry};

pub trait Status {
    fn write_status_text(&self, text: &str) -> io::Result<()>;
    fn write_status_err(&self, text: &str) -> io::Result<()>;
}

impl Screen {
    fn write_status(&self, text: &str, skin: &dyn SkinEntry) -> io::Result<()> {
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
