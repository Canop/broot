

use termion::event::Key;
use termion::input::{Keys};

use std::io::{self, Write, Stdin};

use app::App;

pub trait Input {
    fn read(&mut self, keys: Keys<Stdin>) -> io::Result<String>;
}

impl Input for App {
    fn read(&mut self, keys: Keys<Stdin>) -> io::Result<(String)> {
        let mut input = String::new();
        let y = self.h;
        write!(
            self.stdout,
            "{}{}",
            termion::cursor::Goto(1, y),
            termion::clear::CurrentLine,
        )?;
        for k in keys {
            //println!("{:?}", k);
            match k.unwrap() {
                Key::Char('\n') => { break; },
                Key::Char(c) => {
                    write!(self.stdout, "{}", c)?;
                    input.push(c);
                },
                _              => {},
            }

            self.stdout.flush().unwrap();
        }

        Ok(input)
    }
}
