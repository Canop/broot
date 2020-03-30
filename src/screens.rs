use {
    crate::{
        app::AppContext,
        errors::ProgramError,
        skin::{self, StatusMadSkinSet},
        skin::Skin,
    },
    crossterm::{
        cursor,
        terminal::{Clear, ClearType},
        QueueableCommand,
    },
    std::io::Write,
    termimad::{Area, CompoundStyle, InputField, MadSkin},
};

pub static FLAGS_AREA_WIDTH: u16 = 10;

pub struct Screen {
    pub width: u16,
    pub height: u16,
    pub skin: Skin,
    pub input_field: InputField,
    pub status_skin: StatusMadSkinSet,
    pub help_skin: MadSkin,
}

impl Screen {
    pub fn new(con: &AppContext, skin: Skin) -> Result<Screen, ProgramError> {
        let mut input_field = InputField::new(Area::new(0, 0, 10, 1));
        input_field.set_normal_style(CompoundStyle::from(skin.input.clone()));
        let status_skin = StatusMadSkinSet::from_skin(&skin);
        let help_skin = skin::make_help_mad_skin(&skin);
        let mut screen = Screen {
            width: 0,
            height: 0,
            skin,
            input_field,
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
        self.input_field.change_area(0, h - 1, w - FLAGS_AREA_WIDTH);
    }
    pub fn read_size(&mut self, con: &AppContext) -> Result<(), ProgramError> {
        let (w, h) = termimad::terminal_size();
        self.set_terminal_size(w, h, con);
        Ok(())
    }
    /// move the cursor to x,y and clears the line.
    pub fn goto_clear(&self, w: &mut impl Write, x: u16, y: u16) -> Result<(), ProgramError> {
        self.goto(w, x, y)?;
        self.clear_line(w)
    }
    /// move the cursor to x,y
    pub fn goto(&self, w: &mut impl Write, x: u16, y: u16) -> Result<(), ProgramError> {
        w.queue(cursor::MoveTo(x, y))?;
        Ok(())
    }
    /// clear the whole screen
    pub fn clear(&self, w: &mut impl Write) -> Result<(), ProgramError> {
        w.queue(Clear(ClearType::All))?;
        Ok(())
    }
    /// clear from the cursor to the end of line
    pub fn clear_line(&self, w: &mut impl Write) -> Result<(), ProgramError> {
        w.queue(Clear(ClearType::UntilNewLine))?;
        Ok(())
    }
}
