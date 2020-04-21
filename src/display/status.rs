use {
    crate::{
        display::{
            W,
        },
        errors::ProgramError,
    },
    minimad::{Alignment, Composite},
    std::io::Write,
    super::{
        Screen,
    },
    termimad::{
        Area,
        StyledChar,
    },
};

/// the status contains information written on the grey line
///  near the bottom of the screen
pub struct Status {
    pending_task: Option<&'static str>, // current pending_task
    message: String, // markdown
    error: bool, // is the current message an error?
}

impl Status {

    pub fn new<S: Into<String>>(
        pending_task: Option<&'static str>,
        message: S,
        error: bool,
    ) -> Status {
        Self {
            pending_task,
            message: message.into(),
            error,
        }
    }

    pub fn from_message<S: Into<String>>(message: S) -> Status {
        Self {
            pending_task: None,
            message: message.into(),
            error: false,
        }
    }

    pub fn from_error<S: Into<String>>(message: S) -> Status {
        Self {
            pending_task: None,
            message: message.into(),
            error: true,
        }
    }

    pub fn display(
        &self,
        w: &mut W,
        area: &Area,
        screen: &Screen,
    ) -> Result<(), ProgramError> {
        let y = area.top;
        //screen.goto_clear(w, area.left, y)?;
        screen.goto(w, area.left, y)?;
        let mut x = area.left;
        if let Some(pending_task) = self.pending_task {
            let pending_task = format!(" {}… ", pending_task);
            x += pending_task.chars().count() as u16;
            screen.skin.status_job.queue(w, pending_task)?;
        }
        screen.goto(w, x, y)?;
        let skin = if self.error {
            &screen.status_skin.error
        } else {
            &screen.status_skin.normal
        };
        skin.write_inline_on(w, " ")?;
        let remaining_width = (screen.width - x - 1) as usize ;
        skin.write_composite_fill(
            w,
            Composite::from_inline(&self.message),
            remaining_width,
            Alignment::Left,
        )?;
        //screen.clear_line(w)
        Ok(())
    }

    pub fn erase(
        w: &mut W,
        area: &Area,
        screen: &Screen,
    ) -> Result<(), ProgramError> {
        screen.goto(w, area.left, area.top)?;
        let sc = StyledChar::new(
            screen.status_skin.normal.paragraph.compound_style.clone(),
            ' ',
        );
        sc.queue_repeat(w, area.width as usize)?;
        Ok(())
    }

}
