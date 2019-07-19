//! broot's app is mainly a stack of AppState.
//! Commands parsed from the input are submitted to the current
//! appstate, which replies with a stateCmdResult which may
//! be
//! - a transition to a new state
//! - a pop to get back to the previous one
//! - an operation which keeps the state
//! - a request to quit broot
//! - a request to launch an executable (thus leaving broot)
use std::io::{self, Write};
use std::result::Result;

use crate::app_context::AppContext;
use crate::browser_states::BrowserState;
use crate::command_parsing::parse_command_sequence;
use crate::commands::Command;
use crate::event::EventSource;
use crate::errors::ProgramError;
use crate::errors::TreeBuildError;
use crate::external::Launchable;
use crate::input::Input;
use crate::screens::Screen;
use crate::skin::Skin;
use crate::spinner::Spinner;
use crate::status::Status;
use crate::task_sync::TaskLifetime;

/// Result of applying a command to a state
pub enum AppStateCmdResult {
    Quit,
    Keep,
    Launch(Box<Launchable>),
    DisplayError(String),
    NewState(Box<dyn AppState>, Command),
    PopStateAndReapply, // the state asks the command be executed on a previous state
    PopState,
    RefreshState,
}

impl AppStateCmdResult {
    pub fn verb_not_found(text: &str) -> AppStateCmdResult {
        AppStateCmdResult::DisplayError(format!("verb not found: {:?}", &text))
    }
    pub fn from_optional_state(
        os: Result<Option<BrowserState>, TreeBuildError>,
        cmd: Command,
    ) -> AppStateCmdResult {
        match os {
            Ok(Some(os)) => AppStateCmdResult::NewState(Box::new(os), cmd),
            Ok(None) => AppStateCmdResult::Keep,
            Err(e) => AppStateCmdResult::DisplayError(e.to_string()),
        }
    }
}

impl From<Launchable> for AppStateCmdResult {
    fn from(launchable: Launchable) -> Self {
        AppStateCmdResult::Launch(Box::new(launchable))
    }
}

/// a whole application state, stackable to allow reverting
///  to a previous one
pub trait AppState {
    fn apply(
        &mut self,
        cmd: &mut Command,
        screen: &mut Screen,
        con: &AppContext,
    ) -> io::Result<AppStateCmdResult>;
    fn refresh(&mut self, screen: &Screen, con: &AppContext) -> Command;
    fn has_pending_tasks(&self) -> bool;
    fn do_pending_task(&mut self, screen: &mut Screen, tl: &TaskLifetime);
    fn display(&mut self, screen: &mut Screen, con: &AppContext) -> io::Result<()>;
    fn write_status(&self, screen: &mut Screen, cmd: &Command, con: &AppContext) -> io::Result<()>;
    fn write_flags(&self, screen: &mut Screen, con: &AppContext) -> io::Result<()>;
}

pub struct App {
    states: Vec<Box<dyn AppState>>, // stack: the last one is current
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
        self.states.last_mut().expect("No path has been pushed")
    }
    fn state(&self) -> &Box<dyn AppState> {
        self.states.last().expect("No path has been pushed")
    }

    /// execute all the pending tasks until there's none remaining or
    ///  the allowed lifetime is expired (usually when the user typed a new key)
    fn do_pending_tasks(
        &mut self,
        cmd: &Command,
        screen: &mut Screen,
        con: &AppContext,
        tl: TaskLifetime,
    ) -> io::Result<()> {
        let has_task = self.state().has_pending_tasks();
        if has_task {
            loop {
                self.mut_state().display(screen, con)?;
                self.state().write_status(screen, &cmd, con)?;
                screen.write_spinner(true)?;
                if tl.is_expired() {
                    break;
                }
                self.mut_state().do_pending_task(screen, &tl);
                if !self.state().has_pending_tasks() {
                    break;
                }
            }
            screen.write_spinner(false)?;
            self.mut_state().display(screen, con)?;
            self.mut_state().write_status(screen, &cmd, con)?;
        }
        Ok(())
    }

    /// apply a command, and returns a command, which may be the same (modified or not)
    ///  or a new one.
    /// This normally mutates self
    fn apply_command(
        &mut self,
        cmd: Command,
        screen: &mut Screen,
        con: &AppContext,
    ) -> io::Result<Command> {
        let mut cmd = cmd;
        debug!("action: {:?}", &cmd.action);
        screen.read_size(con)?;
        screen.write_input(&cmd)?;
        self.state().write_flags(screen, con)?;
        screen.write_spinner(false)?;
        match self.mut_state().apply(&mut cmd, screen, con)? {
            AppStateCmdResult::Quit => {
                debug!("cmd result quit");
                self.quitting = true;
            }
            AppStateCmdResult::Launch(launchable) => {
                self.launch_at_end = Some(*launchable);
                self.quitting = true;
            }
            AppStateCmdResult::NewState(boxed_state, new_cmd) => {
                self.push(boxed_state);
                cmd = new_cmd;
                self.state().write_status(screen, &cmd, con)?;
            }
            AppStateCmdResult::RefreshState => {
                cmd = self.mut_state().refresh(screen, con);
            }
            AppStateCmdResult::PopState => {
                if self.states.len() == 1 {
                    debug!("quitting on last pop state");
                    self.quitting = true;
                } else {
                    self.states.pop();
                    cmd = self.mut_state().refresh(screen, con);
                    self.state().write_status(screen, &cmd, con)?;
                }
            }
            AppStateCmdResult::PopStateAndReapply => {
                if self.states.len() == 1 {
                    debug!("quitting on last pop state");
                    self.quitting = true;
                } else {
                    self.states.pop();
                    debug!("about to reapply {:?}", &cmd);
                    return self.apply_command(cmd, screen, con);
                }
            }
            AppStateCmdResult::DisplayError(txt) => {
                screen.write_status_err(&txt)?;
            }
            AppStateCmdResult::Keep => {
                self.state().write_status(screen, &cmd, con)?;
            }
        }
        self.mut_state().display(screen, con)?;
        screen.write_input(&cmd)?;
        self.state().write_flags(screen, con)?;
        Ok(cmd)
    }

    /// This is the main loop of the application
    pub fn run(mut self, con: &AppContext, skin: Skin) -> Result<Option<Launchable>, ProgramError> {
        let mut screen = Screen::new(con, skin)?;

        // create the initial state
        if let Some(bs) = BrowserState::new(
            con.launch_args.root.clone(),
            con.launch_args.tree_options.clone(),
            &screen,
            &TaskLifetime::unlimited(),
        )? {
            self.push(Box::new(bs));
        } else {
            unreachable!();
        }

        let mut cmd = Command::new();

        // if some commands were passed to the application
        //  we execute them before even starting listening for events
        if let Some(unparsed_commands) = &con.launch_args.commands {
            let commands = parse_command_sequence(unparsed_commands, con)?;
            for arg_cmd in &commands {
                cmd = (*arg_cmd).clone();
                cmd = self.apply_command(cmd, &mut screen, con)?;
                self.do_pending_tasks(&cmd, &mut screen, con, TaskLifetime::unlimited())?;
                if self.quitting {
                    return Ok(self.launch_at_end.take());
                }
            }
        }

        // we listen for events in a separate thread so that we can go on listening
        // when a long search is running, and interrupt it if needed
        let event_source = EventSource::new();

        screen.write_input(&cmd)?;
        self.mut_state().display(&mut screen, con)?;
        screen.write_status_text("Hit <esc> to quit, '?' for help, or some letters to search")?;
        self.state().write_flags(&mut screen, con)?;
        loop {
            if !self.quitting {
                self.do_pending_tasks(&cmd, &mut screen, con, event_source.new_task_lifetime())?;
            }
            let event = match event_source.recv() {
                Ok(event) => event,
                Err(_) => {
                    // this is how we quit the application,
                    // when the input thread is properly closed
                    break;
                }
            };
            cmd.add_event(event);
            cmd = self.apply_command(cmd, &mut screen, con)?;
            event_source.unblock(self.quitting);
        }
        Ok(self.launch_at_end.take())
    }
}

impl Drop for App {
    fn drop(&mut self) {
        io::stdout().flush().unwrap();
    }
}
