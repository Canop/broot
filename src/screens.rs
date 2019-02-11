use std::io::{self, stdout, Write};
use termion::color;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;

pub struct Screen {
    pub w: u16,
    pub h: u16,
    pub stdout: AlternateScreen<RawTerminal<io::Stdout>>,
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
    pub fn new() -> io::Result<Screen> {
        let stdout = AlternateScreen::from(stdout().into_raw_mode()?);
        let mut screen = Screen { w: 0, h: 0, stdout };
        screen.read_size()?;
        write!(screen.stdout, "{}", termion::cursor::Hide)?;
        Ok(screen)
    }
    pub fn read_size(&mut self) -> io::Result<()> {
        let (w, h) = termion::terminal_size()?;
        self.w = w;
        self.h = h;
        Ok(())
    }
    pub fn write_lines(&mut self, area: &ScreenArea, lines: &[String]) -> io::Result<()> {
        let mut i = area.scroll as usize;
        for y in area.top..=area.bottom {
            write!(
                self.stdout,
                "{}{}",
                termion::cursor::Goto(1, y),
                termion::clear::CurrentLine,
            )?;
            if i < lines.len() {
                write!(self.stdout, "{}", &lines[i],)?;
                i += 1;
            }
        }
        self.stdout.flush()?;
        Ok(())
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        write!(self.stdout, "{}", termion::cursor::Show).unwrap();
        // if we don't flush now, the standard screen may receive some
        // unflushed data which was meant for the alternate screen.
        self.stdout.flush().unwrap();
    }
}

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
        self.scroll += dy;
        if self.scroll < 0 {
            self.scroll = 0;
        } else if self.scroll >= self.content_length {
            self.scroll = self.content_length - 1;
        }
    }
    // draw a scrollbar at the righ, above content.
    // clears nothing before.
    // (note that this may lead to flickering)
    #[allow(dead_code)]
    pub fn draw_scrollbar(&self, screen: &mut Screen) -> io::Result<()> {
        let h = i32::from(self.bottom) - i32::from(self.top) + 1;
        if self.content_length > h {
            let sbh = h * h / self.content_length;
            let sc = i32::from(self.top) + self.scroll * h / self.content_length;
            write!(
                screen.stdout,
                "{}",
                color::Fg(color::AnsiValue::grayscale(9)),
            )?;
            for y in 0..sbh {
                write!(
                    screen.stdout,
                    "{}▐",
                    termion::cursor::Goto(screen.w, ((y + sc) as u16).min(self.bottom - 1)),
                )?;
            }
            write!(screen.stdout, "{}", color::Fg(color::Reset),)?;
        }
        Ok(())
    }
    // returns the top and bottom of the scrollbar, if any
    pub fn scrollbar(&self) -> Option<(u16, u16)> {
        let h = (self.bottom as i32) - (self.top as i32) + 1;
        if self.content_length <= h {
            return None;
        }
        let sbh = h * h / self.content_length;
        let sc = i32::from(self.top) + self.scroll * h / self.content_length;
        Some((sc as u16, (sc + sbh).min(i32::from(self.bottom) - 1) as u16))
    }
}
