/// broot's app is mainly a stack of AppState.
/// Commands parsed from the input are submitted to the current
/// appstate, which replies with a stateCmdResult which may
/// be
/// - a transition to a new state
/// - a pop to get back to the previous one
/// - an operation which keeps the state
/// - a request to quit broot
/// - a request to launch an executable (thus leaving broot)
use std::io::{self, stdin, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use termion::input::TermRead;

use crate::app_context::AppContext;
use crate::browser_states::BrowserState;
use crate::commands::Command;
use crate::external::Launchable;
use crate::input::Input;
use crate::screens::Screen;
use crate::spinner::Spinner;
use crate::status::Status;
use crate::task_sync::TaskLifetime;

pub enum AppStateCmdResult {
    Quit,
    Keep,
    Launch(Launchable),
    DisplayError(String),
    NewState(Box<dyn AppState>),
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
    fn apply(&mut self, cmd: &mut Command, con: &AppContext) -> io::Result<AppStateCmdResult>;
    fn has_pending_tasks(&self) -> bool;
    fn do_pending_task(&mut self, tl: &TaskLifetime);
    fn display(&mut self, screen: &mut Screen, con: &AppContext) -> io::Result<()>;
    fn write_status(&self, screen: &mut Screen, cmd: &Command, con: &AppContext) -> io::Result<()>;
    fn write_flags(&self, screen: &mut Screen, con: &AppContext) -> io::Result<()>;
}

pub struct App {
    pub states: Vec<Box<dyn AppState>>, // stack: the last one is current
}

impl App {
    pub fn new() -> App {
        App { states: Vec::new() }
    }

    pub fn push(&mut self, new_state: Box<dyn AppState>) {
        self.states.push(new_state);
    }

    pub fn mut_state(&mut self) -> &mut Box<dyn AppState> {
        match self.states.last_mut() {
            Some(s) => s,
            None => {
                panic!("No path has been pushed");
            }
        }
    }
    pub fn state(&self) -> &Box<dyn AppState> {
        match self.states.last() {
            Some(s) => s,
            None => {
                panic!("No path has been pushed");
            }
        }
    }

    pub fn run(mut self, con: &AppContext) -> io::Result<Option<Launchable>> {
        let (w, h) = termion::terminal_size()?;
        let mut screen = Screen::new(w, h)?;
        write!(
            screen.stdout,
            "{}{}",
            termion::clear::All,
            termion::cursor::Hide
        )?;
        let stdin = stdin();
        let keys = stdin.keys();
        let (tx_keys, rx_keys) = mpsc::channel();
        let (tx_quit, rx_quit) = mpsc::channel();
        let cmd_count = Arc::new(AtomicUsize::new(0));
        let key_count = Arc::clone(&cmd_count);
        thread::spawn(move || {
            // we listen for keys in a separate thread so that we can go on listening
            // when a long search is running, and interrupt it if needed
            for c in keys {
                key_count.fetch_add(1, Ordering::SeqCst);
                // we send the command to the receiver in the
                //  main event loop
                tx_keys.send(c).unwrap();
                let quit = rx_quit.recv().unwrap();
                if quit {
                    // cleanly quitting this thread is necessary
                    //  to ensure stdin is properly closed when
                    //  we launch an external application in the same
                    //  terminal
                    return;
                }
            }
        });
        let mut cmd = Command::new();
        screen.write_input(&cmd)?;
        screen
            .write_status_text("Hit <esc> to quit, '?' for help, or type some letters to search")?;
        self.state().write_flags(&mut screen, con)?;
        let mut quit = false;
        loop {
            let tl = TaskLifetime::new(&cmd_count);
            let has_task = self.state().has_pending_tasks();
            if !quit && has_task {
                loop {
                    self.state().write_status(&mut screen, &cmd, con)?;
                    screen.write_spinner(true)?;
                    self.mut_state().display(&mut screen, con)?;
                    if tl.is_expired() {
                        break;
                    }
                    self.mut_state().do_pending_task(&tl);
                    if !self.state().has_pending_tasks() {
                        break;
                    }
                }
                screen.write_spinner(false)?;
            }
            self.mut_state().display(&mut screen, con)?;
            let c = match rx_keys.recv() {
                Ok(c) => c,
                Err(_) => {
                    break;
                }
            };
            cmd.add_key(c?);
            debug!("action: {:?}", &cmd.action);
            screen.write_input(&cmd)?;
            self.state().write_flags(&mut screen, con)?;
            match self.mut_state().apply(&mut cmd, con)? {
                AppStateCmdResult::Quit => {
                    debug!("cmd result quit");
                    quit = true;
                }
                AppStateCmdResult::Launch(launchable) => {
                    return Ok(Some(launchable));
                }
                AppStateCmdResult::NewState(boxed_state) => {
                    self.push(boxed_state);
                    cmd = cmd.pop_verb();
                    self.state().write_status(&mut screen, &cmd, con)?;
                }
                AppStateCmdResult::PopState => {
                    if self.states.len() == 1 {
                        debug!("quitting on last pop state");
                        quit = true;
                    } else {
                        self.states.pop();
                        cmd = Command::new();
                        self.state().write_status(&mut screen, &cmd, con)?;
                    }
                }
                AppStateCmdResult::DisplayError(txt) => {
                    screen.write_status_err(&txt)?;
                }
                AppStateCmdResult::Keep => {
                    self.state().write_status(&mut screen, &cmd, con)?;
                }
            }
            screen.write_input(&cmd)?;
            self.state().write_flags(&mut screen, con)?;
            tx_quit.send(quit).unwrap();
        }
        Ok(None)
    }
}
