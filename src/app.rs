use std::io::{self, Write, stdout, stdin};
use std::path::{PathBuf};

use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;

use commands::Command;
use tree_build::TreeBuilder;
use input::{Input};
use status::{Status};
use tree_views::TreeView;

pub struct App {
    pub w: u16,
    pub h: u16,
    pub stdout: AlternateScreen<RawTerminal<io::Stdout>>,
}

impl Drop for App {
    fn drop(&mut self) {
        write!(self.stdout, "{}", termion::cursor::Show).unwrap();
    }
}

impl App {

    pub fn new() -> io::Result<App> {
        let stdout = AlternateScreen::from(stdout().into_raw_mode()?);
        let (w, h) = termion::terminal_size()?;
        Ok(App {
            w, h, stdout
        })
    }

    pub fn run(mut self, path: PathBuf) -> io::Result<()> {
        let mut tree = TreeBuilder::from(path)?.build(self.h-2)?;
        println!("{:?}", tree);
        write!(
            self.stdout,
            "{}{}",
            termion::clear::All,
            termion::cursor::Hide
        )?;
        self.write_tree(&tree)?;
        self.write_status(&tree)?;
        let stdin = stdin();
        let keys = stdin.keys();
        let mut cmd = Command::new();
        for c in keys {
            self.read(c?, &mut cmd)?;
            cmd.parse();
            if cmd.finished {
                break;
            }
            if !tree.try_select(&cmd.key) {
                tree.selection = 0;
            }
            self.write_tree(&tree)?;
            self.write_status(&tree)?;
        }
        Ok(())
    }

}
