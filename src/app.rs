use std::io::{self, Write, stdout, stdin};
use std::path::{PathBuf};

use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;

use commands::{Action, Command};
use flat_tree::{Tree};
use tree_build::TreeBuilder;
use input::{Input};
use status::{Status};
use tree_options::TreeOptions;
use tree_views::TreeView;
use external::Launchable;
use verbs::VerbStore;

pub enum AppStateCmdResult {
    Quit,
    Keep,
    Launch(Launchable),
    DisplayError(String),
    NewRoot(PathBuf),
    NewOptions(TreeOptions),
    PopState,
}

impl AppStateCmdResult {
    fn verb_not_found(text: &str) -> AppStateCmdResult {
        AppStateCmdResult::DisplayError(
            format!("verb not found: {:?}", &text)
        )
    }
}

pub struct AppState {
    pub tree: Tree,
    pub options: TreeOptions,
}

pub struct App {
    pub w: u16,
    pub h: u16,
    pub states: Vec<AppState>, // stack: the last one is current
}

pub struct Screen {
    pub w: u16,
    pub h: u16,
    pub stdout: AlternateScreen<RawTerminal<io::Stdout>>,
}

impl AppState {
    fn apply(&mut self, cmd: &mut Command, verb_store: &VerbStore) -> io::Result<AppStateCmdResult> {
        Ok(match &cmd.action {
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
                        AppStateCmdResult::NewRoot(
                            self.tree.lines[self.tree.selection].path.clone()
                        )
                    },
                    false     => {
                        AppStateCmdResult::Launch(Launchable::opener(
                            &self.tree.lines[self.tree.selection].path
                        )?)
                    },
                }
            },
            Action::NudeVerb(verb_key) | Action::VerbSelection(verb_key)  => {
                match verb_store.get(&verb_key) {
                    Some(verb)  => {
                        verb.execute(
                            &self
                            //&self.tree.lines[self.tree.selection].path
                        )?
                    },
                    None        => {
                        AppStateCmdResult::verb_not_found(&verb_key)
                    },
                }
            },
            Action::Quit                => {
                AppStateCmdResult::Quit
            },
            _                           => {
                AppStateCmdResult::Keep
            }
        })
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
        Ok(App {
            w, h, states,
        })
    }

    pub fn push(&mut self, path: PathBuf, options: TreeOptions) -> io::Result<()> {
        let tree = TreeBuilder::from(path, options.clone())?.build(self.h-2)?;
        self.states.push(AppState{
            tree,
            options
        });
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

    pub fn run(mut self, verb_store: &VerbStore) -> io::Result<Option<Launchable>> {
        let mut screen = Screen::new(self.w, self.h)?;
        write!(
            screen.stdout,
            "{}{}",
            termion::clear::All,
            termion::cursor::Hide
        )?;
        screen.write_tree(&self.state().tree)?;
        screen.write_status_initial()?;
        let stdin = stdin();
        let keys = stdin.keys();
        let mut cmd = Command::new();
        for c in keys {
            //screen.write_status_text(&format!("{:?}", &c))?;
            cmd.add_key(c?)?;
            //screen.write_status_text(&format!("{:?}", &cmd.action))?;
            match self.mut_state().apply(&mut cmd, &verb_store)? {
                AppStateCmdResult::Quit                 => {
                    break;
                },
                AppStateCmdResult::Launch(launchable)   => {
                    return Ok(Some(launchable));
                },
                AppStateCmdResult::NewRoot(path)        => {
                    let options = self.state().options.clone();
                    self.push(path, options)?;
                    cmd = Command::new();
                    screen.write_status(&self.state())?;
                },
                AppStateCmdResult::NewOptions(options)  => {
                    let path = self.state().tree.root().clone();
                    self.push(path, options)?;
                    cmd = Command::new();
                    screen.write_status(&self.state())?;
                },
                AppStateCmdResult::PopState             => {
                    self.states.pop();
                    cmd = Command::from(&self.state().tree.key());
                    screen.write_status(&self.state())?;
                },
                AppStateCmdResult::DisplayError(txt)    => {
                    screen.write_status_err(&txt)?;
                },
                AppStateCmdResult::Keep                 => {
                    screen.write_status(&self.state())?;
                },
            }
            screen.write_tree(&self.state().tree)?;
            screen.write_input(&cmd)?;
        }
        Ok(None)
    }

}
