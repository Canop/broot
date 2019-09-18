use std::io;

use crate::elision::ElidedString;
use crate::screens::Screen;
use crate::skin::{self, SkinEntry};
use crossterm_style::ObjectStyle;

/// the status module manages writing information on the grey line
///  near the bottom of the screen
pub trait Status {
    fn write_status_text(&self, text: &str) -> io::Result<()>;
    fn write_status_err(&self, text: &str) -> io::Result<()>;
}

impl Screen {
    fn write_status(&self, text: &str, skin: &ObjectStyle) -> io::Result<()> {
        let es = ElidedString::from(text, self.w as usize - 1); // why isn't it -3 ?
        self.goto_clear(2, self.h - 1);
        skin.print_string(" ");
        for (i, part) in es.parts.iter().enumerate() {
            if i > 0 {
                self.skin.status_elision.print_string("…");
            }
            skin.print_string(&part);
        }
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

