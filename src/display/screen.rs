use {
    crate::{
        app::AppContext,
        errors::ProgramError,
        skin::Skin,
        skin::{self, StatusMadSkinSet},
    },
    crossterm::{
        cursor,
        terminal::{Clear, ClearType},
        QueueableCommand,
    },
    super::W,
    termimad::{
        Area,
        MadSkin,
    },
};

pub struct Screen {
    pub width: u16,
    pub height: u16,
    pub skin: Skin,
    pub status_skin: StatusMadSkinSet,
    pub help_skin: MadSkin,
}

impl Screen {
    pub fn new(con: &AppContext, skin: Skin) -> Result<Screen, ProgramError> {
        let status_skin = StatusMadSkinSet::from_skin(&skin);
        let help_skin = skin::make_help_mad_skin(&skin);
        let mut screen = Screen {
            width: 0,
            height: 0,
            skin,
            status_skin,
            help_skin,
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
    /// move the cursor to x,y and clears the line.
    pub fn goto_clear(&self, w: &mut W, x: u16, y: u16) -> Result<(), ProgramError> {
        self.goto(w, x, y)?;
        self.clear_line(w)
    }
    /// move the cursor to x,y
    pub fn goto(&self, w: &mut W, x: u16, y: u16) -> Result<(), ProgramError> {
        w.queue(cursor::MoveTo(x, y))?;
        Ok(())
    }
    /// clear the whole screen
    pub fn clear(&self, w: &mut W) -> Result<(), ProgramError> {
        w.queue(Clear(ClearType::All))?;
        Ok(())
    }
    /// clear from the cursor to the end of line
    pub fn clear_line(&self, w: &mut W) -> Result<(), ProgramError> {
        w.queue(Clear(ClearType::UntilNewLine))?;
        Ok(())
    }
    pub fn clear_area_to_right(&self, w: &mut W, area: &Area)  -> Result<(), ProgramError> {
        for y in area.top..area.top+area.height {
            self.goto(w, area.left, y)?;
            self.clear_line(w)?;
        }
        Ok(())
    }
}
