//! the status module manages writing information on the grey line
//!  near the bottom of the screen

use std::io::{self, Write};
use termion::color;

use app::{AppState, Screen};

pub trait Status {
    fn write_status(&mut self, state: &AppState) -> io::Result<()>;
    fn write_status_initial(&mut self) -> io::Result<()>;
    fn write_status_text(&mut self, text: &str) -> io::Result<()>;
    fn write_status_err(&mut self, text: &str) -> io::Result<()>;
}

impl Status for Screen {
    fn write_status(&mut self, state: &AppState) -> io::Result<()> {
        if state.tree.selection == 0 {
            return self.write_status_text("Hit <enter> to quit, or type a file's key to navigate");
        }
        let line = &state.tree.lines[state.tree.selection];
        self.write_status_text(match line.is_dir() {
            true => "Hit <enter> to focus, or type a space then a verb",
            false => "Hit <enter> to open the file, or type a space then a verb",
        })
        //return self.write_status_text(&line.path.to_string_lossy());
    }
    fn write_status_initial(&mut self) -> io::Result<()> {
        self.write_status_text("Hit <esc> to quit, or type a file's key to navigate")
    }
    fn write_status_err(&mut self, text: &str) -> io::Result<()> {
        let y = self.h - 1;
        write!(
            self.stdout,
            "{}{}{}{}{}{}{}",
            termion::cursor::Goto(1, y),
            color::Bg(color::AnsiValue::grayscale(2)),
            color::Fg(color::Red),
            termion::clear::CurrentLine,
            text,
            color::Bg(color::Reset),
            color::Fg(color::Reset),
        )?;
        self.stdout.flush()?;
        Ok(())
    }
    fn write_status_text(&mut self, text: &str) -> io::Result<()> {
        let y = self.h - 1;
        write!(
            self.stdout,
            "{}{}{}{}{}",
            termion::cursor::Goto(1, y),
            color::Bg(color::AnsiValue::grayscale(2)),
            termion::clear::CurrentLine,
            text,
            color::Bg(color::Reset),
        )?;
        self.stdout.flush()?;
        Ok(())
    }
}
