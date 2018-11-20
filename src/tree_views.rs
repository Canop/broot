

use termion::{style, color};
use std::io::{self, Write};
use app::App;
use flat_tree::{TreeLine, Tree, LineType};

pub trait TreeView {
    fn tree_height(&self) -> u16;
    fn write_tree(&mut self, tree: &Tree) -> io::Result<()>;
    fn write_line(&mut self, line: &TreeLine) -> io::Result<()>;
}

impl TreeView for App {
    fn tree_height(&self) -> u16 {
        self.h - 2
    }
    fn write_tree(&mut self, tree: &Tree) -> io::Result<()> {
        for y in 1..self.h-1 {
            write!(
                self.stdout,
                "{}{}",
                termion::cursor::Goto(1, y),
                termion::clear::CurrentLine,
            )?;
            let line_index = (y -1) as usize;
            if line_index >= tree.lines.len() {
                continue;
            }
            let line = &tree.lines[line_index];
            for depth in 0..line.depth {
                write!(
                    self.stdout,
                    "{}{}{}",
                    color::Fg(color::AnsiValue::grayscale(5)),
                    match line.left_branchs[depth as usize] {
                        true    => {
                            match tree.has_branch(line_index+1, depth as usize) {
                                true    => match depth == line.depth-1 {
                                        true    => "├─",
                                        false   => "│ ",
                                },
                                false   => "└─",
                            }
                        },
                        false   => "  ",
                    },
                    color::Fg(color::Reset),
                )?;
            }
            self.write_line(line)?;
        }
        self.stdout.flush()?;
        Ok(())
    }
    fn write_line(&mut self, line: &TreeLine) -> io::Result<()> {
        match &line.content {
            LineType::Dir(name)        => {
                write!(
                    self.stdout,
                    "{} {} {} {}{}{}",
                    color::Bg(color::AnsiValue::grayscale(2)),
                    &line.key,
                    color::Bg(color::Reset),
                    style::Bold,
                    &name,
                    style::Reset,
                )?;
            },
            LineType::File(name)        => {
                write!(
                    self.stdout,
                    "{} {} {} {}",
                    color::Bg(color::AnsiValue::grayscale(2)),
                    &line.key,
                    color::Bg(color::Reset),
                    &name,
                )?;
            },
            LineType::Pruning(n)  => {
                write!(
                    self.stdout,
                    "{}... {} other files…{}",
                    style::Italic,
                    n,
                    style::Reset,
                )?;
            }
        }
        Ok(())
    }

}
