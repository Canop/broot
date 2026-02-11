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
        display::*,
        errors::ProgramError,
        file_sum,
        git,
        kitty,
        launchable::Launchable,
        path::closest_dir,
        pattern::InputPattern,
        preview::PreviewState,
        skin::*,
        syntactic::SyntaxTheme,
        task_sync::{
            Dam,
            Either,
        },
        terminal,
        verb::Internal,
        watcher::Watcher,
    },
    crokey::crossterm::event::Event,
    std::{
        io::Write,
        path::PathBuf,
        str::FromStr,
        sync::{
            Arc,
            Mutex,
        },
    },
    termimad::{
        EventSource,
        EventSourceOptions,
        crossbeam::channel::{
            Receiver,
            Sender,
            unbounded,
        },
    },
};

/// The GUI
pub struct App {
    /// the panels of the application, with their inputs
    panels: AppPanelsAndInputs,

    /// whether the app is in the (uncancellable) process of quitting
    quitting: bool,

    /// what must be done after having closed the TUI
    launch_at_end: Option<Launchable>,

    /// an optional copy of the root for the --server
    shared_root: Option<Arc<Mutex<PathBuf>>>,

    /// sender to the sequence channel
    tx_seqs: Sender<Sequence>,

    /// receiver to listen to the sequence channel
    rx_seqs: Receiver<Sequence>,

    /// a watcher for notify events
    watcher: Watcher,
}

impl App {
    pub fn new(con: &AppContext) -> Result<App, ProgramError> {
        let mut panels = AppPanelsAndInputs::new(con)?;
        if let Some(path) = con.initial_file.as_ref() {
            // open initial_file in preview
            let preview_state = Box::new(PreviewState::new(
                path.clone(),
                InputPattern::none(),
                None,
                con.initial_tree_options.clone(),
                con,
            ));
            if let Err(err) = panels.new_panel(
                preview_state,
                PanelPurpose::Preview,
                HDir::Right,
                true, // activate
                con,
            ) {
                warn!("could not open preview: {err}");
            }
        }
        let (tx_seqs, rx_seqs) = unbounded::<Sequence>();
        let watcher = Watcher::new(tx_seqs.clone());
        Ok(Self {
            panels,
            quitting: false,
            launch_at_end: None,
            shared_root: None,
            tx_seqs,
            rx_seqs,
            watcher,
        })
    }

    /// apply a command. Change the states but don't redraw on screen.
    fn apply_command(
        &mut self,
        w: &mut W,
        cmd: &Command,
        panel_skin: &PanelSkin,
        app_state: &mut AppState,
        con: &mut AppContext,
    ) -> Result<(), ProgramError> {
        info!("app applying command: {:?}", &cmd);
        let is_input_invocation = cmd.is_verb_invocated_from_input();
        let cmd_result = self
            .panels
            .apply_command(w, cmd, None, panel_skin, app_state, con)?;
        debug!("cmd_result: {:?}", &cmd_result);
        let mut error: Option<String> = None;
        let mut new_active_panel_idx = None;
        match cmd_result {
            CmdResult::ApplyOnPanel { id } => {
                let aop_cmd_result = self.panels.apply_command(
                    w,
                    cmd,
                    Some(PanelReference::Id(id)),
                    panel_skin,
                    app_state,
                    con,
                )?;
                if let CmdResult::DisplayError(txt) = aop_cmd_result {
                    // we should probably handle other results
                    // which implies the possibility of a recursion
                    error = Some(txt);
                } else if is_input_invocation {
                    self.panels.clear_input();
                }
            }
            CmdResult::ClosePanel {
                validate_purpose,
                panel_ref,
                clear_cache,
            } => {
                if is_input_invocation {
                    self.panels.clear_input_invocation(con);
                }
                let close_idx = self.panels.idx_by_ref(panel_ref)
                    .unwrap_or_else(||
                        // when there's a preview panel, we close it rather than the app
                        if self.panels.len()==2 && self.panels.has_preview_panel() {
                            1
                        } else {
                            self.panels.active_panel_idx()
                        }
                    );
                let mut new_arg = None;
                if validate_purpose {
                    let purpose = &self.panels.panel_by_idx_unchecked(close_idx).purpose;
                    if let PanelPurpose::ArgEdition { .. } = purpose {
                        new_arg = self
                            .panels
                            .panel_by_idx_unchecked(close_idx)
                            .state()
                            .selected_path()
                            .map(|p| p.to_string_lossy().to_string());
                    }
                }
                if clear_cache {
                    clear_caches();
                }
                if self.panels.close(close_idx, con) {
                    let screen = self.panels.screen();
                    self.panels.refresh_active_panel(con);
                    if let Some(new_arg) = new_arg {
                        self.panels.set_input_arg(new_arg);
                        let new_input = self.panels.get_input_content();
                        let cmd = Command::from_raw(new_input, false);
                        let app_cmd_context = AppCmdContext {
                            panel_skin,
                            preview_panel: self.panels.preview_panel_id(),
                            stage_panel: self.panels.stage_panel_id(),
                            screen,
                            con,
                        };
                        self.panels.mut_panel().apply_command(
                            w,
                            &cmd,
                            app_state,
                            &app_cmd_context,
                        )?;
                    }
                } else {
                    self.quitting = true;
                }
            }
            CmdResult::ChangeLayout(instruction) => {
                con.layout_instructions.push(instruction);
                self.panels.resize_all(con);
            }
            CmdResult::DisplayError(txt) => {
                error = Some(txt);
            }
            CmdResult::ExecuteSequence { sequence } => {
                if is_input_invocation {
                    self.panels.clear_input();
                }
                self.tx_seqs.send(sequence).unwrap();
            }
            CmdResult::HandleInApp(internal) => {
                debug!("handling internal {internal:?} at app level");
                match internal {
                    Internal::escape => {
                        let mode = self.panels.state().get_mode();
                        let cmd = self.panels.do_input_escape(mode, con);
                        debug!("cmd on escape: {cmd:?}");
                        self.apply_command(w, &cmd, panel_skin, app_state, con)?;
                    }
                    Internal::focus_staging_area_no_open => {
                        self.panels.focus_by_type(PanelStateType::Stage);
                    }
                    Internal::focus_panel_left => {
                        let len = self.panels.len().get();
                        new_active_panel_idx = Some((self.active_panel_idx + len - 1) % len);
                    }
                    Internal::focus_panel_right => {
                        let len = self.panels.len().get();
                        new_active_panel_idx = Some((self.active_panel_idx + 1) % len);
                    }
                    Internal::panel_left_no_open => {
                        // we're here because the state wants us to either move to the panel
                        // to the left, or close the rightest one
                        new_active_panel_idx = if self.panels.active_panel_idx() == 0 {
                            self.panels.close(self.panels.len() - 1, con);
                            None
                        } else {
                            Some(self.panels.active_panel_idx() - 1)
                        };
                    }
                    Internal::panel_right_no_open => {
                        // we either move to the right or close the leftest panel
                        new_active_panel_idx =
                            if self.panels.active_panel_idx() + 1 == self.panels.len() {
                                self.panels.close(0, con);
                                None
                            } else {
                                Some(self.panels.active_panel_idx() + 1)
                            };
                    }
                    Internal::search_again => {
                        if let Some(raw_pattern) = &self.panels.panel().last_raw_pattern {
                            let sequence = Sequence::new_single(raw_pattern.clone());
                            self.tx_seqs.send(sequence).unwrap();
                        }
                    }
                    Internal::set_syntax_theme => {
                        let arg = cmd.as_verb_invocation().and_then(|vi| vi.args.as_ref());
                        match arg {
                            Some(arg) => match SyntaxTheme::from_str(arg) {
                                Ok(theme) => {
                                    con.syntax_theme = Some(theme);
                                    self.panels.update_preview(true, con);
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
                    Internal::toggle_second_tree => {
                        let panels_count = self.panels.len();
                        let trees_count = self.panels.count_of_type(PanelStateType::Tree);
                        if trees_count < 2 {
                            // we open a tree, closing a (non tree) panel if necessary
                            if panels_count >= con.max_panels_count {
                                self.panels.close_first_non_tree(con);
                            }
                            if let Some(selected_path) = self.panels.state().selected_path() {
                                let dir = closest_dir(selected_path);
                                let screen = self.panels.screen();
                                if let Ok(new_state) = BrowserState::new(
                                    dir,
                                    self.panels.state().tree_options().without_pattern(),
                                    screen,
                                    con,
                                    &Dam::unlimited(),
                                ) {
                                    if let Err(s) = self.panels.new_panel(
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
                            self.panels.close_rightest_inactive_tree(con);
                        }
                    }
                    Internal::toggle_watch => {
                        app_state.watch_tree ^= true;
                        if is_input_invocation {
                            self.panels.clear_input_invocation(con);
                        }
                    }
                    _ => {
                        let cmd = self.panels.on_input_internal(internal);
                        if cmd.is_none() {
                            warn!(
                                "unhandled propagated internal. internal={internal:?} cmd={cmd:?}"
                            );
                        } else {
                            self.apply_command(w, &cmd, panel_skin, app_state, con)?;
                        }
                    }
                }
            }
            CmdResult::Keep => {
                if is_input_invocation {
                    self.panels.clear_input_invocation(con);
                }
            }
            CmdResult::Message(md) => {
                if is_input_invocation {
                    self.panels.clear_input_invocation(con);
                }
                self.panels.mut_panel().set_message(md);
            }
            CmdResult::Launch(launchable) => {
                self.launch_at_end = Some(*launchable);
                self.quitting = true;
            }
            CmdResult::NewPanel {
                state,
                purpose,
                direction,
            } => {
                if let Err(s) =
                    self.panels
                        .new_panel(state, purpose, direction, is_input_invocation, con)
                {
                    error = Some(s);
                }
            }
            CmdResult::NewState { state, message } => {
                self.panels.clear_input();
                self.panels.push_state(state);
                if let Some(md) = message {
                    self.panels.mut_panel().set_message(md);
                } else {
                    self.panels.refresh_input_status(app_state, panel_skin, con);
                }
            }
            CmdResult::PopState => {
                if is_input_invocation {
                    self.panels.clear_input();
                }
                if self.panels.remove_state(con) {
                    let screen = self.panels.screen();
                    self.panels.mut_state().refresh(screen, con);
                    self.panels.refresh_input_status(app_state, panel_skin, con);
                } else if con.quit_on_last_cancel {
                    self.quitting = true;
                }
            }
            CmdResult::PopStateAndReapply => {
                if is_input_invocation {
                    self.panels.clear_input();
                }
                if self.panels.remove_state(con) {
                    self.panels.apply_command(
                        w, cmd, None, // active panel
                        panel_skin, app_state, con,
                    )?;
                } else if con.quit_on_last_cancel {
                    self.quitting = true;
                }
            }
            CmdResult::Quit => {
                self.quitting = true;
            }
            CmdResult::RefreshState { clear_cache } => {
                info!("refreshing, clearing cache={clear_cache}");
                if is_input_invocation {
                    self.panels.clear_input_invocation(con);
                }
                if clear_cache {
                    clear_caches();
                }
                app_state.stage.refresh();
                self.panels.refresh_all_panels(con);
            }
        }
        if let Some(text) = error {
            self.panels.mut_panel().set_error(text);
        }

        if let Some(idx) = new_active_panel_idx {
            debug!("activating panel idx {idx}");
            if is_input_invocation {
                self.panels.clear_input();
            }
            self.panels.activate(idx);
            self.panels.refresh_input_status(app_state, panel_skin, con);
        }

        app_state.other_panel_path = self.panels.get_other_panel_path();
        if let Some(path) = self.panels.state().tree_root() {
            app_state.root = path.to_path_buf();
            terminal::update_title(w, app_state, con);
            if con.update_work_dir {
                if let Err(e) = std::env::set_current_dir(&app_state.root) {
                    warn!("Failed to set current dir: {e}");
                }
            }
            if let Some(shared_root) = &mut self.shared_root {
                if let Ok(mut root) = shared_root.lock() {
                    root.clone_from(&app_state.root);
                }
            }
        }

        self.panels.update_preview(false, con);

        Ok(())
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
        w.flush()?;
        let combine_keys = conf.enable_kitty_keyboard.unwrap_or(false) && con.is_tty;
        let event_source = EventSource::with_options(EventSourceOptions {
            combine_keys,
            ..Default::default()
        })?;
        con.keyboard_enhanced = event_source.supports_multi_key_combinations();
        info!(
            "event source is combining: {}",
            event_source.supports_multi_key_combinations()
        );

        let rx_events = event_source.receiver();
        let mut dam = Dam::from(rx_events);
        let skin = AppSkin::new(conf, con.launch_args.color == TriBool::No);
        let mut app_state = AppState::new(&con.initial_root);
        terminal::update_title(w, &app_state, con);

        self.panels
            .screen()
            .clear_bottom_right_char(w, &skin.focused)?;

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
                .send(Sequence::new_local((*raw_sequence).to_string()))
                .map_err(|e| ProgramError::Internal {
                    details: format!("failed to send initial command: {e}"),
                })?;
        }

        #[cfg(unix)]
        let _server = con
            .server_name
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
                self.panels.display_panels(w, &skin, &app_state, con)?;
                time!(
                    Debug,
                    "pending_tasks",
                    self.panels
                        .do_pending_tasks(w, &skin, &mut dam, &mut app_state, con)?,
                );
            }

            // before starting to wait for events, we enable the watcher if needed
            if app_state.watch_tree {
                let paths = self.panels.state().watchable_paths();
                if let Err(e) = self.watcher.watch(paths) {
                    // errors aren't uncommon, especially on huge directories
                    warn!("Failed to watch tree: {e}");
                    // we disable watching
                    app_state.watch_tree = false;
                }
            }
            let event = dam.next(&self.rx_seqs);
            if app_state.watch_tree {
                // we must unwatch before applying the command, as it will probably do many system
                // calls that would trigger events
                self.watcher.stop_watching()?;
            }

            #[allow(unused_mut)]
            match event {
                Either::First(Some(event)) => {
                    info!("<-- event: {:?}", &event);
                    if let Some(key_combination) = event.key_combination {
                        info!("key combination: {key_combination}");
                    }
                    let mut handled = false;

                    // app level handling
                    if let Some((x, y)) = event.as_click() {
                        let clicked_idx = self.panels.clicked_panel_index(x, y);
                        if clicked_idx != self.panels.active_panel_idx() {
                            // panel activation click
                            self.panels.activate(clicked_idx);
                            handled = true;
                        }
                    } else if let Event::Resize(mut width, mut height) = event.event {
                        self.panels.set_terminal_size(width, height, con);
                        handled = true;
                    }

                    // event handled by the panel
                    if !handled {
                        let cmd = self.panels.on_input_event(w, &event, &app_state, con)?;
                        info!("command from panels.on_input_event: {:#?}", &cmd);
                        self.apply_command(w, &cmd, &skin.focused, &mut app_state, con)?;
                    }

                    event_source.unblock(self.quitting);
                }
                Either::First(None) => {
                    // this is how we quit the application,
                    // when the input thread is properly closed
                    break;
                }
                Either::Second(Some(sequence)) => {
                    info!("got command sequence: {:?}", &sequence);
                    for (input, arg_cmd) in sequence.parse(con)? {
                        if !matches!(&arg_cmd, Command::Internal { .. }) {
                            self.panels.input().set_content(&input);
                        }
                        self.apply_command(w, &arg_cmd, &skin.focused, &mut app_state, con)?;
                        if self.quitting {
                            return Ok(self.launch_at_end.take());
                        }
                        self.panels.display_panels(w, &skin, &app_state, con)?;
                        time!(
                            "sequence pending tasks",
                            self.panels.do_pending_tasks(
                                w,
                                &skin,
                                &mut dam,
                                &mut app_state,
                                con
                            )?,
                        );
                    }
                }
                Either::Second(None) => {
                    warn!("I didn't expect a None to occur here");
                }
            }
        }
        terminal::reset_title(w, con);
        if let Ok(mut manager) = kitty::manager().lock() {
            manager.erase_images_before(w, usize::MAX)?;
        }
        w.flush()?;

        Ok(self.launch_at_end.take())
    }
}

/// clear the file sizes and git stats cache.
///
/// This should be done on Refresh actions and after any external command.
fn clear_caches() {
    file_sum::clear_cache();
    git::clear_status_computer_cache();
    #[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
    crate::filesystems::clear_cache();
}
