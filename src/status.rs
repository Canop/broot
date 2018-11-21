

use termion::{color, style};
use std::io::{self, Write};

use flat_tree::Tree;
use app::App;

pub trait Status {
    fn write_status(&mut self, tree: &Tree) -> io::Result<()>;
    fn write_status_text(&mut self, text: &str) -> io::Result<()>;
}

impl Status for App {
    fn write_status(&mut self, tree: &Tree) -> io::Result<()> {
        if tree.selection==0 {
            return self.write_status_text("Hit <enter> to quit, or type a file's key to navigate");
        }
        let line = &tree.lines[tree.selection];
        return self.write_status_text(&line.path.to_string_lossy());
    }
    fn write_status_text(&mut self, text: &str) -> io::Result<()> {
        let y = self.h-1;
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

