

use std::io::{self, Write};
use app::App;
use nodes::Node;

pub trait TreeView {
    fn tree_height(&self) -> u16;
    fn write_tree(&mut self, node: &Node) -> io::Result<()>;
    fn write_sub_tree(&mut self, node: &Node, depth: u16, y: u16) -> io::Result<u16>;
    fn write_line(&mut self, node: &Node, depth: u16, y: u16) -> io::Result<()>;
}

impl TreeView for App {
    fn tree_height(&self) -> u16 {
        self.h - 2
    }
    fn write_tree(&mut self, node: &Node) -> io::Result<()> {
        self.write_sub_tree(node, 0, 1)?;
        Ok(())
    }
    fn write_sub_tree(&mut self, node: &Node, depth: u16, y: u16) -> io::Result<u16> {
        let mut y = y;
        self.write_line(&node, depth, y)?;
        y = y + 1;
        for child in &node.childs {
            self.write_line(&child, depth+1, y)?;
            if y >= self.h -2 { // this will be useless in the future: the tree will be correctly sized
                break;
            }
            y = y + 1;
        }
        Ok(y)
    }
    fn write_line(&mut self, node: &Node, depth: u16, y: u16) -> io::Result<()> {
        write!(
            self.stdout,
            "{}{}{}{}",
            termion::cursor::Goto(1, y),
            termion::clear::CurrentLine,
            "  ".repeat(depth as usize),
            node.name
        )?;
        Ok(())
    }

}
