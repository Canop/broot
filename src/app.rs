
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

use std::env;
use std::io::{self, Write, stdout, stdin};

use status::{Status};
use input::{Input};
use flat_tree::{TreeBuilder, Tree};
use tree_views::TreeView;

pub struct App {
    pub w: u16,
    pub h: u16,
    pub stdout: RawTerminal<io::Stdout>,
}

impl App {

    pub fn new() -> io::Result<App> {
        let stdout = stdout().into_raw_mode()?;
        let (w, h) = termion::terminal_size()?;
        Ok(App {
            w, h, stdout
        })
    }

    pub fn run(mut self) -> io::Result<()> {
        let tree = TreeBuilder::from(env::current_dir()?)?.build(self.h-2)?;
        println!("{:?}", tree);
        write!(
            self.stdout,
            "{}{}",
            termion::clear::All,
            termion::cursor::Hide
        )?;
        self.write_status("Hello")?;
        self.write_tree(&tree)?;
        self.stdout.flush()?;
        let stdin = stdin();
        let keys = stdin.keys();
        self.read(keys)?;
        write!(self.stdout, "{}", termion::cursor::Show)?;
        Ok(())
    }
}
