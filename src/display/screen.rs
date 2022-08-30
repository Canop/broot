use {
    super::W,
    crate::{
        app::AppContext,
        errors::ProgramError,
        skin::PanelSkin,
    },
    crokey::crossterm::{
        cursor,
        terminal::{Clear, ClearType},
        QueueableCommand,
    },
    termimad::Area,
};

/// The dimensions of the screen
#[derive(Clone, Copy)]
pub struct Screen {
    pub width: u16,
    pub height: u16,
}

impl Screen {
    pub fn new(con: &AppContext) -> Result<Screen, ProgramError> {
        let mut screen = Screen {
            width: 0,
            height: 0,
        };
        screen.read_size(con)?;
        Ok(screen)
    }
    pub fn set_terminal_size(&mut self, w: u16, h: u16, con: &AppContext) {
        self.width = w;
        self.height = h;
        if let Some(h) = con.launch_args.height {
            self.height = h;
        }
    }
    pub fn read_size(&mut self, con: &AppContext) -> Result<(), ProgramError> {
        let (w, h) = termimad::terminal_size();
        self.set_terminal_size(w, h, con);
        Ok(())
    }
    /// move the cursor to x,y
    pub fn goto(self, w: &mut W, x: u16, y: u16) -> Result<(), ProgramError> {
        w.queue(cursor::MoveTo(x, y))?;
        Ok(())
    }
    /// clear from the cursor to the end of line
    pub fn clear_line(self, w: &mut W) -> Result<(), ProgramError> {
        w.queue(Clear(ClearType::UntilNewLine))?;
        Ok(())
    }
    /// clear the area and everything to the right.
    /// Should be used with parcimony as it could lead to flickering.
    pub fn clear_area_to_right(self, w: &mut W, area: &Area) -> Result<(), ProgramError> {
        for y in area.top..area.top + area.height {
            self.goto(w, area.left, y)?;
            self.clear_line(w)?;
        }
        Ok(())
    }
    /// just clears the char at the bottom right.
    /// (any redraw of this position makes the whole terminal flicker on some
    /// terminals like win/conemu, so we draw it only once at start of the
    /// app)
    pub fn clear_bottom_right_char(
        &self,
        w: &mut W,
        panel_skin: &PanelSkin,
    ) -> Result<(), ProgramError> {
        self.goto(w, self.width, self.height)?;
        panel_skin.styles.default.queue(w, ' ')?;
        Ok(())
    }
}
