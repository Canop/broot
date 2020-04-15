use {
    crate::{
        errors::ProgramError,
    },
    minimad::{Alignment, Composite},
    std::io::Write,
    super::{
        Screen,
    },
};

/// the status contains information written on the grey line
///  near the bottom of the screen
pub struct Status<'a> {
    pending_task: Option<&'static str>, // current pending_task
    message: Composite<'a>,
    error: bool, // is the current message an error?
}

impl<'a> Status<'a> {

    pub fn new(
        pending_task: Option<&'static str>,
        message: Composite<'a>,
        error: bool,
    ) -> Status<'a> {
        Self {
            pending_task,
            message,
            error,
        }
    }

    pub fn from_message(message: Composite<'a>) -> Status<'a> {
        Self {
            pending_task: None,
            message,
            error: false,
        }
    }

    pub fn from_error(message: Composite<'a>) -> Status<'a> {
        Self {
            pending_task: None,
            message,
            error: true,
        }
    }

    pub fn display(self, w: &mut impl Write, screen: &Screen) -> Result<(), ProgramError> {
        let y = screen.height - 2;
        screen.goto_clear(w, 0, y)?;
        let mut x = 0;
        if let Some(pending_task) = self.pending_task {
            let pending_task = format!(" {}… ", pending_task);
            x += pending_task.chars().count();
            screen.skin.status_job.queue(w, pending_task)?;
        }
        screen.goto(w, x as u16, y)?;
        let skin = if self.error {
            &screen.status_skin.error
        } else {
            &screen.status_skin.normal
        };
        skin.write_inline_on(w, " ")?;
        let remaining_width = screen.width as usize - x - 1;
        skin.write_composite_fill(w, self.message, remaining_width, Alignment::Left)?;
        screen.clear_line(w)
    }
}
