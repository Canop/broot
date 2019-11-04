//! broot's app is mainly a stack of AppState.
//! Commands parsed from the input are submitted to the current
//! appstate, which replies with a stateCmdResult which may
//! be
//! - a transition to a new state
//! - a pop to get back to the previous one
//! - an operation which keeps the state
//! - a request to quit broot
//! - a request to launch an executable (thus leaving broot)
use std::io::Write;

use crossterm::{
    cursor,
    input::{DisableMouseCapture, EnableMouseCapture},
    queue,
    screen::{EnterAlternateScreen, LeaveAlternateScreen},
};
use termimad::EventSource;

use crate::{
    app_context::AppContext,
    browser_states::BrowserState,
    command_parsing::parse_command_sequence,
    commands::Command,
    errors::{ProgramError, TreeBuildError},
    external::Launchable,
    file_sizes,
    screens::Screen,
    skin::Skin,
    spinner::Spinner,
    status::Status,
    task_sync::TaskLifetime,
};

pub type W = std::io::Stderr;

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
    ) -> Result<AppStateCmdResult, ProgramError>;
    fn refresh(&mut self, screen: &Screen, con: &AppContext) -> Command;
    fn has_pending_tasks(&self) -> bool;
    fn do_pending_task(
        &mut self,
        w: &mut W, // TODO make this useless ?
        screen: &mut Screen,
        tl: &TaskLifetime
    );
    fn display(
        &mut self,
        w: &mut W,
        screen: &Screen,
        con: &AppContext
    ) -> Result<(), ProgramError>;
    fn write_status(
        &self,
        w: &mut W,
        screen: &mut Screen,
        cmd: &Command,
        con: &AppContext,
    ) -> Result<(), ProgramError>;
    fn write_flags(
        &self,
        w: &mut W,
        screen: &mut Screen,
        con: &AppContext,
    ) -> Result<(), ProgramError>;
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
        w: &mut W,
        cmd: &Command,
        screen: &mut Screen,
        con: &AppContext,
        tl: TaskLifetime,
    ) -> Result<(), ProgramError> {
        let has_task = self.state().has_pending_tasks();
        if has_task {
            loop {
                self.mut_state().display(w, screen, con)?;
                self.state().write_status(w, screen, &cmd, con)?;
                screen.write_spinner(w, true)?;
                if tl.is_expired() {
                    break;
                }
                self.mut_state().do_pending_task(w, screen, &tl);
                if !self.state().has_pending_tasks() {
                    break;
                }
            }
            screen.write_spinner(w, false)?;
            self.mut_state().display(w, screen, con)?;
            self.mut_state().write_status(w, screen, &cmd, con)?;
        }
        Ok(())
    }

    /// apply a command, and returns a command, which may be the same (modified or not)
    ///  or a new one.
    /// This normally mutates self
    fn apply_command(
        &mut self,
        w: &mut W,
        cmd: Command,
        screen: &mut Screen,
        con: &AppContext,
    ) -> Result<Command, ProgramError> {
        let mut cmd = cmd;
        debug!("action: {:?}", &cmd.action);
        screen.read_size(con)?;
        let mut error: Option<String> = None;
        let cmd_result = self.mut_state().apply(&mut cmd, screen, con)?;
        match cmd_result {
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
            }
            AppStateCmdResult::RefreshState => {
                file_sizes::clear_cache();
                cmd = self.mut_state().refresh(screen, con);
            }
            AppStateCmdResult::PopState => {
                if self.states.len() == 1 {
                    debug!("quitting on last pop state");
                    self.quitting = true;
                } else {
                    self.states.pop();
                    cmd = self.mut_state().refresh(screen, con);
                }
            }
            AppStateCmdResult::PopStateAndReapply => {
                if self.states.len() == 1 {
                    debug!("quitting on last pop state");
                    self.quitting = true;
                } else {
                    self.states.pop();
                    debug!("about to reapply {:?}", &cmd);
                    return self.apply_command(w, cmd, screen, con);
                }
            }
            AppStateCmdResult::DisplayError(txt) => {
                error = Some(txt.clone());
            }
            _ => {}
        }
        self.mut_state().display(w, screen, con)?;
        screen.write_spinner(w, false)?;
        match error {
            Some(text) => screen.write_status_err(w, &text)?,
            None => self.state().write_status(w, screen, &cmd, con)?,
        }
        screen.input_field.set_content(&cmd.raw);
        screen.input_field.display_on(w)?;
        self.state().write_flags(w, screen, con)?;
        Ok(cmd)
    }

    /// called exactly once at end run, cleans the writer (which
    /// is usually stdout or stderr)
    fn end(&mut self, writer: &mut W) ->Result<Option<Launchable>, ProgramError> {
        queue!(writer, DisableMouseCapture)?;
        queue!(writer, cursor::Show)?;
        queue!(writer, LeaveAlternateScreen)?;
        writer.flush()?;
        debug!("we left the screen");
        Ok(self.launch_at_end.take())
    }

    /// This is the main loop of the application
    pub fn run(
        &mut self,
        writer: &mut W,
        con: &AppContext,
        skin: Skin,
    ) -> Result<Option<Launchable>, ProgramError> {

        queue!(writer, EnterAlternateScreen)?;
        queue!(writer, cursor::Hide)?;
        debug!("we're on screen");
        let mut screen = Screen::new(con, skin)?;

        // we listen for events in a separate thread so that we can go on listening
        // when a long search is running, and interrupt it if needed
        queue!(writer, EnableMouseCapture)?;
        let event_source = EventSource::new()?;
        let rx_events = event_source.receiver();

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
                cmd = self.apply_command(writer, cmd, &mut screen, con)?;
                self.do_pending_tasks(writer, &cmd, &mut screen, con, TaskLifetime::unlimited())?;
                if self.quitting {
                    return self.end(writer);
                }
            }
        }

        screen.input_field.display_on(writer)?;
        self.mut_state().display(writer, &screen, con)?;
        screen.write_spinner(writer, false)?;
        screen.write_status_text(writer, "Hit <esc> to quit, '?' for help, or some letters to search")?;
        self.state().write_flags(writer, &mut screen, con)?;

        loop {
            let tl = TaskLifetime::new(event_source.shared_event_count());
            if !self.quitting {
                self.do_pending_tasks(writer, &cmd, &mut screen, con, tl)?;
            }
            let event = match rx_events.recv() {
                Ok(event) => event,
                Err(_) => {
                    // this is how we quit the application,
                    // when the input thread is properly closed
                    break;
                }
            };
            cmd.add_event(&event, &mut screen.input_field, con);
            cmd = self.apply_command(writer, cmd, &mut screen, con)?;
            event_source.unblock(self.quitting);
        }

        self.end(writer)
    }
}
