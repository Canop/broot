
use std::{io, env};
use std::fs::{self, ReadDir};
use std::path::{Path, PathBuf};
use std::collections::VecDeque;

#[derive(Debug)]
pub enum NodeContent {
    Pruning,
    File,
    Dir,
}


#[derive(Debug)]
pub struct Node {
    pub path: PathBuf, // parent path in case of pruning
    pub name: String,
    pub content: NodeContent,
    pub childs: Vec<Node>,
}

impl Node {
    // create a (not pruning) node, doesn't dive
    pub fn create(path: PathBuf) -> io::Result<Node> {
            let name = match path.file_name() {
                Some(s) => s.to_string_lossy().into_owned(),
                None => String::from("???"),
            };
            let metadata = fs::metadata(&path)?;
            let content = match metadata.is_dir() {
                true    => NodeContent::Dir,
                false   => NodeContent::File,
            };
            let childs = Vec::new();
            Ok(Node {
                path,
                name,
                content,
                childs,
            })
    }
    pub fn read_dir(&self) -> io::Result<ReadDir> {
        fs::read_dir(&self.path)
    }
    //pub fn add_child(node: Node) {
    //    if let Dir{ childs } =
    //}
    pub fn read(nb_lines_max: u16) -> io::Result<Node> {
        let current_dir = env::current_dir()?;
        let mut root = Node::create(current_dir)?;
        //let mut nb_lines = 1;
        //let mut depth = 0;
        //let mut frontier = VecDeque<&Node>::new();
        //frontier.push_back(&root);
        //while let Some(node) = frontier.pop_front() {
        //
        //    if nb_lines >= nb_lines_max break;
        //}
        //for entry in fs::read_dir(current_dir)? {
        for entry in root.read_dir()? {
            let child = Node::create(entry?.path())?;
            root.childs.push(child);
        }
        Ok(root)
    }
}
