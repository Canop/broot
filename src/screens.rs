use std::io::{self, stdout, Write};

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
    pub scroll: i32,  // 0 for no scroll, positive if scrolled
    pub content_length: i32,
}
impl ScreenArea {
    pub fn new(top: u16, bottom: u16) -> ScreenArea {
        ScreenArea {
            top,
            bottom,
            scroll: 0,
            content_length: 0,
        }
    }
    pub fn try_scroll(&mut self, dy: i32) {
        self.scroll += dy;
        if self.scroll < 0 {
            self.scroll = 0;
        } else if self.scroll >= self.content_length {
            self.scroll = self.content_length-1;
        }
    }
}


impl Screen {
    pub fn new(w: u16, h: u16) -> io::Result<Screen> {
        let stdout = AlternateScreen::from(stdout().into_raw_mode()?);
        Ok(Screen { w, h, stdout })
    }
    pub fn write_lines(&mut self, area: &ScreenArea, lines: &Vec<String>) -> io::Result<()> {
        let mut i = area.scroll as usize;
        for y in area.top..=area.bottom {
            write!(
                self.stdout,
                "{}{}",
                termion::cursor::Goto(1, y),
                termion::clear::CurrentLine,
            )?;
            if i<lines.len() {
                debug!("{}", &lines[i]);
                write!(
                    self.stdout,
                    "{}",
                    &lines[i],
                )?;
                i += 1;
            }
        }
        Ok(())
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        write!(self.stdout, "{}", termion::cursor::Show).unwrap();
    }
}

pub fn max_tree_height() -> u16 {
    let (_, h) = termion::terminal_size().unwrap();
    h - 2
}

