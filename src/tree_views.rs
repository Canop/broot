

use std::io::{self, Write};
use app::App;
use nodes::Node;

pub trait TreeView {
    fn tree_height(&self) -> u16;
    fn write_tree(&mut self, node: &Node) -> io::Result<()>;
    fn write_sub_tree(&mut self, node: &Node, depth: u16, y: u16) -> io::Result<u16>;
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
        for child in &node.childs {
            write!(
                self.stdout,
                "{}{}{}",
                termion::cursor::Goto(1, y),
                termion::clear::CurrentLine,
                child.name
            )?;
            if y >= self.h -2 { // this will be useless in the future: the tree will be correctly sized
                break;
            }
            y = y + 1;
        }
        Ok(y)
    }

}
