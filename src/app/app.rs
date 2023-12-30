use {
    super::*,
    crate::{
        browser::BrowserState,
        cli::TriBool,
        command::{
            Command,
            Sequence,
        },
        conf::Conf,
        display::{
            Areas,
            Screen,
            W,
        },
        errors::ProgramError,
        file_sum,
        git,
        kitty,
        launchable::Launchable,
        path::closest_dir,
        pattern::InputPattern,
        preview::PreviewState,
        skin::*,
        stage::Stage,
        syntactic::SyntaxTheme,
        task_sync::{
            Dam,
            Either,
        },
        terminal,
        verb::Internal,
    },
    crokey::crossterm::event::Event,
    crossbeam::channel::{
        unbounded,
        Receiver,
        Sender,
    },
    std::{
        io::Write,
        path::PathBuf,
        str::FromStr,
        sync::{
            Arc,
            Mutex,
        },
    },
    strict::NonEmptyVec,
    termimad::EventSource,
};

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
    preview_panel: Option<PanelId>,

    stage_panel: Option<PanelId>,

    /// an optional copy of the root for the --server
    shared_root: Option<Arc<Mutex<PathBuf>>>,

    /// sender to the sequence channel
    tx_seqs: Sender<Sequence>,

    /// receiver to listen to the sequence channel
    rx_seqs: Receiver<Sequence>,

    /// counter incremented at every draw
    drawing_count: usize,
}

impl App {
    pub fn new(con: &AppContext) -> Result<App, ProgramError> {
        let screen = Screen::new(con)?;
        let mut browser_state = Box::new(BrowserState::new(
            con.initial_root.clone(),
            con.initial_tree_options.clone(),
            screen,
            con,
            &Dam::unlimited(),
        )?);
        if let Some(path) = con.initial_file.as_ref() {
            browser_state.tree.try_select_path(path);
        }
        let panel = Panel::new(
            PanelId::from(0),
            browser_state,
            Areas::create(&mut Vec::new(), 0, screen, false),
            con,
        );
        let (tx_seqs, rx_seqs) = unbounded::<Sequence>();
        let mut app = App {
            screen,
            active_panel_idx: 0,
            panels: panel.into(),
            quitting: false,
            launch_at_end: None,
            created_panels_count: 1,
            preview_panel: None,
            stage_panel: None,
            shared_root: None,
            tx_seqs,
            rx_seqs,
            drawing_count: 0,
        };
        if let Some(path) = con.initial_file.as_ref() {
            // open initial_file in preview
            let preview_state = Box::new(PreviewState::new(
                path.to_path_buf(),
                InputPattern::none(),
                None,
                con.initial_tree_options.clone(),
                con,
            ));
            if let Err(err) = app.new_panel(
                preview_state,
                PanelPurpose::Preview,
                HDir::Right,
                false,
                con,
            ) {
                warn!("could not open preview: {err}");
            } else {
                // we focus the preview panel
                app.active_panel_idx = 1;
            }
        }
        Ok(app)
    }

    fn panel_ref_to_idx(
        &self,
        panel_ref: PanelReference,
    ) -> Option<usize> {
        match panel_ref {
            PanelReference::Active => Some(self.active_panel_idx),
            PanelReference::Leftest => Some(0),
            PanelReference::Rightest => Some(self.panels.len().get() - 1),
            PanelReference::Id(id) => self.panel_id_to_idx(id),
            PanelReference::Preview => self.preview_panel.and_then(|id| self.panel_id_to_idx(id)),
        }
    }

    /// return the current index of the panel with given id
    fn panel_id_to_idx(
        &self,
        id: PanelId,
    ) -> Option<usize> {
        self.panels.iter().position(|panel| panel.id == id)
    }

    fn state(&self) -> &dyn PanelState {
        self.panels[self.active_panel_idx].state()
    }
    fn mut_state(&mut self) -> &mut dyn PanelState {
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
    fn close_panel(
        &mut self,
        panel_idx: usize,
    ) -> bool {
        let active_panel_id = self.panels[self.active_panel_idx].id;
        if let Some(preview_id) = self.preview_panel {
            if self.panels.has_len(2) && self.panels[panel_idx].id != preview_id {
                // we don't want to stay with just the preview
                return false;
            }
        }
        if let Some(stage_id) = self.stage_panel {
            if self.panels.has_len(2) && self.panels[panel_idx].id != stage_id {
                // we don't want to stay with just the stage
                return false;
            }
        }
        if let Ok(removed_panel) = self.panels.remove(panel_idx) {
            if self.preview_panel == Some(removed_panel.id) {
                self.preview_panel = None;
            }
            if self.stage_panel == Some(removed_panel.id) {
                self.stage_panel = None;
            }
            Areas::resize_all(
                self.panels.as_mut_slice(),
                self.screen,
                self.preview_panel.is_some(),
            );
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
        app_state: &AppState,
        con: &AppContext,
    ) -> Result<(), ProgramError> {
        self.drawing_count += 1;
        for (idx, panel) in self.panels.as_mut_slice().iter_mut().enumerate() {
            let active = idx == self.active_panel_idx;
            let panel_skin = if active {
                &skin.focused
            } else {
                &skin.unfocused
            };
            let disc = DisplayContext {
                count: self.drawing_count,
                active,
                screen: self.screen,
                panel_skin,
                state_area: panel.areas.state.clone(),
                app_state,
                con,
            };
            time!("display panel", panel.display(w, &disc)?,);
        }
        kitty::manager()
            .lock()
            .unwrap()
            .erase_images_before(w, self.drawing_count)?;
        w.flush()?;
        Ok(())
    }

    /// if there are exactly two non preview panels, return the selection
    /// in the non focused panel
    fn get_other_panel_path(&self) -> Option<PathBuf> {
        let len = self.panels.len().get();
        if len == 3 {
            if let Some(preview_id) = self.preview_panel {
                for (idx, panel) in self.panels.iter().enumerate() {
                    if self.active_panel_idx != idx && panel.id != preview_id {
                        return panel.state().selected_path().map(|p| p.to_path_buf());
                    }
                }
            }
            None
        } else if self.panels.len().get() == 2 && self.preview_panel.is_none() {
            let non_focused_panel_idx = if self.active_panel_idx == 0 { 1 } else { 0 };
            self.panels[non_focused_panel_idx]
                .state()
                .selected_path()
                .map(|p| p.to_path_buf())
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
        app_state: &mut AppState,
        con: &mut AppContext,
    ) -> Result<(), ProgramError> {
        use CmdResult::*;
        let mut error: Option<String> = None;
        let is_input_invocation = cmd.is_verb_invocated_from_input();
        let app_cmd_context = AppCmdContext {
            panel_skin,
            preview_panel: self.preview_panel,
            stage_panel: self.stage_panel,
            screen: self.screen, // it can't change in this function
            con,
        };
        let cmd_result = self
            .mut_panel()
            .apply_command(w, &cmd, app_state, &app_cmd_context)?;
        debug!("cmd_result: {:?}", &cmd_result);
        match cmd_result {
            ApplyOnPanel { id } => {
                if let Some(idx) = self.panel_id_to_idx(id) {
                    if let DisplayError(txt) =
                        self.panels[idx].apply_command(w, &cmd, app_state, &app_cmd_context)?
                    {
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
            ClosePanel {
                validate_purpose,
                panel_ref,
            } => {
                if is_input_invocation {
                    self.mut_panel().clear_input_invocation(con);
                }
                let close_idx = self.panel_ref_to_idx(panel_ref)
                    .unwrap_or_else(||
                        // when there's a preview panel, we close it rather than the app
                        if self.panels.len().get()==2 && self.preview_panel.is_some() {
                            1
                        } else {
                            self.active_panel_idx
                        }
                    );
                let mut new_arg = None;
                if validate_purpose {
                    let purpose = &self.panels[close_idx].purpose;
                    if let PanelPurpose::ArgEdition { .. } = purpose {
                        new_arg = self.panels[close_idx]
                            .state()
                            .selected_path()
                            .map(|p| p.to_string_lossy().to_string());
                    }
                }
                if self.close_panel(close_idx) {
                    let screen = self.screen;
                    self.mut_state().refresh(screen, con);
                    if let Some(new_arg) = new_arg {
                        self.mut_panel().set_input_arg(new_arg);
                        let new_input = self.panel().get_input_content();
                        let cmd = Command::from_raw(new_input, false);
                        let app_cmd_context = AppCmdContext {
                            panel_skin,
                            preview_panel: self.preview_panel,
                            stage_panel: self.stage_panel,
                            screen,
                            con,
                        };
                        self.mut_panel()
                            .apply_command(w, &cmd, app_state, &app_cmd_context)?;
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
                debug!("handling internal {internal:?} at app level");
                match internal {
                    Internal::escape => {
                        let mode = self.panel().state().get_mode();
                        let cmd = self.mut_panel().input.escape(con, mode);
                        debug!("cmd on escape: {cmd:?}");
                        self.apply_command(w, cmd, panel_skin, app_state, con)?;
                    }
                    Internal::panel_left_no_open | Internal::panel_right_no_open => {
                        let new_active_panel_idx = if internal == Internal::panel_left_no_open {
                            // we're here because the state wants us to either move to the panel
                            // to the left, or close the rightest one
                            if self.active_panel_idx == 0 {
                                self.close_panel(self.panels.len().get() - 1);
                                None
                            } else {
                                Some(self.active_panel_idx - 1)
                            }
                        } else {
                            // panel_right
                            // we either move to the right or close the leftest panel
                            if self.active_panel_idx + 1 == self.panels.len().get() {
                                self.close_panel(0);
                                None
                            } else {
                                Some(self.active_panel_idx + 1)
                            }
                        };
                        if let Some(idx) = new_active_panel_idx {
                            if is_input_invocation {
                                self.mut_panel().clear_input();
                            }
                            self.active_panel_idx = idx;
                            let app_cmd_context = AppCmdContext {
                                panel_skin,
                                preview_panel: self.preview_panel,
                                stage_panel: self.stage_panel,
                                screen: self.screen,
                                con,
                            };
                            self.mut_panel()
                                .refresh_input_status(app_state, &app_cmd_context);
                        }
                    }
                    Internal::toggle_second_tree => {
                        let panels_count = self.panels.len().get();
                        let trees_count = self
                            .panels
                            .iter()
                            .filter(|p| p.state().get_type() == PanelStateType::Tree)
                            .count();
                        if trees_count < 2 {
                            // we open a tree, closing a (non tree) panel if necessary
                            if panels_count >= con.max_panels_count {
                                for i in (0..panels_count).rev() {
                                    if self.panels[i].state().get_type() != PanelStateType::Tree {
                                        self.close_panel(i);
                                        break;
                                    }
                                }
                            }
                            if let Some(selected_path) = self.state().selected_path() {
                                let dir = closest_dir(selected_path);
                                if let Ok(new_state) = BrowserState::new(
                                    dir,
                                    self.state().tree_options().without_pattern(),
                                    self.screen,
                                    con,
                                    &Dam::unlimited(),
                                ) {
                                    if let Err(s) = self.new_panel(
                                        Box::new(new_state),
                                        PanelPurpose::None,
                                        HDir::Right,
                                        is_input_invocation,
                                        con,
                                    ) {
                                        error = Some(s);
                                    }
                                }
                            }
                        } else {
                            // we close the rightest inactive tree
                            for i in (0..panels_count).rev() {
                                if self.panels[i].state().get_type() == PanelStateType::Tree {
                                    if i == self.active_panel_idx {
                                        continue;
                                    }
                                    self.close_panel(i);
                                    break;
                                }
                            }
                        }
                    }
                    Internal::set_syntax_theme => {
                        let arg = cmd.as_verb_invocation().and_then(|vi| vi.args.as_ref());
                        match arg {
                            Some(arg) => match SyntaxTheme::from_str(arg) {
                                Ok(theme) => {
                                    con.syntax_theme = Some(theme);
                                    self.update_preview(con, true);
                                }
                                Err(e) => {
                                    error = Some(e.to_string());
                                }
                            },
                            None => {
                                error = Some("no theme provided".to_string());
                            }
                        }
                    }
                    _ => {
                        info!("unhandled propagated internal. cmd={:?}", &cmd);
                    }
                }
            }
            Keep => {
                if is_input_invocation {
                    self.mut_panel().clear_input_invocation(con);
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
                if let Err(s) = self.new_panel(state, purpose, direction, is_input_invocation, con) {
                    error = Some(s);
                }
            }
            NewState { state, message } => {
                self.mut_panel().clear_input();
                self.mut_panel().push_state(state);
                if let Some(md) = message {
                    self.mut_panel().set_message(md);
                } else {
                    self.mut_panel()
                        .refresh_input_status(app_state, &app_cmd_context);
                }
            }
            PopState => {
                if is_input_invocation {
                    self.mut_panel().clear_input();
                }
                if self.remove_state() {
                    self.mut_state().refresh(app_cmd_context.screen, con);
                    self.mut_panel()
                        .refresh_input_status(app_state, &app_cmd_context);
                } else if con.quit_on_last_cancel {
                    self.quitting = true;
                }
            }
            PopStateAndReapply => {
                if is_input_invocation {
                    self.mut_panel().clear_input();
                }
                if self.remove_state() {
                    let app_cmd_context = AppCmdContext {
                        panel_skin,
                        preview_panel: self.preview_panel,
                        stage_panel: self.stage_panel,
                        screen: self.screen,
                        con,
                    };
                    self.mut_panel()
                        .apply_command(w, &cmd, app_state, &app_cmd_context)?;
                } else if con.quit_on_last_cancel {
                    self.quitting = true;
                }
            }
            Quit => {
                self.quitting = true;
            }
            RefreshState { clear_cache } => {
                if is_input_invocation {
                    self.mut_panel().clear_input_invocation(con);
                }
                if clear_cache {
                    clear_caches();
                }
                app_state.stage.refresh();
                for i in 0..self.panels.len().get() {
                    self.panels[i].mut_state().refresh(self.screen, con);
                }
            }
        }
        if let Some(text) = error {
            self.mut_panel().set_error(text);
        }

        app_state.other_panel_path = self.get_other_panel_path();
        if let Some(path) = self.state().tree_root() {
            app_state.root = path.to_path_buf();
            terminal::update_title(w, app_state, con);
            if con.update_work_dir {
                if let Err(e) = std::env::set_current_dir(&app_state.root) {
                    warn!("Failed to set current dir: {e}");
                }
            }
            if let Some(shared_root) = &mut self.shared_root {
                if let Ok(mut root) = shared_root.lock() {
                    *root = app_state.root.clone();
                }
            }
        }

        self.update_preview(con, false);

        Ok(())
    }

    /// update the state of the preview, if there's some
    fn update_preview(
        &mut self,
        con: &AppContext,
        refresh: bool,
    ) {
        let preview_idx = self.preview_panel.and_then(|id| self.panel_id_to_idx(id));
        if let Some(preview_idx) = preview_idx {
            if let Some(path) = self.state().selected_path() {
                let old_path = self.panels[preview_idx].state().selected_path();
                if refresh || Some(path) != old_path {
                    let path = path.to_path_buf();
                    self.panels[preview_idx]
                        .mut_state()
                        .set_selected_path(path, con);
                }
            }
        }
    }

    /// get the index of the panel at x
    fn clicked_panel_index(
        &self,
        x: u16,
        _y: u16,
    ) -> usize {
        let len = self.panels.len().get();
        (len * x as usize) / (self.screen.width as usize + 1)
    }

    /// handle CmdResult::NewPanel
    fn new_panel(
        &mut self,
        state: Box<dyn PanelState>,
        purpose: PanelPurpose,
        direction: HDir,
        is_input_invocation: bool, // if true we clean the input
        con: &AppContext,
    ) -> Result<(), String> {
        match state.get_type() {
            PanelStateType::Preview if self.preview_panel.is_some() => {
                return Err("There can be only one preview panel".to_owned());
                // todo replace instead ?
            }
            PanelStateType::Stage if self.stage_panel.is_some() => {
                return Err("There can be only one stage panel".to_owned());
                // todo replace instead ?
            }
            _ => {}
        }
        if is_input_invocation {
            self.mut_panel().clear_input_invocation(con);
        }
        let insertion_idx = if purpose.is_preview() {
            self.panels.len().get()
        } else if direction == HDir::Right {
            self.active_panel_idx + 1
        } else {
            self.active_panel_idx
        };
        let with_preview = purpose.is_preview() || self.preview_panel.is_some();
        let areas = Areas::create(
            self.panels.as_mut_slice(),
            insertion_idx,
            self.screen,
            with_preview,
        );
        let panel_id = self.created_panels_count.into();
        match state.get_type() {
            PanelStateType::Preview => {
                self.preview_panel = Some(panel_id);
            }
            PanelStateType::Stage => {
                self.stage_panel = Some(panel_id);
            }
            _ => {
                self.active_panel_idx = insertion_idx;
            }
        }
        let mut panel = Panel::new(panel_id, state, areas, con);
        panel.purpose = purpose;
        self.created_panels_count += 1;
        self.panels.insert(insertion_idx, panel);
        Ok(())
    }

    /// do the pending tasks, if any, and refresh the screen accordingly
    fn do_pending_tasks(
        &mut self,
        w: &mut W,
        skin: &AppSkin,
        dam: &mut Dam,
        app_state: &mut AppState,
        con: &AppContext,
    ) -> Result<(), ProgramError> {
        while self.has_pending_task() && !dam.has_event() {
            let error = self.do_pending_task(app_state, con, dam).err();
            self.update_preview(con, false); // the selection may have changed
            if let Some(error) = &error {
                self.mut_panel().set_error(error.to_string());
            } else {
                let panel_skin = &skin.focused;
                let app_cmd_context = AppCmdContext {
                    panel_skin,
                    preview_panel: self.preview_panel,
                    stage_panel: self.stage_panel,
                    screen: self.screen, // it can't change in this function
                    con,
                };
                self.mut_panel()
                    .refresh_input_status(app_state, &app_cmd_context);
            }
            self.display_panels(w, skin, app_state, con)?;
            if error.is_some() {
                return Ok(()); // breaking pending tasks chain on first error/interruption
            }
        }
        Ok(())
    }

    /// Do the next pending task
    fn do_pending_task(
        &mut self,
        app_state: &mut AppState,
        con: &AppContext,
        dam: &mut Dam,
    ) -> Result<(), ProgramError> {
        let screen = self.screen;
        // we start with the focused panel
        if self.panel().has_pending_task() {
            return self
                .mut_panel()
                .do_pending_task(app_state, screen, con, dam);
        }
        // then the other ones
        for idx in 0..self.panels.len().get() {
            if idx != self.active_panel_idx {
                let panel = &mut self.panels[idx];
                if panel.has_pending_task() {
                    return panel.do_pending_task(app_state, screen, con, dam);
                }
            }
        }
        warn!("unexpected lack of pending task");
        Ok(())
    }

    fn has_pending_task(&mut self) -> bool {
        self.panels.iter().any(|p| p.has_pending_task())
    }

    /// This is the main loop of the application
    pub fn run(
        mut self,
        w: &mut W,
        con: &mut AppContext,
        conf: &Conf,
    ) -> Result<Option<Launchable>, ProgramError> {
        #[cfg(feature = "clipboard")]
        {
            // different systems have different clipboard capabilities
            // and it may be useful to know which one we have
            debug!("Clipboard backend: {:?}", terminal_clipboard::get_type());
        }
        // we listen for events in a separate thread so that we can go on listening
        // when a long search is running, and interrupt it if needed
        let event_source = EventSource::new()?;
        let rx_events = event_source.receiver();
        let mut dam = Dam::from(rx_events);
        let skin = AppSkin::new(conf, con.launch_args.color == TriBool::No);
        let mut app_state = AppState {
            stage: Stage::default(),
            root: con.initial_root.clone(),
            other_panel_path: None,
        };
        terminal::update_title(w, &app_state, con);

        self.screen.clear_bottom_right_char(w, &skin.focused)?;

        #[cfg(windows)]
        if con.cmd().is_some() {
            // Powershell sends to broot a resize event after it was launched
            // which interrupts its task queue. An easy fix is to wait for a
            // few ms for the terminal to be stabilized.
            // It's possible some other terminals, even not on Windows, might
            // need the same trick in the future
            let delay = std::time::Duration::from_millis(10);
            std::thread::sleep(delay);
            let dropped_events = dam.clear();
            debug!("Dropped {dropped_events} events");
            event_source.unblock(self.quitting);
        }

        if let Some(raw_sequence) = &con.cmd() {
            self.tx_seqs
                .send(Sequence::new_local(raw_sequence.to_string()))
                .unwrap();
        }

        #[cfg(unix)]
        let _server = con
            .launch_args
            .listen
            .as_ref()
            .map(|server_name| {
                let shared_root = Arc::new(Mutex::new(app_state.root.clone()));
                let server = crate::net::Server::new(
                    server_name,
                    self.tx_seqs.clone(),
                    Arc::clone(&shared_root),
                );
                self.shared_root = Some(shared_root);
                server
            })
            .transpose()?;

        loop {
            if !self.quitting {
                self.display_panels(w, &skin, &app_state, con)?;
                time!(
                    Info,
                    "pending_tasks",
                    self.do_pending_tasks(w, &skin, &mut dam, &mut app_state, con)?,
                );
            }
            #[allow(unused_mut)]
            match dam.next(&self.rx_seqs) {
                Either::First(Some(event)) => {
                    info!("event: {:?}", &event);
                    let mut handled = false;

                    // app level handling
                    if let Some((x, y)) = event.as_click() {
                        if self.clicked_panel_index(x, y) != self.active_panel_idx {
                            // panel activation click
                            self.active_panel_idx = self.clicked_panel_index(x, y);
                            handled = true;
                        }
                    } else if let Event::Resize(mut width, mut height) = event.event {
                        // I don't know why but Crossterm seems to always report an
                        // understimated size on Windows
                        #[cfg(windows)]
                        {
                            width += 1;
                            height += 1;
                        }
                        self.screen.set_terminal_size(width, height, con);
                        Areas::resize_all(
                            self.panels.as_mut_slice(),
                            self.screen,
                            self.preview_panel.is_some(),
                        );
                        for panel in &mut self.panels {
                            panel.mut_state().refresh(self.screen, con);
                        }
                        handled = true;
                    }

                    // event handled by the panel
                    if !handled {
                        let cmd = self.mut_panel().add_event(w, event, &app_state, con)?;
                        debug!("command after add_event: {:?}", &cmd);
                        self.apply_command(w, cmd, &skin.focused, &mut app_state, con)?;
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
                        self.apply_command(w, arg_cmd, &skin.focused, &mut app_state, con)?;
                        if self.quitting {
                            // is that a 100% safe way of quitting ?
                            return Ok(self.launch_at_end.take());
                        } else {
                            self.display_panels(w, &skin, &app_state, con)?;
                            time!(
                                "sequence pending tasks",
                                self.do_pending_tasks(w, &skin, &mut dam, &mut app_state, con)?,
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
