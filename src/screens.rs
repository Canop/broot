use std::io::{self};
use termion::color;

use crossterm::{self, Crossterm, TerminalCursor};

use crate::app_context::AppContext;
use crate::skin::Skin;

/// A wrapper around the solution used to write on screen,
/// the dimensions, and the skin
pub struct Screen {
    pub w: u16,
    pub h: u16,
    pub alternate_screen: crossterm::AlternateScreen, // alternateScree.screen implements Write
    //pub crossterm: crossterm::Crossterm,
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
        let screen = crossterm::Screen::default();
        let alternate_screen = screen.enable_alternate_modes(true).unwrap();
        //let crossterm = Crossterm::from_screen(&alternate_screen.screen);
        let mut screen = Screen {
            w: 0,
            h: 0,
            alternate_screen,
            //crossterm,
            skin,
        };
        screen.read_size(con)?;
        info!("screen size: {} x {}", screen.w, screen.h);
        screen.write(&format!("{}", termion::cursor::Hide));
        Ok(screen)
    }
    pub fn read_size(&mut self, con: &AppContext) -> io::Result<()> {
        let (w, h) = termion::terminal_size()?;
        self.w = w;
        self.h = h;
        if let Some(h) = con.launch_args.height {
            self.h = h;
        }
        Ok(())
    }
    pub fn reset_colors(&mut self) {
        self.write(&format!(
            "{}{}",
            color::Fg(color::Reset),
            color::Bg(color::Reset),
        ));
    }
    pub fn flush(&mut self) -> io::Result<()> {
        self.alternate_screen.screen.stdout.flush()
    }
    pub fn write(&mut self, s: &str) {
        let crossterm = Crossterm::from_screen(&self.alternate_screen.screen);
        let terminal = crossterm.terminal();
        if let Err(e) = terminal.write(s) {
            warn!("error in write: {:?}", e);
        }
        self.flush().unwrap();
    }
    pub fn goto(&mut self, x: u16, y: u16) {
        let cursor = TerminalCursor::from_output(&self.alternate_screen.screen.stdout);
        info!("goto x={}, y={}", x, y);
        cursor.goto(x+1, y+1).unwrap();
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        let cursor = TerminalCursor::from_output(&self.alternate_screen.screen.stdout);
        cursor.show().unwrap();
    }
}

//impl Drop for Screen {
//    fn drop(&mut self) {
//        write!(self.stdout, "{}", termion::cursor::Show).unwrap();
//        // if we don't flush now, the standard screen may receive some
//        // unflushed data which was meant for the alternate screen.
//        // see https://gitlab.redox-os.org/redox-os/termion/issues/158
//        self.stdout.flush().unwrap();
//    }
//}

//impl Write for Screen {
//    fn write(&mut self, buf: &[u8]) -> Result<usize> {
//        self.stdout.write_buf(buf)
//    }
//
//    fn flush(&mut self) -> Result<()> {
//        self.stdout.flush()
//    }
//}

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
