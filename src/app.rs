use std::io::{self, stdin, Write};
use termion::input::TermRead;

use commands::Command;
use external::Launchable;
use input::Input;
use screens::Screen;
use status::Status;
use verbs::VerbStore;

pub enum AppStateCmdResult {
    Quit,
    Keep,
    Launch(Launchable),
    DisplayError(String),
    NewState(Box<AppState>),
    PopState,
}

impl AppStateCmdResult {
    pub fn verb_not_found(text: &str) -> AppStateCmdResult {
        AppStateCmdResult::DisplayError(format!("verb not found: {:?}", &text))
    }
}

pub trait AppState {
    fn apply(&mut self, cmd: &mut Command, verb_store: &VerbStore) -> io::Result<AppStateCmdResult>;
    fn display(&mut self, screen: &mut Screen, verb_store: &VerbStore) -> io::Result<()>;
    fn write_status(&self, screen: &mut Screen, cmd: &Command) -> io::Result<()>;
}

pub struct App {
    pub states: Vec<Box<AppState>>, // stack: the last one is current
}

impl App {
    pub fn new() -> App {
        let states = Vec::new();
        App { states }
    }

    pub fn push(&mut self, new_state: Box<AppState>) {
        self.states.push(new_state);
    }

    pub fn mut_state(&mut self) -> &mut Box<AppState> {
        match self.states.last_mut() {
            Some(s) => s,
            None => {
                panic!("No path has been pushed");
            }
        }
    }
    pub fn state(&self) -> &Box<AppState> {
        match self.states.last() {
            Some(s) => s,
            None => {
                panic!("No path has been pushed");
            }
        }
    }

    pub fn run(mut self, verb_store: &VerbStore) -> io::Result<Option<Launchable>> {
        let (w, h) = termion::terminal_size()?;
        let mut screen = Screen::new(w, h)?;
        write!(
            screen.stdout,
            "{}{}",
            termion::clear::All,
            termion::cursor::Hide
        )?;
        self.mut_state().display(&mut screen, &verb_store)?;
        screen.write_status_text(
            "Hit <esc> to quit, '?' for help, or type a file's key to navigate",
        )?;
        let stdin = stdin();
        let keys = stdin.keys();
        let mut cmd = Command::new();
        for c in keys {
            //debug!("key: {:?}", &c);
            cmd.add_key(c?)?;
            info!("{:?}", &cmd.action);
            match self.mut_state().apply(&mut cmd, &verb_store)? {
                AppStateCmdResult::Quit => {
                    break;
                }
                AppStateCmdResult::Launch(launchable) => {
                    return Ok(Some(launchable));
                }
                AppStateCmdResult::NewState(boxed_state) => {
                    self.push(boxed_state);
                    cmd = Command::new();
                    self.state().write_status(&mut screen, &cmd)?;
                }
                AppStateCmdResult::PopState => {
                    self.states.pop();
                    //cmd = Command::from(&self.state().tree.key()); // doesn't really seem convenient
                    cmd = Command::new();
                    self.state().write_status(&mut screen, &cmd)?;
                }
                AppStateCmdResult::DisplayError(txt) => {
                    screen.write_status_err(&txt)?;
                }
                AppStateCmdResult::Keep => {
                    self.state().write_status(&mut screen, &cmd)?;
                }
            }
            self.mut_state().display(&mut screen, &verb_store)?;
            screen.write_input(&cmd)?;
        }
        Ok(None)
    }
}
