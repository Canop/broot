//! broot's app is mainly a stack of AppState.
//! Commands parsed from the input are submitted to the current
//! appstate, which replies with a stateCmdResult which may
//! be
//! - a transition to a new state
//! - a pop to get back to the previous one
//! - an operation which keeps the state
//! - a request to quit broot
//! - a request to launch an executable (thus leaving broot)
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

/// Result of applying a command to a state
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
    states: Vec<Box<dyn AppState>>,    // stack: the last one is current
    quitting: bool,
    launch_at_end: Option<Launchable>, // what must be launched after end
}

impl App {
    pub fn new() -> App {
        App {
            states: Vec::new(),
            quitting: false,
            launch_at_end: None,
        }
    }

    pub fn push(&mut self, new_state: Box<dyn AppState>) {
        self.states.push(new_state);
    }

    fn mut_state(&mut self) -> &mut Box<dyn AppState> {
        match self.states.last_mut() {
            Some(s) => s,
            None => {
                panic!("No path has been pushed");
            }
        }
    }
    fn state(&self) -> &Box<dyn AppState> {
        match self.states.last() {
            Some(s) => s,
            None => {
                panic!("No path has been pushed");
            }
        }
    }

    /// execute all the pending tasks until there's none remaining or
    ///  the allowed lifetime is expired (usually when the user typed a new key)
    fn do_pending_tasks(&mut self, cmd: &Command, screen: &mut Screen, con: &AppContext, tl: TaskLifetime) -> io::Result<()> {
        let has_task = self.state().has_pending_tasks();
        if has_task {
            loop {
                self.state().write_status(screen, &cmd, con)?;
                screen.write_spinner(true)?;
                self.mut_state().display(screen, con)?;
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
        self.mut_state().display(screen, con)?;
        Ok(())
    }

    /// apply a command, and returns a command, which may be the same (modified or not)
    ///  or a new one.
    /// This normally mutates self
    fn apply_command(&mut self, cmd: Command, screen: &mut Screen, con: &AppContext) -> io::Result<Command> {
        let mut cmd = cmd;
        debug!("action: {:?}", &cmd.action);
        screen.write_input(&cmd)?;
        self.state().write_flags(screen, con)?;
        match self.mut_state().apply(&mut cmd, con)? {
            AppStateCmdResult::Quit => {
                debug!("cmd result quit");
                self.quitting = true;
            }
            AppStateCmdResult::Launch(launchable) => {
                self.launch_at_end = Some(launchable);
                self.quitting = true;
            }
            AppStateCmdResult::NewState(boxed_state) => {
                self.push(boxed_state);
                cmd = cmd.pop_verb();
                self.state().write_status(screen, &cmd, con)?;
            }
            AppStateCmdResult::PopState => {
                if self.states.len() == 1 {
                    debug!("quitting on last pop state");
                    self.quitting = true;
                } else {
                    self.states.pop();
                    cmd = Command::new();
                    self.state().write_status(screen, &cmd, con)?;
                }
            }
            AppStateCmdResult::DisplayError(txt) => {
                screen.write_status_err(&txt)?;
            }
            AppStateCmdResult::Keep => {
                self.state().write_status(screen, &cmd, con)?;
            }
        }
        screen.write_input(&cmd)?;
        self.state().write_flags(screen, con)?;
        Ok(cmd)
    }

    /// This is the main loop of the application
    pub fn run(mut self, con: &AppContext, input_commands: Vec<Command>) -> io::Result<Option<Launchable>> {
        let (w, h) = termion::terminal_size()?;
        let mut screen = Screen::new(w, h)?;
        write!(
            screen.stdout,
            "{}{}",
            termion::clear::All,
            termion::cursor::Hide
        )?;

        // if some commands were passed to the application
        //  we execute them before even starting listening for keys
        for cmd in input_commands {
            let cmd = self.apply_command(cmd, &mut screen, con)?;
            self.do_pending_tasks(
                &cmd,
                &mut screen,
                con,
                TaskLifetime::unlimited(),
            )?;
            if self.quitting {
                return Ok(self.launch_at_end);
            }
        }

        // we listen for keys in a separate thread so that we can go on listening
        // when a long search is running, and interrupt it if needed
        let keys = stdin().keys();
        let (tx_keys, rx_keys) = mpsc::channel();
        let (tx_quit, rx_quit) = mpsc::channel();
        let cmd_count = Arc::new(AtomicUsize::new(0));
        let key_count = Arc::clone(&cmd_count);
        thread::spawn(move || {
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
        screen.write_status_text("Hit <esc> to quit, '?' for help, or type some letters to search")?;
        self.state().write_flags(&mut screen, con)?;
        loop {
            if !self.quitting {
                self.do_pending_tasks(
                    &cmd,
                    &mut screen,
                    con,
                    TaskLifetime::new(&cmd_count),
                )?;
            }
            let c = match rx_keys.recv() {
                Ok(c) => c,
                Err(_) => {
                    // this is how we quit the application,
                    // when the input thread is properly closed
                    break;
                }
            };
            cmd.add_key(c?);
            cmd = self.apply_command(cmd, &mut screen, con)?;
            tx_quit.send(self.quitting).unwrap();
        }
        Ok(self.launch_at_end)
    }
}
