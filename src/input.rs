

use std::io::{self, Write};

use termion::event::Key;

use app::App;
use commands::{Action, Command};

pub trait Input {
    fn readInput(&mut self, key: Key, cmd: &mut Command) -> io::Result<()>;
    fn writeInput(&mut self, cmd: &Command) -> io::Result<()>;
}

impl Input for App {
    fn readInput(&mut self, key: Key, cmd: &mut Command) -> io::Result<()> {
        //println!("{:?}", key);
        match key {
            Key::Char('\n') => {
                cmd.finished = true;
                if cmd.raw == "" {
                    cmd.action = Action::Quit;
                } else {
                    cmd.action = Action::from(&cmd.raw);
                }
            },
            Key::Up         => {
                cmd.action = Action::MoveSelection(-1);
            },
            Key::Down       => {
                cmd.action = Action::MoveSelection(1);
            },
            Key::Char(c)    => {
                //write!(self.stdout, "{}", c)?;
                cmd.raw.push(c);
                cmd.action = Action::from(&cmd.raw);
            },
            Key::Backspace  => {
                cmd.raw.pop();
                cmd.action = Action::from(&cmd.raw);
            },
            _               => {
            },
        }
        Ok(())
    }
    fn writeInput(&mut self, cmd: &Command) -> io::Result<()> {
        write!(
            self.stdout,
            "{}{}{}",
            termion::cursor::Goto(1, self.h),
            termion::clear::CurrentLine,
            cmd.raw,
        )?;
        self.stdout.flush()?;
        Ok(())
    }
}

