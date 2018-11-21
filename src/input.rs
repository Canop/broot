

use std::io::{self, Write};

use termion::event::Key;

use app::App;
use commands::Command;

pub trait Input {
    fn read(&mut self, key: Key, cmd: &mut Command) -> io::Result<()>;
}

impl Input for App {
    fn read(&mut self, key: Key, cmd: &mut Command) -> io::Result<()> {
        let y = self.h;
        //println!("{:?}", key);
        match key {
            Key::Char('\n') => {
                cmd.finished = true;
            },
            Key::Char(c)    => {
                write!(self.stdout, "{}", c)?;
                cmd.raw.push(c);
            },
            Key::Backspace  => {
                cmd.raw.pop();
            },
            _               => {
            },
        }
        write!(
            self.stdout,
            "{}{}{}",
            termion::cursor::Goto(1, y),
            termion::clear::CurrentLine,
            cmd.raw,
        )?;
        self.stdout.flush()?;
        Ok(())
    }
}

