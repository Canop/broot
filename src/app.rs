//! broot's app is mainly a stack of AppState.
//! Commands parsed from the input are submitted to the current
//! appstate, which replies with a stateCmdResult which may
//! be
//! - a transition to a new state
//! - a pop to get back to the previous one
//! - an operation which keeps the state
//! - a request to quit broot
//! - a request to launch an executable (thus leaving broot)

use {
    crate::{
        app_context::AppContext,
        app_state::{AppState, AppStateCmdResult},
        browser_states::BrowserState,
        command_parsing::parse_command_sequence,
        commands::Command,
        errors::ProgramError,
        external::Launchable,
        file_sizes,
        io::W,
        screens::Screen,
        skin::Skin,
        status::Status,
        task_sync::TaskLifetime,
    },
    crossterm::{
        cursor,
        event::{DisableMouseCapture, EnableMouseCapture},
        terminal::{EnterAlternateScreen, LeaveAlternateScreen},
        QueueableCommand,
    },
    minimad::Composite,
    std::io::Write,
    termimad::EventSource,
};

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

    fn mut_state(&mut self) -> &mut dyn AppState {
        self.states
            .last_mut()
            .expect("No path has been pushed")
            .as_mut()
    }
    fn state(&self) -> &dyn AppState {
        self.states
            .last()
            .expect("No path has been pushed")
            .as_ref()
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
        let state = self.mut_state();
        while state.has_pending_task() & !tl.is_expired() {
            state.do_pending_task(screen, &tl);
            state.display(w, screen, con)?;
            state.write_status(w, cmd, &screen, con)?;
        }
        Ok(())
    }

    /// apply a command, and returns a command, which may be the same (modified or not)
    ///  or a new one.
    /// This normally mutates self
    fn apply_command(
        &mut self,
        w: &mut W,
        mut cmd: Command,
        screen: &mut Screen,
        con: &AppContext,
    ) -> Result<Command, ProgramError> {
        debug!("action: {:?}", &cmd.action);
        let mut error: Option<String> = None;
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
            }
            AppStateCmdResult::RefreshState { clear_cache } => {
                if clear_cache {
                    file_sizes::clear_cache();
                }
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
                error = Some(txt);
            }
            _ => {}
        }
        self.mut_state().display(w, screen, con)?;
        match error {
            Some(text) => Status::from_error(Composite::from_inline(&text)).display(w, screen)?,
            None => self.state().write_status(w, &cmd, screen, con)?,
        }
        screen.input_field.set_content(&cmd.raw);
        screen.input_field.display_on(w)?;
        self.state().write_flags(w, screen, con)?;
        Ok(cmd)
    }

    /// called exactly once at end of `run`, cleans the writer (which
    /// is usually stdout or stderr)
    fn end(self, writer: &mut W) -> Result<Option<Launchable>, ProgramError> {
        writer.queue(DisableMouseCapture)?;
        writer.queue(cursor::Show)?;
        writer.queue(LeaveAlternateScreen)?;
        writer.flush()?;
        debug!("we left the screen");
        Ok(self.launch_at_end)
    }

    /// This is the main loop of the application
    pub fn run(
        mut self,
        writer: &mut W,
        con: &AppContext,
        skin: Skin,
    ) -> Result<Option<Launchable>, ProgramError> {
        writer.queue(EnterAlternateScreen)?;
        writer.queue(cursor::Hide)?;
        debug!("we're on screen");
        let mut screen = Screen::new(con, skin)?;

        // we listen for events in a separate thread so that we can go on listening
        // when a long search is running, and interrupt it if needed
        writer.queue(EnableMouseCapture)?;
        let event_source = EventSource::new()?;
        let rx_events = event_source.receiver();

        self.push(Box::new(
            BrowserState::new(
                con.launch_args.root.clone(),
                con.launch_args.tree_options.clone(),
                &screen,
                &TaskLifetime::unlimited(),
            )?
            .expect("Failed to create BrowserState"),
        ));

        let mut cmd = Command::new();

        // if some commands were passed to the application
        //  we execute them before even starting listening for events
        if let Some(unparsed_commands) = &con.launch_args.commands {
            let lifetime = TaskLifetime::unlimited();

            for arg_cmd in parse_command_sequence(unparsed_commands, con)? {
                cmd = self.apply_command(writer, arg_cmd, &mut screen, con)?;
                self.do_pending_tasks(writer, &cmd, &mut screen, con, lifetime.clone())?;
                if self.quitting {
                    return self.end(writer);
                }
            }
        }

        let state = self.mut_state();
        state.display(writer, &screen, con)?;
        state.write_status(writer, &cmd, &screen, con)?;
        state.write_flags(writer, &mut screen, con)?;

        screen.input_field.display_on(writer)?;

        for event in rx_events {
            cmd.add_event(&event, &mut screen.input_field, con, self.state());
            cmd = self.apply_command(writer, cmd, &mut screen, con)?;
            event_source.unblock(self.quitting);

            if !self.quitting {
                self.do_pending_tasks(
                    writer,
                    &cmd,
                    &mut screen,
                    con,
                    TaskLifetime::new(event_source.shared_event_count()),
                )?;
            }
        }

        self.end(writer)
    }
}
