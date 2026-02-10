use {
    super::*,
    crate::{
        browser::*,
        command::*,
        display::*,
        errors::ProgramError,
        kitty,
        skin::*,
        task_sync::Dam,
        verb::*,
    },
    crokey::crossterm::{
        cursor::MoveTo,
        queue,
    },
    std::{
        io::Write,
        path::{
            Path,
            PathBuf,
        },
    },
    termimad::TimedEvent,
};

/// Stores panels of the application and their inputs.
///
/// Most fields are private to enforce consistency.
/// This thing is designed so that the inputs and panels can be
/// borrowed separately, which is useful for input handling and drawing.
pub struct AppPanelsAndInputs {

    /// a count of all panels created
    created_panels_count: usize,

    panels: AppPanels,

    /// one input per panel, in the same order. Never empty.
    inputs: Vec<PanelInput>,

    /// counter incremented at every draw
    drawing_count: usize,
}

/// Stores panels of the application.
///
/// This structure is designed to be borrowed by reference for state access and manipulation,
///  especially when applying mutations to the input when handling events.
///
/// Fields are private to enforce consistency.
pub struct AppPanels {
    /// dimensions of the screen
    pub screen: Screen,

    active_panel_idx: usize, // guaranteed to be < panels.len()
    //
    /// panels from left to right, never empty
    panels: Vec<Panel>,
}

impl AppPanelsAndInputs {

    /// Create the appPanelsAndInputs which should be kept for the whole life of
    /// the application, starting with a single panel (it can't be empty), based on
    /// the initial_root
    pub fn new(con: &AppContext) -> Result<Self, ProgramError> {
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
        let areas = Areas::create(&mut Vec::new(), &con.layout_instructions, 0, screen, false);
        let input = PanelInput::new(areas.input.clone());
        let panel = Panel::new(
            PanelId::from(0),
            browser_state,
            areas,
            con,
        );
        debug!("initial panel areas: {:?}", panel.areas);
        Ok(Self {
            created_panels_count: 0,
            panels: AppPanels {
                screen,
                active_panel_idx: 0,
                panels: vec![panel],
            },
            inputs: vec![input],
            drawing_count: 0,
        })
    }

    // ----------------------------------------------------
    // Accessors

    pub fn screen(&self) -> Screen {
        self.panels.screen
    }

    pub fn len(&self) -> usize {
        self.inputs.len()
    }


    // ----------------------------------------------------
    // resizing and layout

    pub fn set_terminal_size(
        &mut self,
        w: u16,
        h: u16,
        con: &AppContext,
    ) {
        self.screen().set_terminal_size(w, h, con);
        self.resize_all(con);
    }

    pub fn resize_all(
        &mut self,
        con: &AppContext,
    ) {
        let screen = self.screen();
        let has_preview = self.has_preview_panel();
        Areas::resize_all(
            self.panels.panels.as_mut_slice(),
            &con.layout_instructions,
            screen,
            has_preview,
        );
        for panel in &mut self.panels.panels {
            panel.mut_state().refresh(screen, con);
        }
    }


    // ----------------------------------------------------
    // state access

    pub fn state(&self) -> &dyn PanelState {
        self.panels.panels[self.active_panel_idx()].state()
    }
    pub fn mut_state(&mut self) -> &mut dyn PanelState {
        let idx = self.active_panel_idx();
        self.panels.panels[idx].mut_state()
    }

    /// if there are exactly two non preview panels, return the selection
    /// in the non focused panel
    pub fn get_other_panel_path(&self) -> Option<PathBuf> {
        let mut non_preview_count = 0;
        let mut other_panel_idx = None;
        for (idx, panel) in self.panels.panels.iter().enumerate() {
            if panel.state().get_type() != PanelStateType::Preview {
                non_preview_count += 1;
                if idx != self.active_panel_idx() {
                    other_panel_idx = Some(idx);
                }
            }
        }
        if non_preview_count == 2 {
            if let Some(other_panel_idx) = other_panel_idx {
                return self.panels.panels[other_panel_idx]
                    .state()
                    .selected_path()
                    .map(Path::to_path_buf);
            }
        }
        None
    }


    // ----------------------------------------------------
    // state manipulation

    pub fn push_state(
        &mut self,
        new_state: Box<dyn PanelState>,
    ) {
        let idx = self.active_panel_idx();
        self.inputs[idx].set_content(&new_state.get_starting_input());
        self.panels.panels[idx].push_state(new_state);
    }

    /// remove the top state of the current panel
    ///
    /// Close the panel too if that was its only state.
    /// Close nothing and return false if there's not
    /// at least two states in the app.
    pub fn remove_state(
        &mut self,
        con: &AppContext,
    ) -> bool {
        let idx = self.active_panel_idx();
        if self.panels.panels[idx].remove_state() {
            let input_content = self.state().get_starting_input();
            self.inputs[idx].set_content(&input_content);
            true
        } else {
            self.close(idx, con)
        }
    }

    // ----------------------------------------------------
    // panels access

    pub fn panels(&self) -> &AppPanels {
        &self.panels
    }
    pub fn panel(&self) -> &Panel {
        &self.panels.panels[self.active_panel_idx()]
    }
    pub fn mut_panel(&mut self) -> &mut Panel {
        let idx = self.active_panel_idx();
        &mut self.panels.panels[idx]
    }
    pub fn preview_panel_id(&self) -> Option<PanelId> {
        self.panels.by_type(PanelStateType::Preview).map(|p| p.id)
    }
    pub fn stage_panel_id(&self) -> Option<PanelId> {
        self.panels.by_type(PanelStateType::Stage).map(|p| p.id)
    }
    pub fn idx_by_ref(
        &self,
        panel_ref: PanelReference,
    ) -> Option<usize> {
        self.panels.idx_by_ref(panel_ref)
    }

    pub fn has_preview_panel(&self) -> bool {
        self.panels.has_type(PanelStateType::Preview)
    }
    pub fn has_stage_panel(&self) -> bool {
        self.panels.has_type(PanelStateType::Stage)
    }
    pub fn active_panel_idx(&self) -> usize {
        self.panels.active_panel_idx
    }
    pub fn preview_panel_idx(&self) -> Option<usize> {
        self.panels.idx_by_type(PanelStateType::Preview)
    }
    pub fn panel_by_idx_unchecked(
        &self,
        idx: usize,
    ) -> &Panel {
        &self.panels.panels[idx]
    }
    pub fn count_of_type(
        &self,
        state_type: PanelStateType,
    ) -> usize {
        self.panels
            .panels
            .iter()
            .filter(|panel| panel.state().get_type() == state_type)
            .count()
    }


    // ----------------------------------------------------
    // panel manipulation

    pub fn new_panel(
        &mut self,
        state: Box<dyn PanelState>,
        purpose: PanelPurpose,
        direction: HDir,
        activate: bool,
        con: &AppContext,
    ) -> Result<(), String> {
        let screen = self.screen();
        match state.get_type() {
            PanelStateType::Preview if self.panels.has_type(PanelStateType::Preview) => {
                return Err("There can be only one preview panel".to_owned());
                // todo replace instead ?
            }
            PanelStateType::Stage if self.panels.has_type(PanelStateType::Stage) => {
                return Err("There can be only one stage panel".to_owned());
                // todo replace instead ?
            }
            _ => {}
        }
        let insertion_idx = if purpose.is_preview() {
            self.len()
        } else if direction == HDir::Right {
            self.active_panel_idx() + 1
        } else {
            self.active_panel_idx()
        };
        let with_preview = purpose.is_preview() || self.panels.has_type(PanelStateType::Preview);
        let areas = Areas::create(
            self.panels.panels.as_mut_slice(),
            &con.layout_instructions,
            insertion_idx,
            screen,
            with_preview,
        );
        let mut input = PanelInput::new(areas.input.clone());
        input.set_content(&state.get_starting_input());
        let panel_id = self.created_panels_count.into();
        if activate {
            self.panels.active_panel_idx = insertion_idx;
        }
        let mut panel = Panel::new(panel_id, state, areas, con);
        panel.purpose = purpose;
        self.created_panels_count += 1;
        self.panels.panels.insert(insertion_idx, panel);
        self.inputs.insert(insertion_idx, input);
        Ok(())
    }

    pub fn activate(
        &mut self,
        panel_idx: usize,
    ) {
        if panel_idx < self.len() {
            self.panels.active_panel_idx = panel_idx;
        }
    }
    pub fn focus_by_type( // FIXME unconsistent naming with activate
        &mut self,
        state_type: PanelStateType,
    ) -> bool {
        if let Some(idx) = self.panels.idx_by_type(state_type) {
            self.panels.active_panel_idx = idx;
            true
        } else {
            false
        }
    }

    /// close the panel if it's not the last one
    ///
    /// Return true when the panel has been removed (ie it wasn't the last one)
    pub fn close(
        &mut self,
        panel_idx: usize,
        con: &AppContext,
    ) -> bool {
        let len = self.len();
        let screen = self.screen();
        if panel_idx >= len {
            return false;
        }
        if len < 2 {
            return false; // we can't remove the last panel
        }
        if len == 2 {
            let non_removed_idx = if panel_idx == 0 { 1 } else { 0 };
            let non_removed_panel = &self.panels.panels[non_removed_idx];
            if non_removed_panel.state().get_type() == PanelStateType::Preview
                || non_removed_panel.state().get_type() == PanelStateType::Stage
            {
                return false; // we don't want to stay with just the preview or stage
            }
        }
        let active_panel_id = self.panels.panels[self.active_panel_idx()].id;
        self.panels.panels.remove(panel_idx);
        self.inputs.remove(panel_idx);
        let has_preview = self.panels.has_type(PanelStateType::Preview);
        Areas::resize_all(
            &mut self.panels.panels,
            &con.layout_instructions,
            screen,
            has_preview,
        );
        self.panels.active_panel_idx = self
            .panels
            .panels
            .iter()
            .position(|p| p.id == active_panel_id)
            .unwrap_or(self.len() - 1);
        true
    }

    pub fn close_first_non_tree(
        &mut self,
        con: &AppContext,
    ) -> bool {
        let idx = self
            .panels
            .panels
            .iter()
            .position(|panel| panel.state().get_type() != PanelStateType::Tree);
        if let Some(idx) = idx {
            self.close(idx, con)
        } else {
            false
        }
    }

    pub fn close_rightest_inactive_tree(
        &mut self,
        con: &AppContext,
    ) -> bool {
        let idx = self
            .panels
            .panels
            .iter()
            .enumerate()
            .rev()
            .find(|(idx, panel)| {
                *idx != self.active_panel_idx() && panel.state().get_type() == PanelStateType::Tree
            })
            .map(|(idx, _)| idx);
        if let Some(idx) = idx {
            self.close(idx, con)
        } else {
            false
        }
    }

    // ----------------------------------------------------
    // event handling

    /// get the index of the panel at x
    pub fn clicked_panel_index(
        &self,
        x: u16,
        _y: u16,
    ) -> usize {
        let len = self.len();
        for (idx, panel) in self.panels.panels.iter().enumerate() {
            let area = &panel.areas.state;
            if area.left <= x && x < area.left + area.width {
                return idx;
            }
        }
        // fallback: distribute evenly, but it misses that panels
        // may have different widths
        (len * x as usize) / (self.screen().width as usize + 1)
    }

    pub fn on_input_event(
        &mut self,
        w: &mut W,
        timed_event: &TimedEvent,
        app_state: &AppState,
        con: &AppContext,
    ) -> Result<Command, ProgramError> {
        let panel_idx = self.panels.active_panel_idx;
        debug!("input event for panel idx: {} / {}", panel_idx, self.len());
        self.inputs[panel_idx].on_event(w, timed_event, &self.panels, app_state, con)
    }

    // ----------------------------------------------------
    // command execution

    fn app_cmd_context<'c>(
        &self,
        panel_skin: &'c PanelSkin,
        con: &'c AppContext,
    ) -> AppCmdContext<'c> {
        AppCmdContext {
            panel_skin,
            preview_panel: self.preview_panel_id(),
            stage_panel: self.stage_panel_id(),
            screen: self.screen(),
            con,
        }
    }

    pub fn on_input_internal(
        &mut self,
        internal: Internal,
    ) -> Command {
        let idx = self.active_panel_idx();
        self.inputs[idx].on_internal(internal)
    }

    pub fn apply_command<'c>(
        &mut self,
        w: &'c mut W,
        cmd: &'c Command,
        // A panel ref which may override the one in the command
        panel_ref: Option<PanelReference>,
        panel_skin: &PanelSkin,
        app_state: &mut AppState,
        con: &AppContext,
    ) -> Result<CmdResult, ProgramError> {
        let panel_ref = panel_ref.unwrap_or_else(|| {
            cmd.triggered_verb(&con.verb_store)
                .map(|v| v.impacted_panel)
                .unwrap_or(PanelReference::Active)
        });
        let panel_idx = self
            .panels
            .idx_by_ref(panel_ref)
            .unwrap_or(self.active_panel_idx());

        let app_cmd_context = self.app_cmd_context(panel_skin, con);
        self.panels.panels[panel_idx].apply_command(w, cmd, app_state, &app_cmd_context)
    }

    pub fn has_pending_task(&mut self) -> bool {
        self.panels.panels.iter().any(Panel::has_pending_task)
    }
    /// do the pending tasks, if any, and refresh the screen accordingly
    pub fn do_pending_tasks(
        &mut self,
        w: &mut W,
        skin: &AppSkin,
        dam: &mut Dam,
        app_state: &mut AppState,
        con: &AppContext,
    ) -> Result<(), ProgramError> {
        while self.has_pending_task() && !dam.has_event() {
            let error = self.do_pending_task(app_state, con, dam).err();
            self.update_preview(false, con); // the selection may have changed
            if let Some(error) = &error {
                self.mut_panel().set_error(error.to_string());
            } else {
                let panel_skin = &skin.focused;
                self.refresh_input_status(app_state, panel_skin, con);
            }
            self.display_panels(w, skin, app_state, con)?;
            if error.is_some() {
                return Ok(()); // breaking pending tasks chain on first error/interruption
            }
        }
        Ok(())
    }
    /// Do the next pending task
    pub fn do_pending_task(
        &mut self,
        app_state: &mut AppState,
        con: &AppContext,
        dam: &mut Dam,
    ) -> Result<(), ProgramError> {
        let screen = self.screen();
        // we start with the focused panel
        let active_panel_idx = self.active_panel_idx();
        if self.panels.panels[active_panel_idx].has_pending_task() {
            return self.panels.panels[active_panel_idx]
                .do_pending_task(app_state, screen, con, dam);
        }
        // then the other ones
        for idx in 0..self.len() {
            if idx != self.active_panel_idx() {
                let panel = &mut self.panels.panels[idx];
                if panel.has_pending_task() {
                    return panel.do_pending_task(app_state, screen, con, dam);
                }
            }
        }
        warn!("unexpected lack of pending task");
        Ok(())
    }

    pub fn refresh_active_panel(
        &mut self,
        con: &AppContext,
    ) { // FIXME the returned command is never used
        let idx = self.active_panel_idx();
        let screen = self.screen();
        let panel = &mut self.panels.panels[idx];
        panel.mut_state().refresh(screen, con);
    }
    pub fn refresh_all_panels(
        &mut self,
        con: &AppContext,
    ) {
        let screen = self.screen();
        for panel in &mut self.panels.panels {
            panel.mut_state().refresh(screen, con);
        }
    }
    pub fn refresh_input_status(
        &mut self,
        app_state: &mut AppState,
        panel_skin: &PanelSkin,
        con: &AppContext,
    ) {
        let idx = self.active_panel_idx();
        let has_previous_state = self.len() > 1;
        let input = &mut self.inputs[idx];
        let panel = &mut self.panels.panels[idx];
        let cmd = Command::from_raw(input.get_content(), false);
        let areas = panel.areas.clone(); // FIXME avoid clone
        let purpose = panel.purpose;
        let status_width = panel.areas.status.width as usize;
        let app_cmd_context = self.app_cmd_context(panel_skin, con);
        let cc = CmdContext {
            cmd: &cmd,
            app: &app_cmd_context,
            panel: PanelCmdContext {
                areas: &areas,
                purpose,
            },
        };
        let status = self.state().get_status(
            app_state,
            &cc,
            has_previous_state,
            status_width,
        );
        self.panels.panels[idx].status = status;
    }

    /// update the state of the preview, if there's some
    pub fn update_preview(
        &mut self,
        refresh: bool,
        con: &AppContext,
    ) {
        let Some(preview_idx) = self.panels.idx_by_type(PanelStateType::Preview) else {
            return;
        };
        if let Some(path) = self.state().selected_path() {
            let old_path = self.panels.panels[preview_idx].state().selected_path();
            if refresh || Some(path) != old_path {
                let path = path.to_path_buf();
                self.panels.panels[preview_idx]
                    .mut_state()
                    .set_selected_path(path, con);
            }
        }
    }

    // ----------------------------------------------------
    // Input access and manipulation

    pub fn input(&mut self) -> &mut PanelInput {
        &mut self.inputs[self.panels.active_panel_idx]
    }
    pub fn get_input_content(&self) -> String {
        self.inputs[self.panels.active_panel_idx].get_content()
    }
    /// change the argument of the verb in the input, if there's one
    pub fn set_input_arg(
        &mut self,
        arg: String,
    ) {
        let input = &mut self.inputs[self.panels.active_panel_idx];
        let mut command_parts = CommandParts::from(input.get_content());
        if let Some(invocation) = &mut command_parts.verb_invocation {
            invocation.args = Some(arg);
            let new_input = format!("{command_parts}");
            input.set_content(&new_input);
        }
    }
    pub fn do_input_escape(
        &mut self,
        mode: Mode,
        con: &AppContext,
    ) -> Command {
        let panel_idx = self.panels.active_panel_idx;
        let input = &mut self.inputs[panel_idx];
        input.escape(mode, con)
    }
    pub fn clear_input(&mut self) {
        let panel_idx = self.panels.active_panel_idx;
        if let Some(input) = self.inputs.get_mut(panel_idx) {
            input.input_field.clear();
        }
    }

    /// remove the verb invocation from the input but keep
    /// the filter if there's one
    pub fn clear_input_invocation(
        &mut self,
        con: &AppContext,
    ) {
        let panel_idx = self.panels.active_panel_idx;
        let mut command_parts = CommandParts::from(self.inputs[panel_idx].get_content());
        if command_parts.verb_invocation.is_some() {
            command_parts.verb_invocation = None;
            let new_input = format!("{command_parts}");
            self.inputs[panel_idx].set_content(&new_input);
        }
        self.mut_state().set_mode(con.initial_mode());
    }

    // ----------------------------------------------------
    // drawing

    /// redraw the whole screen. All drawing
    /// are supposed to happen here, and only here.
    pub fn display_panels(
        &mut self,
        w: &mut W,
        skin: &AppSkin,
        app_state: &AppState,
        con: &AppContext,
    ) -> Result<(), ProgramError> {
        self.drawing_count += 1;
        let screen = self.screen();
        let mut cursor_pos = None;
        let active_panel_idx = self.active_panel_idx();
        for (idx, panel) in self.panels.panels.iter_mut().enumerate() {
            let input = &mut self.inputs[idx];
            let active = idx == active_panel_idx;
            let panel_skin = if active {
                &skin.focused
            } else {
                &skin.unfocused
            };
            let disc = DisplayContext {
                count: self.drawing_count,
                active,
                screen,
                panel_skin,
                state_area: panel.areas.state.clone(),
                app_state,
                con,
            };
            panel.mut_state().display(w, &disc)?;
            if disc.active || !WIDE_STATUS {
                let watching = disc.app_state.watch_tree;
                panel.write_status(w, watching, disc.panel_skin, disc.screen)?;
            }
            let mut input_area = panel.areas.input.clone();
            if disc.active {
                panel.write_purpose(w, disc.panel_skin, disc.screen, disc.con)?;
                let flags = panel.state().get_flags();
                #[allow(clippy::cast_possible_truncation)]
                let input_content_len = input.get_content().len() as u16;
                let flags_len = flags_display::visible_width(&flags);
                if input_area.width > input_content_len + 1 + flags_len {
                    input_area.width -= flags_len + 1;
                    disc.screen
                        .goto(w, input_area.left + input_area.width, input_area.top)?;
                    flags_display::write(w, &flags, disc.panel_skin)?;
                }
            }
            let mode = panel.state().get_mode();
            if let Some(pos) = input.display(w, disc.active, mode, input_area, disc.panel_skin)? {
                cursor_pos = Some(pos);
            }

        }

        // after drawing all the panels, move cursor to the end of the active panel input,
        // so that input methods can popup at correct position.
        if let Some(cursor_pos) = cursor_pos {
            queue!(w, MoveTo(cursor_pos.0, cursor_pos.1))?;
        }

        kitty::manager()
            .lock()
            .unwrap()
            .erase_images_before(w, self.drawing_count)?;
        w.flush()?;
        Ok(())
    }

}


impl AppPanels {
    fn idx_by_type(
        &self,
        state_type: PanelStateType,
    ) -> Option<usize> {
        self.panels
            .iter()
            .position(|panel| panel.state().get_type() == state_type)
    }
    fn idx_by_ref(
        &self,
        panel_ref: PanelReference,
    ) -> Option<usize> {
        match panel_ref {
            PanelReference::Active => Some(self.active_panel_idx),
            PanelReference::Leftest => Some(0),
            PanelReference::Rightest => Some(self.panels.len() - 1),
            PanelReference::Idx(idx) => {
                if idx < self.panels.len() {
                    Some(idx)
                } else {
                    None
                }
            }
            PanelReference::Id(id) => self.panels.iter().position(|panel| panel.id == id),
            PanelReference::Preview => self.idx_by_type(PanelStateType::Preview),
        }
    }
    fn by_type(
        &self,
        state_type: PanelStateType,
    ) -> Option<&Panel> {
        self.panels
            .iter()
            .find(|panel| panel.state().get_type() == state_type)
    }
    fn by_ref(
        &self,
        panel_ref: PanelReference,
    ) -> Option<&Panel> {
        self.panels.get(self.idx_by_ref(panel_ref)?)
    }
    fn has_type(
        &self,
        state_type: PanelStateType,
    ) -> bool {
        self.idx_by_type(state_type).is_some()
    }

    pub fn state(&self) -> &dyn PanelState {
        self.panels[self.active_panel_idx].state()
    }
    pub fn mut_state(&mut self) -> &mut dyn PanelState {
        self.panels[self.active_panel_idx].mut_state()
    }
    pub fn state_by_ref(
        &self,
        panel_ref: PanelReference,
    ) -> Option<&dyn PanelState> {
        self.by_ref(panel_ref).map(|panel| panel.state())
    }
}
