/// displays the "input" at the bottom of the screen
/// (reading is managed in the app module)
use std::io::{self};

use crossterm::{Attribute::{self, Reset}, Color::{self, *}, Colored, Color::AnsiValue};

use crate::commands::Command;
use crate::screens::Screen;

pub trait Input {
    fn write_input(&mut self, cmd: &Command) -> io::Result<()>;
}

impl Input for Screen {
    fn write_input(&mut self, cmd: &Command) -> io::Result<()> {
        self.goto_clear(1, self.h);
        self.write(&format!(
            "{}{} {}",
            self.skin.input.apply_to(&cmd.raw),
            Attribute::Reverse,
            Attribute::NoInverse,
        ));
        Ok(())
    }
}
