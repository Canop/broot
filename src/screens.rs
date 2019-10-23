
use crossterm::{AlternateScreen, ClearType, TerminalCursor};
use termimad::{Area, CompoundStyle, InputField};

use crate::{app_context::AppContext, errors::ProgramError, skin::Skin};

/// A wrapper around the solution used to write on screen,
/// the dimensions, and the skin
pub struct Screen {
    pub w: u16,
    pub h: u16,
    pub alternate_screen: crossterm::AlternateScreen,
    pub skin: Skin,
    pub input_field: InputField,
}

impl Screen {
    pub fn new(con: &AppContext, skin: Skin) -> Result<Screen, ProgramError> {
        let alternate_screen = AlternateScreen::to_alternate(true)?;
        let mut input_field = InputField::new(Area::new(0, 0, 10, 1));
        input_field.set_normal_style(CompoundStyle::from(skin.input.clone()));
        let mut screen = Screen {
            w: 0,
            h: 0,
            alternate_screen,
            skin,
            input_field,
        };
        screen.read_size(con)?;
        debug!("screen size: {} x {}", screen.w, screen.h);
        let cursor = TerminalCursor::new();
        cursor.hide().unwrap();
        Ok(screen)
    }
    pub fn read_size(&mut self, con: &AppContext) -> Result<(), ProgramError> {
        let (w, h) = termimad::terminal_size();
        self.w = w;
        self.h = h;
        if let Some(h) = con.launch_args.height {
            self.h = h;
        }
        self.input_field.change_area(0, h, w - 15);
        Ok(())
    }
    /// move the cursor to x,y and clears the line.
    ///
    /// top left corner is (1, 1)
    pub fn goto_clear(&self, x: u16, y: u16) {
        self.goto(x, y);
        self.clear_line();
    }
    /// move the cursor to x,y
    ///
    /// top left corner is (1, 1)
    pub fn goto(&self, x: u16, y: u16) {
        let cursor = TerminalCursor::new();
        cursor.goto(x - 1, y - 1).unwrap();
    }
    pub fn clear_line(&self) {
        let terminal = crossterm::Terminal::new();
        terminal.clear(ClearType::UntilNewLine).unwrap(); // FIXME try to manage those errors
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        let cursor = TerminalCursor::new();
        cursor.show().unwrap();
    }
}
