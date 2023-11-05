use {
    super::*,
    crate::{
        app::*,
        display::W,
        errors::ProgramError,
        keys,
        skin::PanelSkin,
        verb::*,
    },
    crokey::key,
    crokey::crossterm::{
        cursor,
        event::{
            Event,
            KeyEvent,
            KeyModifiers,
            MouseButton,
            MouseEvent,
            MouseEventKind,
        },
        queue,
    },
    termimad::{Area, TimedEvent, InputField},
};

/// Wrap the input of a panel, receive events and make commands
pub struct PanelInput {
    pub input_field: InputField,
    tab_cycle_count: usize,
    input_before_cycle: Option<String>,
}

impl PanelInput {

    pub fn new(area: Area) -> Self {
        Self {
            input_field: InputField::new(area),
            tab_cycle_count: 0,
            input_before_cycle: None,
        }
    }

    pub fn set_content(&mut self, content: &str) {
        self.input_field.set_str(content);
    }

    pub fn get_content(&self) -> String {
        self.input_field.get_content()
    }

    pub fn display(
        &mut self,
        w: &mut W,
        active: bool,
        mode: Mode,
        mut area: Area,
        panel_skin: &PanelSkin,
    ) -> Result<(), ProgramError> {
        self.input_field.set_normal_style(panel_skin.styles.input.clone());
        self.input_field.set_focus(active && mode == Mode::Input);
        if mode == Mode::Command && active {
            queue!(w, cursor::MoveTo(area.left, area.top))?;
            panel_skin.styles.mode_command_mark.queue_str(w, "C")?;
            area.width -= 1;
            area.left += 1;
        }
        self.input_field.set_area(area);
        self.input_field.display_on(w)?;
        Ok(())
    }

    /// consume the event to
    /// - maybe change the input
    /// - build a command
    /// then redraw the input field
    #[allow(clippy::too_many_arguments)]
    pub fn on_event(
        &mut self,
        w: &mut W,
        timed_event: TimedEvent,
        con: &AppContext,
        sel_info: SelInfo<'_>,
        app_state: &AppState,
        mode: Mode,
        panel_state_type: PanelStateType,
    ) -> Result<Command, ProgramError> {
        let cmd = match timed_event.event {
            Event::Mouse(MouseEvent { kind, column, row, modifiers: KeyModifiers::NONE }) => {
                self.on_mouse(timed_event, kind, column, row)
            }
            Event::Key(key) => {
                self.on_key(timed_event, key, con, sel_info, app_state, mode, panel_state_type)
            }
            _ => Command::None,
        };
        self.input_field.display_on(w)?;
        Ok(cmd)
    }

    /// check whether the verb is an action on the input (like
    /// deleting a word) and if it's the case, applies it and
    /// return true
    fn handle_input_related_verb(
        &mut self,
        verb: &Verb,
        _con: &AppContext,
    ) -> bool {
        if let VerbExecution::Internal(internal_exec) = &verb.execution {
            match internal_exec.internal {
                Internal::input_clear => {
                    if self.input_field.get_content().is_empty() {
                        false
                    } else {
                        self.input_field.clear();
                        true
                    }
                }
                Internal::input_del_char_left => self.input_field.del_char_left(),
                Internal::input_del_char_below => self.input_field.del_char_below(),
                Internal::input_del_word_left => self.input_field.del_word_left(),
                Internal::input_del_word_right => self.input_field.del_word_right(),
                Internal::input_go_left => self.input_field.move_left(),
                Internal::input_go_right => self.input_field.move_right(),
                Internal::input_go_word_left => self.input_field.move_word_left(),
                Internal::input_go_word_right => self.input_field.move_word_right(),
                Internal::input_go_to_start => self.input_field.move_to_start(),
                Internal::input_go_to_end => self.input_field.move_to_end(),
                #[cfg(feature = "clipboard")]
                Internal::input_selection_cut => {
                    let s = self.input_field.cut_selection();
                    if let Err(err) = terminal_clipboard::set_string(s) {
                        warn!("error in writing into clipboard: {}", err);
                    }
                    true
                }
                #[cfg(feature = "clipboard")]
                Internal::input_selection_copy => {
                    let s = self.input_field.copy_selection();
                    if let Err(err) = terminal_clipboard::set_string(s) {
                        warn!("error in writing into clipboard: {}", err);
                    }
                    true
                }
                #[cfg(feature = "clipboard")]
                Internal::input_paste => {
                    match terminal_clipboard::get_string() {
                        Ok(pasted) => {
                            for c in pasted
                                .chars()
                                .filter(|c| c.is_alphanumeric() || c.is_ascii_punctuation())
                            {
                                self.input_field.put_char(c);
                            }
                        }
                        Err(e) => {
                            warn!("Error in reading clipboard: {:?}", e);
                        }
                    }
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }

    /// when a key is used to enter input mode, we don't always
    /// consume it. Sometimes it should be consumed, sometimes it
    /// should be added to the input
    fn enter_input_mode_with_key(
        &mut self,
        key: KeyEvent,
        parts: &CommandParts,
    ) {
        if let Some(c) = crokey::as_letter(key) {
            let add = match c {
                // '/' if !parts.raw_pattern.is_empty() => true,
                ' ' if parts.verb_invocation.is_none() => true,
                ':' if parts.verb_invocation.is_none() => true,
                _ => false,
            };
            if add {
                self.input_field.put_char(c);
            }
        }
    }

    /// escape (bound to the 'esc' key)
    ///
    /// This function is better called from the on_key method of
    /// panel input, when a key triggers it, because then it
    /// can also properly deal with completion sequence.
    /// When ':escape' is called from a verb's cmd sequence, then
    /// it's not called on on_key but by the app.
    pub fn escape(
        &mut self,
        con: &AppContext,
        mode: Mode,
    ) -> Command {
        self.tab_cycle_count = 0;
        if let Some(raw) = self.input_before_cycle.take() {
            // we cancel the tab cycling
            self.input_field.set_str(&raw);
            self.input_before_cycle = None;
            Command::from_raw(raw, false)
        } else if con.modal && mode == Mode::Input {
            // leave insertion mode
            Command::Internal {
                internal: Internal::mode_command,
                input_invocation: None,
            }
        } else {
            // general back command
            let raw = self.input_field.get_content();
            let parts = CommandParts::from(raw.clone());
            self.input_field.clear();
            let internal = Internal::back;
            Command::Internal {
                internal,
                input_invocation: parts.verb_invocation,
            }
        }
    }

    /// autocomplete a verb (bound to 'tab')
    fn auto_complete_verb(
        &mut self,
        con: &AppContext,
        sel_info: SelInfo<'_>,
        raw: String,
        parts: CommandParts,
    ) -> Command {
        let parts_before_cycle;
        let completable_parts = if let Some(s) = &self.input_before_cycle {
            parts_before_cycle = CommandParts::from(s.clone());
            &parts_before_cycle
        } else {
            &parts
        };
        let completions = Completions::for_input(completable_parts, con, sel_info);
        info!(" -> completions: {:?}", &completions);
        let added = match completions {
            Completions::None => {
                debug!("nothing to complete!");
                self.tab_cycle_count = 0;
                self.input_before_cycle = None;
                None
            }
            Completions::Common(completion) => {
                self.tab_cycle_count = 0;
                Some(completion)
            }
            Completions::List(mut completions) => {
                let idx = self.tab_cycle_count % completions.len();
                if self.tab_cycle_count == 0 {
                    self.input_before_cycle = Some(raw.to_string());
                }
                self.tab_cycle_count += 1;
                Some(completions.swap_remove(idx))
            }
        };
        if let Some(added) = added {
            let mut raw = self
                .input_before_cycle
                .as_ref()
                .map_or(raw, |s| s.to_string());
            raw.push_str(&added);
            self.input_field.set_str(&raw);
            Command::from_raw(raw, false)
        } else {
            Command::None
        }
    }

    fn find_key_verb<'c>(
        &mut self,
        key: KeyEvent,
        con: &'c AppContext,
        sel_info: SelInfo<'_>,
        panel_state_type: PanelStateType,
    ) -> Option<&'c Verb> {
        for verb in con.verb_store.verbs().iter() {
            // note that there can be several verbs with the same key and
            // not all of them can apply
            if !verb.keys.contains(&key) {
                continue;
            }
            if !sel_info.is_accepted_by(verb.selection_condition) {
                continue;
            }
            if !verb.can_be_called_in_panel(panel_state_type) {
                continue;
            }
            if !verb.file_extensions.is_empty() {
                let extension = sel_info.extension();
                if !extension.map_or(false, |ext| verb.file_extensions.iter().any(|ve| ve == ext)) {
                    continue;
                }
            }
            debug!("verb for key: {}", &verb.execution);
            return Some(verb);
        }
        None
    }

    /// Consume the event, maybe change the input, return a command
    fn on_mouse(
        &mut self,
        timed_event: TimedEvent,
        kind: MouseEventKind,
        column: u16,
        row: u16,
    ) -> Command {
        if self.input_field.apply_timed_event(timed_event) {
            Command::empty()
        } else {
            match kind {
                MouseEventKind::Up(MouseButton::Left) => {
                    if timed_event.double_click {
                        Command::DoubleClick(column, row)
                    } else {
                        Command::Click(column, row)
                    }
                }
                MouseEventKind::ScrollDown => {
                    Command::Internal {
                        internal: Internal::line_down,
                        input_invocation: None,
                    }
                }
                MouseEventKind::ScrollUp => {
                    Command::Internal {
                        internal: Internal::line_up,
                        input_invocation: None,
                    }
                }
                _ => Command::None,
            }
        }
    }

    fn is_key_allowed_for_verb(
        &self,
        key: KeyEvent,
        mode: Mode,
    ) -> bool {
        match mode {
            Mode::Input => match key {
                key!(left) => !self.input_field.can_move_left(),
                key!(right) => !self.input_field.can_move_right(),
                _ => !keys::is_key_only_modal(key),
            }
            Mode::Command => true,
        }
    }

    /// Consume the event, maybe change the input, return a command
    #[allow(clippy::too_many_arguments)]
    fn on_key(
        &mut self,
        timed_event: TimedEvent,
        key: KeyEvent,
        con: &AppContext,
        sel_info: SelInfo<'_>,
        app_state: &AppState,
        mode: Mode,
        panel_state_type: PanelStateType,
    ) -> Command {

        // value of raw and parts before any key related change
        let raw = self.input_field.get_content();
        let parts = CommandParts::from(raw.clone());

        let verb = if self.is_key_allowed_for_verb(key, mode) {
            self.find_key_verb(
                key,
                con,
                sel_info,
                panel_state_type,
            )
        } else {
            None
        };

        // WARNINGS:
        // - beware the execution order below: we must execute
        // escape before the else clause of next_match, and we must
        // be sure this else clause (which ends cycling) is always
        // executed when neither next_match or escape is triggered
        // - some behaviors can't really be handled as normally
        // triggered internals because of the interactions with
        // the input

        // usually 'esc' key
        if Verb::is_some_internal(verb, Internal::escape) {
            return self.escape(con, mode);
        }

        // 'tab' completion of a verb or one of its arguments
        if Verb::is_some_internal(verb, Internal::next_match) {
            if parts.verb_invocation.is_some() {
                return self.auto_complete_verb(con, sel_info, raw, parts);
            }
            // if no verb is being edited, the state may handle this internal
            // in a specific way
        } else {
            self.tab_cycle_count = 0;
            self.input_before_cycle = None;
        }

        // 'enter': trigger the verb if any on the input. If none, then may be
        // used as trigger of another verb
        if key == key!(enter) && parts.has_not_empty_verb_invocation() {
            return Command::from_parts(parts, true);
        }

        // a '?' opens the help when it's the first char or when it's part
        // of the verb invocation. It may be used as a verb name in other cases
        if (key == key!('?') || key == key!(shift-'?'))
            && (raw.is_empty() || parts.verb_invocation.is_some()) {
            return Command::Internal {
                internal: Internal::help,
                input_invocation: parts.verb_invocation,
            };
        }

        if let Some(verb) = verb {
            if self.handle_input_related_verb(verb, con) {
                return Command::from_raw(self.input_field.get_content(), false);
            }
            if mode != Mode::Input && verb.is_internal(Internal::mode_input) {
                self.enter_input_mode_with_key(key, &parts);
            }
            if verb.auto_exec {
                return Command::VerbTrigger {
                    verb_id: verb.id,
                    input_invocation: parts.verb_invocation,
                };
            }
            if let Some(invocation_parser) = &verb.invocation_parser {
                let exec_builder = ExecutionStringBuilder::without_invocation(
                    sel_info,
                    app_state,
                );
                let verb_invocation = exec_builder.invocation_with_default(
                    &invocation_parser.invocation_pattern
                );
                let mut parts = parts;
                parts.verb_invocation = Some(verb_invocation);
                self.set_content(&parts.to_string());
                return Command::VerbEdit(parts.verb_invocation.unwrap());
            }
        }

        // input field management
        if mode == Mode::Input && self.input_field.apply_timed_event(timed_event) {
            return Command::from_raw(self.input_field.get_content(), false);
        }
        Command::None
    }
}

