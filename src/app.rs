use std::io::{self, Write, stdout, stdin};
use std::path::{PathBuf};

use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;

use commands::{Action, Command};
use flat_tree::{LineType, Tree};
use tree_build::TreeBuilder;
use input::{Input};
use status::{Status};
use tree_views::TreeView;
use external;
use verbs::VerbStore;

enum AppStateCmdResult {
    Quit,
    Keep,
    NewState(PathBuf),
    PopState,
}

pub struct AppState {
    pub tree: Tree,
}

pub struct App {
    pub w: u16,
    pub h: u16,
    pub states: Vec<AppState>, // stack: the last one is current
    verb_store: VerbStore,
}

pub struct Screen {
    pub w: u16,
    pub h: u16,
    pub stdout: AlternateScreen<RawTerminal<io::Stdout>>,
}

impl AppState {
    fn apply(&mut self, cmd: &mut Command, app: &App) -> AppStateCmdResult {
        match &cmd.action {
            Action::Back                => {
                AppStateCmdResult::PopState
            },
            Action::MoveSelection(dy)   => {
                self.tree.move_selection(*dy);
                cmd.raw = self.tree.key();
                AppStateCmdResult::Keep
            },
            Action::Select(key)         => {
                if !self.tree.try_select(key) {
                    self.tree.selection = 0;
                }
                AppStateCmdResult::Keep
            },
            Action::OpenSelection       => {
                match self.tree.lines[self.tree.selection].is_dir() {
                    true      => {
                        println!("opening dir");
                        AppStateCmdResult::NewState(
                            self.tree.lines[self.tree.selection].path.clone()
                        )
                    },
                    false     => {
                        println!("opening file");
                        external::open_file(&self.tree.lines[self.tree.selection].path);
                        AppStateCmdResult::Keep
                    },
                }
            },
            Action::VerbSelection(verb_key) => {
                match app.verb_store.file_verb(&verb_key) {
                    Some(verb)  => {
                        // TODO
                    },
                    None    => {
                        // TODO
                    }
                }
            },
            Action::Quit                => {
                AppStateCmdResult::Quit
            },
            _                           => {
                AppStateCmdResult::Keep
            }
        }
    }
}

impl Screen {
    pub fn new(w: u16, h:u16) -> io::Result<Screen> {
        let stdout = AlternateScreen::from(stdout().into_raw_mode()?);
        Ok(Screen {
            w, h, stdout
        })
    }

}

impl Drop for Screen {
    fn drop(&mut self) {
        write!(self.stdout, "{}", termion::cursor::Show).unwrap();
    }
}

impl App {

    pub fn new() -> io::Result<App> {
        let (w, h) = termion::terminal_size()?;
        let states = Vec::new();
        let verb_store = VerbStore::new();
        Ok(App {
            w, h, states, verb_store,
        })
    }

    pub fn push(&mut self, path:PathBuf) -> io::Result<()> {
        let tree = TreeBuilder::from(path)?.build(self.h-2)?;
        self.states.push(AppState{ tree });
        Ok(())
    }

    pub fn mut_state(&mut self) -> &mut AppState {
        match self.states.last_mut() {
            Some(s) => s,
            None    => {
                panic!("No path has been pushed");
            },
        }
    }
    pub fn state(&self) -> &AppState {
        match self.states.last() {
            Some(s) => s,
            None    => {
                panic!("No path has been pushed");
            },
        }
    }

    pub fn run(mut self) -> io::Result<()> {
        let mut screen = Screen::new(self.w, self.h)?;
        write!(
            screen.stdout,
            "{}{}",
            termion::clear::All,
            termion::cursor::Hide
        )?;
        screen.write_tree(&self.state().tree)?;
        screen.write_status(&self.state())?;
        let stdin = stdin();
        let keys = stdin.keys();
        let mut cmd = Command::new();
        for c in keys {
            cmd.add_key(c?)?;
            screen.write_status_text(&format!("{:?}", &cmd.action))?;
            match self.mut_state().apply(&mut cmd) {
                AppStateCmdResult::Quit           => {
                    break;
                },
                AppStateCmdResult::NewState(path) => {
                    self.push(path)?;
                    cmd = Command::new();
                },
                AppStateCmdResult::PopState       => {
                    self.states.pop();
                    cmd = Command::from(&self.state().tree.key());
                },
                AppStateCmdResult::Keep           => {
                },
            }
            let state = self.state();
            screen.write_tree(&state.tree)?;
            //screen.write_status(&state)?; // TODO pass cmd too
            screen.writeInput(&cmd)?;
        }
        Ok(())
    }

}
