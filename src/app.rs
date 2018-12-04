use std::io::{self, stdin, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use termion::input::TermRead;

use browser_states::BrowserState;
use commands::Command;
use external::Launchable;
use input::Input;
use screens::Screen;
use status::Status;
use task_sync::TaskLifetime;
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
    pub fn from_optional_state(os: Option<BrowserState>) -> AppStateCmdResult {
        match os {
            Some(os) => AppStateCmdResult::NewState(Box::new(os)),
            None => AppStateCmdResult::Keep,
        }
    }
}

pub trait AppState {
    fn apply(
        &mut self,
        cmd: &mut Command,
        verb_store: &VerbStore,
        tl: TaskLifetime,
    ) -> io::Result<AppStateCmdResult>;
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
        let (tx, rx) = mpsc::channel();
        let cmd_count = Arc::new(AtomicUsize::new(0));
        let key_count = Arc::clone(&cmd_count);
        thread::spawn(move || {
            for c in keys {
                key_count.fetch_add(1, Ordering::SeqCst);
                tx.send(c).unwrap();
            }
        });
        let mut cmd = Command::new();
        loop {
            let c = rx.recv().unwrap();
            //debug!("key: {:?}", &c);
            cmd.add_key(c?);
            let tl = TaskLifetime::new(&cmd_count);
            info!("{:?}", &cmd.action);
            screen.write_input(&cmd)?;
            match self.mut_state().apply(&mut cmd, &verb_store, tl)? {
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
                    if self.states.len() == 0 {
                        break;
                    }
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
            screen.write_input(&cmd)?;
            self.mut_state().display(&mut screen, &verb_store)?;
        }
        Ok(None)
    }
}
