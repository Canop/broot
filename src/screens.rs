use std::io::Write;

use crossterm::{
    cursor,
    terminal::{Clear, ClearType},
    queue,
};
use termimad::{Area, CompoundStyle, InputField};

use crate::{
    app_context::AppContext,
    errors::ProgramError,
    io::W,
    mad_skin::StatusMadSkinSet,
    skin::Skin,
};

pub struct Screen {
    pub width: u16,
    pub height: u16,
    pub skin: Skin,
    pub input_field: InputField,
    pub status_skin: StatusMadSkinSet,
}

impl Screen {
    pub fn new(con: &AppContext, skin: Skin) -> Result<Screen, ProgramError> {
        let mut input_field = InputField::new(Area::new(0, 0, 10, 1));
        input_field.set_normal_style(CompoundStyle::from(skin.input.clone()));
        let status_skin = StatusMadSkinSet::from_skin(&skin);
        let mut screen = Screen {
            width: 0,
            height: 0,
            skin,
            input_field,
            status_skin,
        };
        screen.read_size(con)?;
        Ok(screen)
    }
    pub fn read_size(&mut self, con: &AppContext) -> Result<(), ProgramError> {
        let (w, h) = termimad::terminal_size();
        self.width = w;
        self.height = h;
        if let Some(h) = con.launch_args.height {
            self.height = h;
        }
        debug!("screen size: {} x {}", self.width, self.height);
        self.input_field.change_area(0, h-1, w - 15);
        debug!("input_field area: {:?}", self.input_field.area);
        Ok(())
    }
    /// move the cursor to x,y and clears the line.
    pub fn goto_clear(&self, w: &mut W, x: u16, y: u16)
    -> Result<(), ProgramError> {
        self.goto(w, x, y)?;
        self.clear_line(w)
    }
    /// move the cursor to x,y
    pub fn goto(
        &self,
        w: &mut W,
        x: u16,
        y: u16
    ) -> Result<(), ProgramError> {
        queue!(w, cursor::MoveTo(x, y))?;
        Ok(())
    }
    /// clear the whole screen
    pub fn clear(&self, w: &mut W) -> Result<(), ProgramError> {
        queue!(w, Clear(ClearType::All))?;
        Ok(())
    }
    /// clear from the cursor to the end of line
    pub fn clear_line(&self, w: &mut W) -> Result<(), ProgramError> {
        queue!(w, Clear(ClearType::UntilNewLine))?;
        Ok(())
    }
}
