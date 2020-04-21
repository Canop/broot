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
            Areas,
            Screen,
            W,
        },
        task_sync::Dam,
    },
    crossterm::{
        event::KeyModifiers,
    },
    std::io::Write,
    strict::NonEmptyVec,
    super::{
        AppContext,
        AppState,
        AppStateCmdResult,
        Panel,
    },
    termimad::{
        Event,
        EventSource,
    },
};

pub struct App {
    panels: NonEmptyVec<Panel>,
    active_panel_idx: usize,
    quitting: bool,
    launch_at_end: Option<Launchable>, // what must be launched after end
}

impl App {
    pub fn new(
        con: & AppContext,
        screen: &Screen,
    ) -> Result<App, ProgramError> {
        let panel = Panel::new(
            Box::new(
                BrowserState::new(
                    con.launch_args.root.clone(),
                    con.launch_args.tree_options.clone(),
                    screen,
                    &Dam::unlimited(),
                )?
                .expect("Failed to create BrowserState"),
            ),
            Areas::create(&mut Vec::new(), 0, screen)?,
            Command::new(),
            screen,
        );
        Ok(App {
            active_panel_idx: 0,
            panels: panel.into(),
            quitting: false,
            launch_at_end: None,
        })
    }

    pub fn add_panel(
        &mut self,
        new_state: Box<dyn AppState>,
        areas: Areas,
        cmd: Command,
        screen: &Screen,
    ) {
        let panel = Panel::new(new_state, areas, cmd, screen);
        self.active_panel_idx = self.panels.len().get();
        self.panels.push(panel);
    }
    pub fn push_state(&mut self, new_state: Box<dyn AppState>) {
        self.panels[self.active_panel_idx].push(new_state);
    }
    fn mut_state(&mut self) -> &mut dyn AppState {
        self.panels[self.active_panel_idx].mut_state()
    }
    fn mut_panel(&mut self) -> &mut Panel {
        unsafe {
            self.panels.as_mut_slice().get_unchecked_mut(self.active_panel_idx)
        }
    }

    /// return true when the panel has been removed (ie it wasn't the last one)
    fn close_active_panel(&mut self, screen: &Screen) -> bool {
        if let Ok(_removed_panel) = self.panels.swap_remove(self.active_panel_idx) {
            //let parent_idx = _removed_panel.parent_panel_idx;
            // FIXME we can't use the parent_idx... we change the idx when we remove
            // elements. But we must store in the panel the parent id
            self.active_panel_idx = self.panels.len().get() - 1;
            Areas::resize_all(
                self.panels.as_mut_slice(),
                screen,
            ).expect("removing a panel should be easy");
            true
        } else {
            false // there's no other panel to go to
        }
    }
    fn remove_state(&mut self, screen: &Screen) -> bool {
        self.panels[self.active_panel_idx].remove_state()
            || self.close_active_panel(screen)
    }

    fn display_panels(
        &mut self,
        w: &mut W,
        screen: &mut Screen,
        con: &AppContext,
    ) -> Result<(), ProgramError> {
        for (idx, panel) in self.panels.as_mut_slice().iter_mut().enumerate() {
            panel.display(w, idx == self.active_panel_idx, screen, con)?;
        }
        Ok(())
    }

    /// execute all the pending tasks until there's none remaining or
    ///  the dam asks for interruption
    fn do_pending_tasks(
        &mut self,
        w: &mut W,
        screen: &mut Screen,
        con: &AppContext,
        dam: &mut Dam,
    ) -> Result<(), ProgramError> {
        while self.mut_state().has_pending_task() & !dam.has_event() {
            self.mut_state().do_pending_task(screen, dam);
            self.display_panels(w, screen, con)?;
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
        cmd: Command,
        screen: &mut Screen,
        con: &AppContext,
    ) -> Result<(), ProgramError> {
        use AppStateCmdResult::*;
        let mut error: Option<String> = None;
        match self.mut_panel().apply_command(cmd, screen, con)? {
            Quit => {
                self.quitting = true;
            }
            Launch(launchable) => {
                self.launch_at_end = Some(*launchable);
                self.quitting = true;
            }
            NewState{ state, cmd: new_cmd, in_new_panel } => {
                if in_new_panel {
                    let insertion_idx = self.panels.len().get();
                    match Areas::create(
                        self.panels.as_mut_slice(),
                        insertion_idx,
                        screen,
                    ) {
                        Ok(areas) => {
                            self.add_panel(state, areas, new_cmd, screen);
                        }
                        Err(e) => {
                            error = Some(e.to_string());
                        }
                    }
                } else {
                    self.push_state(state);
                    // FIXME should we set the cmd ?
                }
            }
            RefreshState { clear_cache } => {
                if clear_cache {
                    clear_caches();
                }
                // should we set the cmd ?
                self.mut_state().refresh(screen, con);
                //self.mut_panel().cmd = self.mut_state().refresh(screen, con);
            }
            PopState => {
                if self.remove_state(screen) {
                    // should we set the cmd ?
                    self.mut_state().refresh(screen, con);
                } else {
                    self.quitting = true;
                }
            }
            PopStateAndReapply => {
                if self.remove_state(screen) {
                    let cmd = self.mut_panel().get_command();
                    // FIXME check this
                    self.mut_panel().apply_command(cmd, screen, con)?;
                } else {
                    self.quitting = true;
                }
            }
            PopPanel => {
                if self.close_active_panel(screen) {
                    // FIXME check this
                    self.mut_state().refresh(screen, con);
                } else {
                    self.quitting = true;
                }
            }
            DisplayError(txt) => {
                error = Some(txt);
            }
            Keep => {}
        }
        if let Some(text) = error {
            self.mut_panel().set_error(text);
        }
        // FIXME flags
        //self.state().write_flags(w, screen, con)?;
        Ok(())
    }

    fn clicked_panel_index(&self, x: u16, y: u16, screen: &Screen) -> usize {
        (self.panels.len().get() * x as usize ) / (screen.width as usize + 1)
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

        // if some commands were passed to the application
        //  we execute them before even starting listening for events
        if let Some(unparsed_commands) = &con.launch_args.commands {
            for arg_cmd in parse_command_sequence(unparsed_commands, con)? {
                self.apply_command(w, arg_cmd, screen, con)?;
                self.do_pending_tasks(
                    w,
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
        // FIXME rétablir flags (en mieux)
        //self.mut_state().write_flags(w, screen, con)?;
        w.flush()?;

        loop {
            if !self.quitting {
                self.do_pending_tasks(w, screen, con, &mut dam)?;
            }
            let event = match dam.next_event() {
                Some(event) => event,
                None => {
                    // this is how we quit the application,
                    // when the input thread is properly closed
                    break;
                }
            };
            // we first check the event isn't simple left a click
            // in a inactive panel
            // FIXME new crossterm
            let mut ignore_event = false;
            if let Event::Click(x, y, modifiers) = event {
                if modifiers == KeyModifiers::empty() {
                    let clicked_panel_idx = self.clicked_panel_index(x, y, screen);
                    if clicked_panel_idx != self.active_panel_idx {
                        self.active_panel_idx = clicked_panel_idx;
                        ignore_event = true;
                    }
                }
            }
            if !ignore_event {
                let cmd = self.mut_panel().add_event(w, event, con)?;
                debug!("command after add_event: {:?}", &cmd);
                self.apply_command(w, cmd, screen, con)?;
            }
            self.display_panels(w, screen, con)?;
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
