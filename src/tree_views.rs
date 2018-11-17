

use std::io::{self, Write};
use app::App;
use flat_tree::{TreeLine, Tree};

pub trait TreeView {
    fn tree_height(&self) -> u16;
    fn write_tree(&mut self, tree: &Tree) -> io::Result<()>;
    fn write_line(&mut self, line: &TreeLine, y: u16) -> io::Result<()>;
}

impl TreeView for App {
    fn tree_height(&self) -> u16 {
        self.h - 2
    }
    fn write_tree(&mut self, tree: &Tree) -> io::Result<()> {
        for y in 1..self.h-2 {
            if y-1 < (tree.lines.len() as u16) {
                self.write_line(&tree.lines[(y-1) as usize], y)?;
            } else {
                write!(
                    self.stdout,
                    "{}{}",
                    termion::cursor::Goto(1, y),
                    termion::clear::CurrentLine,
                )?;
            }
        }
        Ok(())
    }
    fn write_line(&mut self, line: &TreeLine, y: u16) -> io::Result<()> {
        write!(
            self.stdout,
            "{}{}{}{}",
            termion::cursor::Goto(1, y),
            termion::clear::CurrentLine,
            "  ".repeat(line.depth as usize),
            line.name
        )?;
        Ok(())
    }

}
