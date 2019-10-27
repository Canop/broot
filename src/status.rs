
use termimad::CompoundStyle;

use crate::{
    app::W,
    elision::ElidedString,
    errors::ProgramError,
    screens::Screen,
    skin,
};

/// the status module manages writing information on the grey line
///  near the bottom of the screen
pub trait Status {
    fn write_status_text(&self, w: &mut W, text: &str) -> Result<(), ProgramError>;
    fn write_status_err(&self, w: &mut W, text: &str) -> Result<(), ProgramError>;
}

impl Screen {
    fn write_status(
        &self,
        w: &mut W,
        text: &str,
        skin: &CompoundStyle,
    ) -> Result<(), ProgramError> {
        let es = ElidedString::from(text, self.width as usize - 3); // -1 ? why isn't it -3 ?
        self.goto_clear(w, 2, self.height - 2)?;
        skin.queue_str(w, " ")?;
        for (i, part) in es.parts.iter().enumerate() {
            if i > 0 {
                self.skin.status_elision.queue_str(w, "…")?;
            }
            skin.queue_str(w, &part)?;
        }
        skin.queue_bg(w)?;
        self.clear_line(w)?;
        skin::reset(w)
    }
}

impl Status for Screen {
    fn write_status_err(&self, w: &mut W, text: &str) -> Result<(), ProgramError> {
        self.write_status(w, text, &self.skin.status_error)
    }
    fn write_status_text(&self, w: &mut W, text: &str) -> Result<(), ProgramError> {
        self.write_status(w, text, &self.skin.status_normal)
    }
}
