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
        browser::BrowserState,
        command::{
            Command,
            parse_command_sequence,
        },
        errors::ProgramError,
        launchable::Launchable,
        file_sizes,
        git,
        display::{
            Screen,
            Status,
            W,
        },
        task_sync::Dam,
    },
    minimad::Composite,
    std::io::Write,
    strict::NonEmptyVec,
    super::{
        AppContext,
        AppState,
        AppStateCmdResult,
        StatePanel,
    },
    termimad::EventSource,
};

pub struct App {
    panels: NonEmptyVec<StatePanel>,
    active_panel_idx: usize,
    quitting: bool,
    launch_at_end: Option<Launchable>, // what must be launched after end
}

impl App {
    pub fn new(
        con: & AppContext,
        screen: &Screen,
    ) -> Result<App, ProgramError> {
        let panel = StatePanel::new(
            Box::new(
                BrowserState::new(
                    con.launch_args.root.clone(),
                    con.launch_args.tree_options.clone(),
                    screen,
                    &Dam::unlimited(),
                )?
                .expect("Failed to create BrowserState"),
            ),
        );
        Ok(App {
            active_panel_idx: 0,
            panels: panel.into(),
            quitting: false,
            launch_at_end: None,
        })
    }

    pub fn add_panel(&mut self, new_state: Box<dyn AppState>) {
        let panel = StatePanel::new(new_state);
        self.active_panel_idx = self.panels.len().get();
        self.panels.push(panel);
    }

    pub fn push_state(&mut self, new_state: Box<dyn AppState>, in_new_panel: bool) {
        debug!("push_state in_new_panel={:?}", in_new_panel);
        if in_new_panel {
            self.add_panel(new_state);
        } else {
            self.panels[self.active_panel_idx].push(new_state);
        }
    }
    fn state(&self) -> &dyn AppState {
        self.panels[self.active_panel_idx].state()
    }
    fn mut_state(&mut self) -> &mut dyn AppState {
        self.panels[self.active_panel_idx].mut_state()
    }
    /// return true when the panel has been removed (ie it wasn't the last one)
    fn close_active_panel(&mut self) -> bool {
        if let Ok(_removed_panel) = self.panels.swap_remove(self.active_panel_idx) {
            //let parent_idx = _removed_panel.parent_panel_idx;
            // FIXME we can't use the parent_idx... we change the idx when we remove
            // elements. But we must store in the panel the parent id
            self.active_panel_idx = self.panels.len().get() - 1;
            true
        } else {
            false // there's no other panel to go to
        }
    }
    fn remove_state(&mut self) -> bool {
        self.panels[self.active_panel_idx].remove_state() || self.close_active_panel()
    }

    fn display_panels(
        &mut self,
        w: &mut W,
        screen: &mut Screen,
        con: &AppContext,
    ) -> Result<(), ProgramError> {
        let mut areas = screen.panel_areas(&self.panels);
        for (i, area) in areas.drain(..).enumerate() {
            self.panels[i].mut_state().display(w, screen, area, con)?;
        }
        Ok(())
    }

    /// execute all the pending tasks until there's none remaining or
    ///  the dam asks for interruption
    fn do_pending_tasks(
        &mut self,
        w: &mut W,
        cmd: &Command,
        screen: &mut Screen,
        con: &AppContext,
        dam: &mut Dam,
    ) -> Result<(), ProgramError> {
        while self.mut_state().has_pending_task() & !dam.has_event() {
            self.mut_state().do_pending_task(screen, dam);
            self.display_panels(w, screen, con)?;
            self.mut_state().write_status(w, cmd, screen, con)?;
            w.flush()?;
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
        use AppStateCmdResult::*;
        debug!("action: {:?}", &cmd.action);
        let mut error: Option<String> = None;
        match self.mut_state().apply(&mut cmd, screen, con)? {
            Quit => {
                self.quitting = true;
            }
            Launch(launchable) => {
                self.launch_at_end = Some(*launchable);
                self.quitting = true;
            }
            NewState{ state, cmd: new_cmd, in_new_panel } => {
                self.push_state(state, in_new_panel);
                cmd = new_cmd;
            }
            RefreshState { clear_cache } => {
                if clear_cache {
                    clear_caches();
                }
                cmd = self.mut_state().refresh(screen, con);
            }
            PopState => {
                if self.remove_state() {
                    cmd = self.mut_state().refresh(screen, con);
                } else {
                    self.quitting = true;
                }
            }
            PopStateAndReapply => {
                if self.remove_state() {
                    return self.apply_command(w, cmd, screen, con);
                } else {
                    self.quitting = true;
                }
            }
            PopPanel => {
                if self.close_active_panel() {
                    cmd = self.mut_state().refresh(screen, con);
                } else {
                    self.quitting = true;
                }
            }
            DisplayError(txt) => {
                error = Some(txt);
            }
            Keep => {}
        }
        self.display_panels(w, screen, con)?;
        match error {
            Some(text) => Status::from_error(Composite::from_inline(&text)).display(w, screen)?,
            None => self.state().write_status(w, &cmd, screen, con)?,
        }
        screen.input_field.set_content(&cmd.raw);
        screen.input_field.display_on(w)?;
        self.state().write_flags(w, screen, con)?;
        w.flush()?;
        Ok(cmd)
    }

    /// This is the main loop of the application
    pub fn run(
        mut self,
        w: &mut W,
        screen: &mut Screen,
        con: &AppContext,
    ) -> Result<Option<Launchable>, ProgramError> {
        debug!("we're on screen");

        // we listen for events in a separate thread so that we can go on listening
        // when a long search is running, and interrupt it if needed
        let event_source = EventSource::new()?;
        let rx_events = event_source.receiver();
        let mut dam = Dam::from(rx_events);

        let mut cmd = Command::new();

        // if some commands were passed to the application
        //  we execute them before even starting listening for events
        if let Some(unparsed_commands) = &con.launch_args.commands {
            for arg_cmd in parse_command_sequence(unparsed_commands, con)? {
                cmd = self.apply_command(w, arg_cmd, screen, con)?;
                self.do_pending_tasks(
                    w,
                    &cmd,
                    screen,
                    con,
                    &mut dam,
                )?;
                if self.quitting {
                    return Ok(self.launch_at_end.take());
                }
            }
        }

        self.display_panels(w, screen, con)?;
        self.mut_state().write_status(w, &cmd, screen, con)?;
        self.mut_state().write_flags(w, screen, con)?;
        screen.input_field.display_on(w)?;
        w.flush()?;

        loop {
            if !self.quitting {
                self.do_pending_tasks(w, &cmd, screen, con, &mut dam)?;
            }
            let event = match dam.next_event() {
                Some(event) => event,
                None => {
                    // this is how we quit the application,
                    // when the input thread is properly closed
                    break;
                }
            };
            cmd.add_event(&event, &mut screen.input_field, con, self.state());
            debug!("command after add_event: {:?}", &cmd);
            cmd = self.apply_command(w, cmd, screen, con)?;
            w.flush()?;
            event_source.unblock(self.quitting);
        }

        Ok(self.launch_at_end.take())
    }
}

/// clear the file sizes and git stats cache.
/// This should be done on Refresh actions and after any external
/// command.
fn clear_caches() {
    file_sizes::clear_cache();
    git::clear_status_computer_cache();
}
