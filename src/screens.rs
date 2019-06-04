use std::io::{self};

use crossterm::{self, AlternateScreen, ClearType, TerminalCursor};

use crate::app_context::AppContext;
use crate::skin::Skin;

/// A wrapper around the solution used to write on screen,
/// the dimensions, and the skin
pub struct Screen {
    pub w: u16,
    pub h: u16,
    pub alternate_screen: crossterm::AlternateScreen,
    pub skin: Skin,
}

#[derive(Debug)]
pub struct ScreenArea {
    pub top: u16,    // first line
    pub bottom: u16, // last line, included
    pub scroll: i32, // 0 for no scroll, positive if scrolled
    pub content_length: i32,
    pub width: u16,
}

impl Screen {
    pub fn new(con: &AppContext, skin: Skin) -> io::Result<Screen> {
        let alternate_screen = AlternateScreen::to_alternate(true)?;
        let mut screen = Screen {
            w: 0,
            h: 0,
            alternate_screen,
            skin,
        };
        screen.read_size(con)?;
        info!("screen size: {} x {}", screen.w, screen.h);
        let cursor = TerminalCursor::new();
        cursor.hide().unwrap();
        Ok(screen)
    }
    pub fn read_size(&mut self, con: &AppContext) -> io::Result<()> {
        let terminal = crossterm::Terminal::new();
        let (w, h) = terminal.terminal_size();
        self.w = w;
        self.h = h+1;
        if let Some(h) = con.launch_args.height {
            self.h = h;
        }
        Ok(())
    }
    pub fn write(&mut self, s: &str) {
        let terminal = crossterm::Terminal::new();
        if let Err(e) = terminal.write(s) {
            warn!("error in write: {:?}", e);
        }
    }
    // move the cursor to x,y
    // top left corner is (1, 1)
    pub fn goto_clear(&self, x: u16, y: u16) {
        self.goto(x, y);
        self.clear_line();
    }
    pub fn goto(&self, x: u16, y: u16) {
        let cursor = TerminalCursor::new();
        //debug!("goto x={}, y={}", x, y);
        cursor.goto(x-1, y-1).unwrap();
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

/// a specific area on screen
/// FIXME use Termimad's one ?
impl ScreenArea {
    pub fn new(top: u16, bottom: u16, width: u16) -> ScreenArea {
        ScreenArea {
            top,
            bottom,
            scroll: 0,
            content_length: 0,
            width,
        }
    }
    pub fn try_scroll(&mut self, dy: i32) {
        self.scroll = (self.scroll + dy)
            .max(0)
            .min(self.content_length - self.height() + 1);
    }
    pub fn scrollbar(&self) -> Option<(u16, u16)> {
        let h = self.height();
        if self.content_length <= h {
            return None;
        }
        let sbh = h * h / self.content_length;
        let sc = i32::from(self.top) + self.scroll * h / self.content_length;
        Some((sc as u16, (sc + sbh - 1).min(i32::from(self.bottom)) as u16))
    }
    pub fn height(&self) -> i32 {
        i32::from(self.bottom - self.top) + 1
    }
}
