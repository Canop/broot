use {
    super::*,
    crate::{
        browser::BrowserState,
        command::{parse_command_sequence, Command},
        conf::Conf,
        display::{Areas, Screen, W},
        errors::ProgramError,
        file_sizes, git,
        launchable::Launchable,
        skin::*,
        task_sync::Dam,
        verb::Internal,
    },
    crossterm::event::KeyModifiers,
    std::{
        io::Write,
        path::PathBuf,
    },
    strict::NonEmptyVec,
    termimad::{Event, EventSource},
};

const ESCAPE_TO_QUIT: bool = false;

/// The GUI
pub struct App {
    panels: NonEmptyVec<Panel>,
    active_panel_idx: usize,
    quitting: bool,
    launch_at_end: Option<Launchable>, // what must be launched after end
    created_panels_count: usize,
}

impl App {

    pub fn new(
        con: &AppContext,
        screen: &Screen,
    ) -> Result<App, ProgramError> {
        let panel = Panel::new(
            PanelId::from(0),
            Box::new(
                BrowserState::new(
                    con.launch_args.root.clone(),
                    con.launch_args.tree_options.clone(),
                    screen,
                    con,
                    &Dam::unlimited(),
                )?
                .expect("Failed to create BrowserState"),
            ),
            Areas::create(&mut Vec::new(), 0, screen)?,
            con,
        );
        Ok(App {
            active_panel_idx: 0,
            panels: panel.into(),
            quitting: false,
            launch_at_end: None,
            created_panels_count: 1,
        })
    }

    fn mut_state(&mut self) -> &mut dyn AppState {
        self.panels[self.active_panel_idx].mut_state()
    }
    fn panel(&self) -> &Panel {
        &self.panels[self.active_panel_idx]
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
        skin: &AppSkin,
        con: &AppContext,
    ) -> Result<(), ProgramError> {
        for (idx, panel) in self.panels.as_mut_slice().iter_mut().enumerate() {
            let focused = idx == self.active_panel_idx;
            let skin = if focused { &skin.focused } else { &skin.unfocused };
            panel.display(w, focused, screen, skin, con)?;
        }
        Ok(())
    }

    /// if there are exactly two panels, return the selection
    /// in the non focused panel
    fn get_other_panel_path(&self) -> Option<PathBuf> {
        if self.panels.len().get() == 2 {
            let non_focused_panel_idx = if self.active_panel_idx == 0 { 1 } else { 0 };
            Some(self.panels[non_focused_panel_idx].state().selected_path().to_path_buf())
        } else {
            None
        }
    }

    /// apply a command, and returns a command, which may be the same (modified or not)
    ///  or a new one.
    fn apply_command(
        &mut self,
        cmd: Command,
        screen: &mut Screen,
        panel_skin: &PanelSkin,
        con: &AppContext,
    ) -> Result<(), ProgramError> {
        use AppStateCmdResult::*;
        let mut error: Option<String> = None;
        let is_input_invocation = cmd.is_verb_invocated_from_input();
        let other_path = self.get_other_panel_path();
        match self.mut_panel().apply_command(
            &cmd,
            &other_path,
            screen,
            panel_skin,
            con,
        )? {
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
                        let new_input = self.panel().get_input_content();
                        let cmd = Command::from_raw(new_input, false);
                        self.mut_panel().apply_command(&cmd, &other_path, screen, panel_skin, con)?;
                    }
                } else {
                    self.quitting = true;
                }
            }
            DisplayError(txt) => {
                error = Some(txt);
            }
            Keep => {}
            Launch(launchable) => {
                self.launch_at_end = Some(*launchable);
                self.quitting = true;
            }
            NewPanel {
                state,
                purpose,
                direction,
            } => {
                if is_input_invocation {
                    self.mut_panel().clear_input();
                }
                let insertion_idx = if direction == HDir::Right {
                    self.active_panel_idx + 1
                } else {
                    self.active_panel_idx
                };
                match Areas::create(self.panels.as_mut_slice(), insertion_idx, screen) {
                    Ok(areas) => {
                        let mut panel = Panel::new(self.created_panels_count.into(), state, areas, con);
                        panel.purpose = purpose;
                        self.created_panels_count += 1;
                        self.panels.insert(insertion_idx, panel);
                        self.active_panel_idx = insertion_idx;
                    }
                    Err(e) => {
                        error = Some(e.to_string());
                    }
                }
            }
            NewState(state) => {
                self.mut_panel().clear_input();
                self.mut_panel().push_state(state);
                let other_path = self.get_other_panel_path();
                self.mut_panel().refresh_input_status(&other_path, con);
            }
            PopState => {
                if is_input_invocation {
                    self.mut_panel().clear_input();
                }
                if self.remove_state(screen) {
                    // should we set the cmd ?
                    self.mut_state().refresh(screen, con);
                } else if ESCAPE_TO_QUIT {
                    self.quitting = true;
                }
            }
            PopStateAndReapply => {
                if is_input_invocation {
                    self.mut_panel().clear_input();
                }
                if self.remove_state(screen) {
                    self.mut_panel().apply_command(&cmd, &other_path, screen, panel_skin, con)?;
                } else if ESCAPE_TO_QUIT {
                    self.quitting = true;
                }
            }
            Propagate(internal) => {
                let new_active_panel_idx = match internal {
                    Internal::panel_left if self.active_panel_idx > 0 => {
                        Some(self.active_panel_idx - 1)
                    }
                    Internal::panel_right if self.active_panel_idx + 1 < self.panels.len().get() => {
                        Some(self.active_panel_idx + 1)
                    }
                    _ => {
                        debug!("unhandled propagated internal. cmd={:?}", &cmd);
                        None
                    }
                };
                if let Some(idx) = new_active_panel_idx {
                    if is_input_invocation {
                        self.mut_panel().clear_input();
                    }
                    self.active_panel_idx = idx;
                    let other_path = self.get_other_panel_path();
                    self.mut_panel().refresh_input_status(&other_path, con);
                }
            }
            Quit => {
                self.quitting = true;
            }
            RefreshState { clear_cache } => {
                if is_input_invocation {
                    self.mut_panel().clear_input();
                }
                if clear_cache {
                    clear_caches();
                }
                for i in 0..self.panels.len().get() {
                    self.panels[i].mut_state().refresh(screen, con);
                }
            }
        }
        if let Some(text) = error {
            self.mut_panel().set_error(text);
        }
        Ok(())
    }

    fn clicked_panel_index(&self, x: u16, _y: u16, screen: &Screen) -> usize {
        let len = self.panels.len().get();
        (len * x as usize) / (screen.width as usize + 1)
    }

    fn do_pending_tasks(
        &mut self,
        screen: &mut Screen,
        con: &AppContext,
        dam: &mut Dam,
    ) -> Result<(), ProgramError> {
        // we start with the focused panel
        self.mut_panel().do_pending_tasks(screen, con, dam)?;
        // then the other ones
        for idx in 0..self.panels.len().get() {
            if idx != self.active_panel_idx {
                self.panels[idx].do_pending_tasks(screen, con, dam)?;
            }
        }
        Ok(())
    }

    /// This is the main loop of the application
    pub fn run(
        mut self,
        w: &mut W,
        screen: &mut Screen,
        con: &AppContext,
        conf: &Conf,
    ) -> Result<Option<Launchable>, ProgramError> {
        // we listen for events in a separate thread so that we can go on listening
        // when a long search is running, and interrupt it if needed
        let event_source = EventSource::new()?;
        let rx_events = event_source.receiver();
        let mut dam = Dam::from(rx_events);

        let skin = AppSkin::new(conf);

        // if some commands were passed to the application
        //  we execute them before even starting listening for events
        if let Some(unparsed_commands) = &con.launch_args.commands {
            for (input, arg_cmd) in parse_command_sequence(unparsed_commands, con)? {
                self.mut_panel().set_input_content(input);
                self.apply_command(arg_cmd, screen, &skin.focused, con)?;
                self.display_panels(w, screen, &skin, con)?;
                w.flush()?;
                self.do_pending_tasks(screen, con, &mut dam)?;
                self.display_panels(w, screen, &skin, con)?;
                w.flush()?;
                if self.quitting {
                    return Ok(self.launch_at_end.take());
                }
            }
        }

        self.display_panels(w, screen, &skin, con)?;
        w.flush()?;

        loop {
            if !self.quitting {
                self.do_pending_tasks(screen, con, &mut dam)?;
                self.display_panels(w, screen, &skin, con)?;
                w.flush()?;
            }
            let event = match dam.next_event() {
                Some(event) => event,
                None => {
                    // this is how we quit the application,
                    // when the input thread is properly closed
                    break;
                }
            };
            debug!("event: {:?}", &event);
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
                    self.apply_command(cmd, screen, &skin.focused, con)?;
                }
            }
            self.display_panels(w, screen, &skin, con)?;
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
