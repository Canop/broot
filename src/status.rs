
use crate::{
    elision::ElidedString,
    errors::ProgramError,
    io::W,
    screens::Screen,
    skin,
};

/// the status contains information written on the grey line
///  near the bottom of the screen
pub struct Status {
    pending_task: Option<&'static str>, // current pending_task
    message: String,
    error: bool, // is the current message an error?
}

impl Status {

    pub fn from_message<S: Into<String>>(message: S) -> Self {
        Self {
            pending_task: None,
            message: message.into(),
            error: false,
        }
    }

    pub fn from_error<S: Into<String>>(error: S) -> Self {
        Self {
            pending_task: None,
            message: error.into(),
            error: true,
        }
    }

    pub fn set_pending_task(
        &mut self,
        pending_task: &'static str,
    ) {
        self.pending_task = Some(pending_task);
    }

    pub fn has_pending_task(
        &self,
    ) -> bool {
        self.pending_task.is_some()
    }

    pub fn display(
        &self,
        w: &mut W,
        screen: &Screen,
    ) -> Result<(), ProgramError> {
        let y = screen.height - 2;
        screen.goto_clear(w, 0, y)?;
        let mut x = 0;
        if let Some(pending_task) = self.pending_task {
            let pending_task = format!(" {} ", pending_task);
            x += pending_task.len();
            screen.skin.status_job.queue(w, pending_task)?;
        }
        screen.goto(w, x as u16, y)?;
        let skin = if self.error {
            &screen.skin.status_error
        } else {
            &screen.skin.status_normal
        };
        let es = ElidedString::from(&self.message, screen.width as usize - x - 1);
        skin.queue_str(w, " ")?;
        for (i, part) in es.parts.iter().enumerate() {
            if i > 0 {
                screen.skin.status_elision.queue_str(w, "…")?;
            }
            skin.queue_str(w, &part)?;
        }
        skin.queue_bg(w)?;
        screen.clear_line(w)?;
        skin::reset(w) // FIXME check it's necessary
    }

}

