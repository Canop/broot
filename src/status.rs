
use crossterm::ObjectStyle;

use crate::{
    elision::ElidedString,
    errors::ProgramError,
    screens::Screen,
    skin::{self, SkinEntry},
};

/// the status module manages writing information on the grey line
///  near the bottom of the screen
pub trait Status {
    fn write_status_text(&self, text: &str) -> Result<(), ProgramError>;
    fn write_status_err(&self, text: &str) -> Result<(), ProgramError>;
}

impl Screen {
    fn write_status(&self, text: &str, skin: &ObjectStyle) -> Result<(), ProgramError> {
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
    fn write_status_err(&self, text: &str) -> Result<(), ProgramError> {
        self.write_status(text, &self.skin.status_error)
    }
    fn write_status_text(&self, text: &str) -> Result<(), ProgramError> {
        self.write_status(text, &self.skin.status_normal)
    }
}
