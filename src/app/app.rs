use {
    super::*,
    crate::{
        browser::BrowserState,
        command::{parse_command_sequence, Command},
        display::{Areas, Screen, W},
        errors::ProgramError,
        file_sizes, git,
        launchable::Launchable,
        task_sync::Dam,
    },
    crossterm::event::KeyModifiers,
    std::io::Write,
    strict::NonEmptyVec,
    termimad::{Event, EventSource},
};

/// The GUI
pub struct App {
    panels: NonEmptyVec<Panel>,
    active_panel_idx: usize,
    quitting: bool,
    launch_at_end: Option<Launchable>, // what must be launched after end
    created_panels_count: usize,
}

impl App {
    pub fn new(con: &AppContext, screen: &Screen) -> Result<App, ProgramError> {
        let panel = Panel::new(
            PanelId::from(0),
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
            screen,
        );
        Ok(App {
            active_panel_idx: 0,
            panels: panel.into(),
            quitting: false,
            launch_at_end: None,
            created_panels_count: 1,
        })
    }

    pub fn add_panel(
        &mut self,
        new_state: Box<dyn AppState>,
        purpose: PanelPurpose,
        areas: Areas,
        screen: &Screen,
    ) {
        let mut panel = Panel::new(self.created_panels_count.into(), new_state, areas, screen);
        panel.purpose = purpose;
        self.created_panels_count += 1;
        self.active_panel_idx = self.panels.len().get();
        self.panels.push(panel);
    }
    fn mut_state(&mut self) -> &mut dyn AppState {
        self.panels[self.active_panel_idx].mut_state()
    }
    fn mut_panel(&mut self) -> &mut Panel {
        unsafe {
            self.panels
                .as_mut_slice()
                .get_unchecked_mut(self.active_panel_idx)
        }
    }

    /// return true when the panel has been removed (ie it wasn't the last one)
    fn close_active_panel(&mut self, screen: &Screen) -> bool {
        if let Ok(_removed_panel) = self.panels.swap_remove(self.active_panel_idx) {
            self.active_panel_idx = self.panels.len().get() - 1;
            Areas::resize_all(self.panels.as_mut_slice(), screen)
                .expect("removing a panel should be easy");
            true
        } else {
            false // there's no other panel to go to
        }
    }
    fn remove_state(&mut self, screen: &Screen) -> bool {
        self.panels[self.active_panel_idx].remove_state() || self.close_active_panel(screen)
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

    /// apply a command, and returns a command, which may be the same (modified or not)
    ///  or a new one.
    /// This normally mutates self
    fn apply_command(
        &mut self,
        cmd: Command,
        screen: &mut Screen,
        con: &AppContext,
    ) -> Result<(), ProgramError> {
        use AppStateCmdResult::*;
        let mut error: Option<String> = None;
        let is_input_invocation = cmd.is_verb_invocated_from_input();
        match self.mut_panel().apply_command(&cmd, screen, con)? {
            Quit => {
                self.quitting = true;
            }
            Launch(launchable) => {
                self.launch_at_end = Some(*launchable);
                self.quitting = true;
            }
            NewPanel {
                state,
                purpose,
            } => {
                if is_input_invocation {
                    self.mut_panel().clear_input();
                }
                let insertion_idx = self.panels.len().get();
                match Areas::create(self.panels.as_mut_slice(), insertion_idx, screen) {
                    Ok(areas) => {
                        self.add_panel(state, purpose, areas, screen);
                    }
                    Err(e) => {
                        error = Some(e.to_string());
                    }
                }
            }
            NewState(state) => {
                self.mut_panel().clear_input();
                self.mut_panel().push_state(state);
            }
            RefreshState { clear_cache } => {
                if is_input_invocation {
                    self.mut_panel().clear_input();
                }
                if clear_cache {
                    clear_caches();
                }
                // should we set the cmd ?
                self.mut_state().refresh(screen, con);
            }
            PopState => {
                if is_input_invocation {
                    self.mut_panel().clear_input();
                }
                if self.remove_state(screen) {
                    // should we set the cmd ?
                    self.mut_state().refresh(screen, con);
                } else {
                    self.quitting = true;
                }
            }
            PopStateAndReapply => {
                if is_input_invocation {
                    self.mut_panel().clear_input();
                }
                if self.remove_state(screen) {
                    self.mut_panel().apply_command(&cmd, screen, con)?;
                } else {
                    self.quitting = true;
                }
            }
            ClosePanel { validate_purpose } => {
                let mut new_arg = None;
                if validate_purpose {
                    let purpose = &self.panels[self.active_panel_idx].purpose;
                    if let PanelPurpose::ArgEdition { .. } = purpose {
                        let path = self.panels[self.active_panel_idx].state().selected_path();
                        new_arg = Some(path.to_string_lossy().to_string());
                    }
                }
                if self.close_active_panel(screen) {
                    self.mut_state().refresh(screen, con);
                    if let Some(new_arg) = new_arg {
                        self.mut_panel().set_input_arg(new_arg);
                    }
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

    fn clicked_panel_index(&self, x: u16, _y: u16, screen: &Screen) -> usize {
        let len = self.panels.len().get();
        (len * x as usize) / (screen.width as usize + 1)
        //if idx < len { Some(idx) } else { None }
    }

    /// This is the main loop of the application
    pub fn run(
        mut self,
        w: &mut W,
        screen: &mut Screen,
        con: &AppContext,
    ) -> Result<Option<Launchable>, ProgramError> {
        // we listen for events in a separate thread so that we can go on listening
        // when a long search is running, and interrupt it if needed
        let event_source = EventSource::new()?;
        let rx_events = event_source.receiver();
        let mut dam = Dam::from(rx_events);

        // if some commands were passed to the application
        //  we execute them before even starting listening for events
        if let Some(unparsed_commands) = &con.launch_args.commands {
            for arg_cmd in parse_command_sequence(unparsed_commands, con)? {
                self.apply_command(arg_cmd, screen, con)?;
                self.mut_panel().do_pending_tasks(w, screen, con, &mut dam)?;
                if self.quitting {
                    return Ok(self.launch_at_end.take());
                }
            }
        }

        self.display_panels(w, screen, con)?;
        w.flush()?;

        loop {
            if !self.quitting {
                self.mut_panel().do_pending_tasks(w, screen, con, &mut dam)?;
            }
            let event = match dam.next_event() {
                Some(event) => event,
                None => {
                    // this is how we quit the application,
                    // when the input thread is properly closed
                    break;
                }
            };
            match event {
                Event::Click(x, y, KeyModifiers::NONE)
                    if self.clicked_panel_index(x, y, screen) != self.active_panel_idx =>
                {
                    // panel activation clic
                    // this will be cleaner when if let will be allowed in match guards with
                    // chaining (currently experimental)
                    self.active_panel_idx = self.clicked_panel_index(x, y, screen);
                }
                Event::Resize(w, h) => {
                    screen.set_terminal_size(w, h, con);
                    Areas::resize_all(self.panels.as_mut_slice(), screen)?;
                    for panel in &mut self.panels {
                        panel.mut_state().refresh(screen, con);
                    }
                }
                _ => {
                    // event handled by the panel
                    let cmd = self.mut_panel().add_event(w, event, con)?;
                    debug!("command after add_event: {:?}", &cmd);
                    self.apply_command(cmd, screen, con)?;
                }
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
