use {
    super::*,
    crate::{
        browser::BrowserState,
        command::{Command, Sequence},
        conf::Conf,
        display::{Areas, Screen, W},
        errors::ProgramError,
        file_sum, git,
        launchable::Launchable,
        skin::*,
        task_sync::{Dam, Either},
        verb::Internal,
    },
    crossbeam::channel::{
        Receiver,
        Sender,
        unbounded,
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

#[cfg(feature = "client-server")]
use std::sync::{Arc, Mutex};

/// The GUI
pub struct App {
    /// dimensions of the screen
    screen: Screen,

    /// the panels of the application, at least one
    panels: NonEmptyVec<Panel>,

    /// index of the currently focused panel
    active_panel_idx: usize,

    /// whether the app is in the (uncancellable) process of quitting
    quitting: bool,

    /// what must be done after having closed the TUI
    launch_at_end: Option<Launchable>,

    /// a count of all panels created
    created_panels_count: usize,

    /// the panel dedicated to preview, if any
    preview: Option<PanelId>,

    /// the root of the active panel
    #[cfg(feature = "client-server")]
    root: Arc<Mutex<PathBuf>>,

    /// sender to the sequence channel
    tx_seqs: Sender<Sequence>,

    /// receiver to listen to the sequence channel
    rx_seqs: Receiver<Sequence>,
}

impl App {

    pub fn new(
        con: &AppContext,
    ) -> Result<App, ProgramError> {
        let screen = Screen::new(con)?;
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
            Areas::create(&mut Vec::new(), 0, screen, false)?,
            con,
        );
        let (tx_seqs, rx_seqs) = unbounded::<Sequence>();
        Ok(App {
            screen,
            active_panel_idx: 0,
            panels: panel.into(),
            quitting: false,
            launch_at_end: None,
            created_panels_count: 1,
            preview: None,

            #[cfg(feature = "client-server")]
            root: Arc::new(Mutex::new(con.launch_args.root.clone())),
            tx_seqs,
            rx_seqs,
        })
    }

    fn panel_ref_to_idx(&self, panel_ref: PanelReference) -> Option<usize> {
        match panel_ref {
            PanelReference::Active => Some(self.active_panel_idx),
            PanelReference::Leftest => Some(0),
            PanelReference::Rightest => Some(self.panels.len().get() - 1),
            PanelReference::Id(id) => self.panel_id_to_idx(id),
            PanelReference::Preview => self.preview.and_then(|id| self.panel_id_to_idx(id)),
        }
    }

    /// return the current index of the panel whith given id
    fn panel_id_to_idx(&self, id: PanelId) -> Option<usize> {
        self.panels.iter().position(|panel| panel.id == id)
    }

    fn state(&self) -> &dyn AppState {
        self.panels[self.active_panel_idx].state()
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

    /// close the panel if it's not the last one
    ///
    /// Return true when the panel has been removed (ie it wasn't the last one)
    fn close_panel(&mut self, panel_idx: usize) -> bool {
        let active_panel_id = self.panels[self.active_panel_idx].id;
        if let Some(preview_id) = self.preview {
            if self.panels.has_len(2) && self.panels[panel_idx].id != preview_id {
                // we don't want to stay with just the preview
                return false;
            }
        }
        if let Ok(removed_panel) = self.panels.remove(panel_idx) {
            if self.preview == Some(removed_panel.id) {
                self.preview = None;
            }
            Areas::resize_all(
                self.panels.as_mut_slice(),
                self.screen,
                self.preview.is_some(),
            )
            .expect("removing a panel should be easy");
            self.active_panel_idx = self
                .panels
                .iter()
                .position(|p| p.id == active_panel_id)
                .unwrap_or(self.panels.len().get() - 1);
            true
        } else {
            false // there's no other panel to go to
        }
    }

    /// remove the top state of the current panel
    ///
    /// Close the panel too if that was its only state.
    /// Close nothing and return false if there's not
    /// at least two states in the app.
    fn remove_state(&mut self) -> bool {
        self.panels[self.active_panel_idx].remove_state()
            || self.close_panel(self.active_panel_idx)
    }

    /// redraw the whole screen. All drawing
    /// are supposed to happen here, and only here.
    fn display_panels(
        &mut self,
        w: &mut W,
        skin: &AppSkin,
        con: &AppContext,
    ) -> Result<(), ProgramError> {
        // if some images are displayed by kitty, we'll erase it,
        // but only after having displayed the new ones (if any)
        // to prevent some flickerings
        #[cfg(unix)]
        let previous_images = crate::kitty::image_renderer()
            .as_ref()
            .and_then(|renderer| {
                let mut renderer = renderer.lock().unwrap();
                renderer.take_current_images()
            });
        for (idx, panel) in self.panels.as_mut_slice().iter_mut().enumerate() {
            let focused = idx == self.active_panel_idx;
            let skin = if focused { &skin.focused } else { &skin.unfocused };
            time!(
                "display panel",
                panel.display(w, focused, self.screen, skin, con)?,
            );
        }
        #[cfg(unix)]
        if let Some(previous_images) = previous_images {
            if let Some(renderer) = crate::kitty::image_renderer().as_ref() {
                let mut renderer = renderer.lock().unwrap();
                renderer.erase(w, previous_images)?;
            }
        }
        w.flush()?;
        Ok(())
    }

    /// if there are exactly two non preview panels, return the selection
    /// in the non focused panel
    fn get_other_panel_path(&self) -> Option<PathBuf> {
        let len = self.panels.len().get();
        if len == 3 {
            if let Some(preview_id) = self.preview {
                for (idx, panel) in self.panels.iter().enumerate() {
                    if self.active_panel_idx != idx && panel.id != preview_id {
                        return Some(panel.state().selected_path().to_path_buf());
                    }
                }
            }
            None
        } else if self.panels.len().get() == 2 && self.preview.is_none() {
            let non_focused_panel_idx = if self.active_panel_idx == 0 { 1 } else { 0 };
            Some(self.panels[non_focused_panel_idx].state().selected_path().to_path_buf())
        } else {
            None
        }
    }

    /// apply a command. Change the states but don't redraw on screen.
    fn apply_command(
        &mut self,
        w: &mut W,
        cmd: Command,
        panel_skin: &PanelSkin,
        con: &AppContext,
    ) -> Result<(), ProgramError> {
        use AppStateCmdResult::*;
        let mut error: Option<String> = None;
        let is_input_invocation = cmd.is_verb_invocated_from_input();
        let other_path = self.get_other_panel_path();
        let preview = self.preview;
        let screen = self.screen; // it can't change in this function
        match self.mut_panel().apply_command(
            w,
            &cmd,
            &other_path,
            screen,
            panel_skin,
            preview,
            con,
        )? {
            ApplyOnPanel { id } => {
                if let Some(idx) = self.panel_id_to_idx(id) {
                    if let DisplayError(txt) = self.panels[idx].apply_command(
                        w,
                        &cmd,
                        &other_path, // unsure...
                        screen,
                        panel_skin,
                        preview,
                        con,
                    )? {
                        // we should probably handle other results
                        // which implies the possibility of a recursion
                        error = Some(txt);
                    } else if is_input_invocation {
                        self.mut_panel().clear_input();
                    }
                } else {
                    warn!("no panel found for ApplyOnPanel");
                }
            }
            ClosePanel { validate_purpose, panel_ref } => {
                if is_input_invocation {
                    self.mut_panel().clear_input_invocation();
                }
                let close_idx = self.panel_ref_to_idx(panel_ref)
                    .unwrap_or_else(||
                        // when there's a preview panel, we close it rather than the app
                        if self.panels.len().get()==2 && self.preview.is_some() {
                            1
                        } else {
                            self.active_panel_idx
                        }
                    );
                let mut new_arg = None;
                if validate_purpose {
                    let purpose = &self.panels[close_idx].purpose;
                    if let PanelPurpose::ArgEdition { .. } = purpose {
                        let path = self.panels[close_idx].state().selected_path();
                        new_arg = Some(path.to_string_lossy().to_string());
                    }
                }
                if self.close_panel(close_idx) {
                    self.mut_state().refresh(screen, con);
                    if let Some(new_arg) = new_arg {
                        self.mut_panel().set_input_arg(new_arg);
                        let new_input = self.panel().get_input_content();
                        let cmd = Command::from_raw(new_input, false);
                        let preview = self.preview;
                        self.mut_panel().apply_command(
                            w,
                            &cmd,
                            &other_path,
                            screen,
                            panel_skin,
                            preview,
                            con,
                        )?;
                    }
                } else {
                    self.quitting = true;
                }
            }
            DisplayError(txt) => {
                error = Some(txt);
            }
            ExecuteSequence { sequence } => {
                self.tx_seqs.send(sequence).unwrap();
            }
            HandleInApp(internal) => {
                let new_active_panel_idx = match internal {
                    Internal::panel_left => {
                        // we're not here to create a panel, this is handled in the state
                        // we're here because the state wants us to either move to the panel
                        // to the left, or close the rightest one
                        if self.active_panel_idx == 0 {
                            self.close_panel(self.panels.len().get()-1);
                            None
                        } else {
                            Some(self.active_panel_idx - 1)
                        }
                    }
                    Internal::panel_right => {
                        // we're not here to create panels (it's done in the state).
                        // So we either move to the right or close the leftes panel
                        if self.active_panel_idx + 1 == self.panels.len().get() {
                            self.close_panel(0);
                            None
                        } else {
                            Some(self.active_panel_idx + 1)
                        }
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
            Keep => {
                if is_input_invocation {
                    self.mut_panel().clear_input_invocation();
                }
            }
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
                    self.mut_panel().clear_input_invocation();
                }
                let insertion_idx = if purpose.is_preview() {
                    self.panels.len().get()
                } else if direction == HDir::Right {
                    self.active_panel_idx + 1
                } else {
                    self.active_panel_idx
                };
                let with_preview = purpose.is_preview() || self.preview.is_some();
                match Areas::create(
                    self.panels.as_mut_slice(),
                    insertion_idx,
                    screen,
                    with_preview,
                ) {
                    Ok(areas) => {
                        let panel_id = self.created_panels_count.into();
                        let mut panel = Panel::new(panel_id, state, areas, con);
                        panel.purpose = purpose;
                        self.created_panels_count += 1;
                        self.panels.insert(insertion_idx, panel);
                        if purpose.is_preview() {
                            debug_assert!(self.preview.is_none());
                            self.preview = Some(panel_id);
                        } else {
                            self.active_panel_idx = insertion_idx;
                        }
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
                if self.remove_state() {
                    self.mut_state().refresh(screen, con);
                    let other_path = self.get_other_panel_path();
                    self.mut_panel().refresh_input_status(&other_path, con);
                } else if ESCAPE_TO_QUIT {
                    self.quitting = true;
                }
            }
            PopStateAndReapply => {
                if is_input_invocation {
                    self.mut_panel().clear_input();
                }
                if self.remove_state() {
                    let preview = self.preview;
                    self.mut_panel().apply_command(
                        w,
                        &cmd,
                        &other_path,
                        screen,
                        panel_skin,
                        preview,
                        con,
                    )?;
                } else if ESCAPE_TO_QUIT {
                    self.quitting = true;
                }
            }
            Quit => {
                self.quitting = true;
            }
            RefreshState { clear_cache } => {
                if is_input_invocation {
                    self.mut_panel().clear_input_invocation();
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
        self.update_preview(con);

        #[cfg(feature="client-server")]
        if let Ok(mut root) = self.root.lock() { // when does this not work ?
            *root = self.state().selected_path().to_path_buf();
        }

        Ok(())
    }

    /// update the state of the preview, if there's some
    fn update_preview(&mut self, con: &AppContext) {
        let preview_idx = self.preview.and_then(|id| self.panel_id_to_idx(id));
        if let Some(preview_idx) = preview_idx {
            let path = self.state().selected_path();
            let old_path = self.panels[preview_idx].state().selected_path();
            if path != old_path && path.is_file() {
                let path = path.to_path_buf();
                self.panels[preview_idx].mut_state().set_selected_path(path.into(), con);
            }
        }
    }

    /// get the index of the panel at x
    fn clicked_panel_index(&self, x: u16, _y: u16) -> usize {
        let len = self.panels.len().get();
        (len * x as usize) / (self.screen.width as usize + 1)
    }

    /// do the pending tasks, if any, and refresh the screen accordingly
    fn do_pending_tasks(
        &mut self,
        w: &mut W,
        skin: &AppSkin,
        dam: &mut Dam,
        con: &AppContext,
    ) -> Result<(), ProgramError> {
        while self.has_pending_task() && !dam.has_event() {
            if self.do_pending_task(con, dam) {
                self.update_preview(con); // the selection may have changed
                let other_path = self.get_other_panel_path();
                self.mut_panel().refresh_input_status(&other_path, con);
                self.display_panels(w, &skin, con)?;
            } else {
                warn!("unexpected lack of update on do_pending_task");
                return Ok(());
            }
        }
        Ok(())
    }

    /// do the next pending task
    fn do_pending_task(
        &mut self,
        con: &AppContext,
        dam: &mut Dam,
    ) -> bool {
        let screen = self.screen;
        // we start with the focused panel
        if self.panel().has_pending_task() {
            self.mut_panel().do_pending_task(screen, con, dam);
            return true;
        }
        // then the other ones
        for idx in 0..self.panels.len().get() {
            if idx != self.active_panel_idx {
                let panel = &mut self.panels[idx];
                if panel.has_pending_task() {
                    panel.do_pending_task(screen, con, dam);
                    return true;
                }
            }
        }
        false
    }

    fn has_pending_task(&mut self) -> bool {
        self.panels.iter().any(|p| p.has_pending_task())
    }

    /// This is the main loop of the application
    pub fn run(
        mut self,
        w: &mut W,
        con: &AppContext,
        conf: &Conf,
    ) -> Result<Option<Launchable>, ProgramError> {
        // we listen for events in a separate thread so that we can go on listening
        // when a long search is running, and interrupt it if needed
        let event_source = EventSource::new()?;
        let rx_events = event_source.receiver();
        let mut dam = Dam::from(rx_events);

        let skin = AppSkin::new(conf, con.launch_args.no_style);

        self.screen.clear_bottom_right_char(w, &skin.focused)?;

        if let Some(raw_sequence) = &con.launch_args.commands {
            self.tx_seqs
                .send(Sequence::new_local(raw_sequence.to_string()))
                .unwrap();
        }

        #[cfg(feature="client-server")]
        let _server = con.launch_args.listen.as_ref()
            .map(|server_name| crate::net::Server::new(
                &server_name,
                self.tx_seqs.clone(),
                Arc::clone(&self.root),
            ))
            .transpose()?;

        loop {
            if !self.quitting {
                self.display_panels(w, &skin, con)?;
                time!(
                    "pending_tasks",
                    self.do_pending_tasks(w, &skin, &mut dam, con)?,
                );
            }
            match dam.next(&self.rx_seqs) {
                Either::First(Some(event)) => {
                    info!("event: {:?}", &event);
                    match event {
                        Event::Click(x, y, KeyModifiers::NONE)
                            if self.clicked_panel_index(x, y) != self.active_panel_idx =>
                        {
                            // panel activation click
                            // this will be cleaner when if let will be allowed in match guards with
                            // chaining (currently experimental)
                            self.active_panel_idx = self.clicked_panel_index(x, y);
                        }
                        Event::Resize(w, h) => {
                            self.screen.set_terminal_size(w, h, con);
                            Areas::resize_all(
                                self.panels.as_mut_slice(),
                                self.screen,
                                self.preview.is_some(),
                            )?;
                            for panel in &mut self.panels {
                                panel.mut_state().refresh(self.screen, con);
                            }
                        }
                        _ => {
                            // event handled by the panel
                            let cmd = self.mut_panel().add_event(w, event, con)?;
                            debug!("command after add_event: {:?}", &cmd);
                            self.apply_command(w, cmd, &skin.focused, con)?;
                        }
                    }
                    event_source.unblock(self.quitting);
                }
                Either::First(None) => {
                    // this is how we quit the application,
                    // when the input thread is properly closed
                    break;
                }
                Either::Second(Some(raw_sequence)) => {
                    debug!("got command sequence: {:?}", &raw_sequence);
                    for (input, arg_cmd) in raw_sequence.parse(con)? {
                        self.mut_panel().set_input_content(&input);
                        self.apply_command(w, arg_cmd, &skin.focused, con)?;
                        if self.quitting {
                            // is that a 100% safe way of quitting ?
                            return Ok(self.launch_at_end.take());
                        } else {
                            self.display_panels(w, &skin, con)?;
                            time!(
                                "sequence pending tasks",
                                self.do_pending_tasks(w, &skin, &mut dam, con)?,
                            );
                        }
                    }
                }
                Either::Second(None) => {
                    warn!("I didn't expect a None to occur here");
                }
            }
        }

        Ok(self.launch_at_end.take())
    }
}

/// clear the file sizes and git stats cache.
/// This should be done on Refresh actions and after any external
/// command.
fn clear_caches() {
    file_sum::clear_cache();
    git::clear_status_computer_cache();
    #[cfg(unix)]
    crate::filesystems::clear_cache();
}
