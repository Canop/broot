
use std::{io, env, fs};

#[derive(Debug)]
pub struct Node {
    pub name: String,
    pub childs: Vec<Node>,
}

impl Node {
    pub fn new(name: String) -> Node {
        Node {
            name,
            childs: Vec::new()
        }
    }
    pub fn read() -> io::Result<Node> {
        let current_dir = env::current_dir()?;
        let mut root = Node::new("root".to_owned());
        for entry in fs::read_dir(current_dir)? {
            let name = match entry?.path().file_name() {
                Some(s) => s.to_string_lossy().into_owned(),
                None => "???".to_owned(),
            };
            root.childs.push(Node::new(
                   name
            ));
        }
        Ok(root)
    }
}
