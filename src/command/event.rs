use {
    super::*,
    crate::{
        app::{
            AppContext,
            AppState,
        },
        display::W,
        errors::ProgramError,
        keys,
        skin::PanelSkin,
        verb::{Internal, Verb, VerbExecution},
    },
    termimad::{Area, Event, InputField},
};

/// wrap the input of a panel,
/// receive events and make commands
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
        self.input_field.set_content(content);
    }

    pub fn get_content(&self) -> String {
        self.input_field.get_content()
    }

    pub fn display(
        &mut self,
        w: &mut W,
        active: bool,
        area: Area,
        panel_skin: &PanelSkin,
    ) -> Result<(), ProgramError> {
        self.input_field.set_normal_style(panel_skin.styles.input.clone());
        self.input_field.focused = active;
        self.input_field.area = area;
        self.input_field.display_on(w)?;
        Ok(())
    }

    /// consume the event to
    /// - maybe change the input
    /// - build a command
    pub fn on_event(
        &mut self,
        w: &mut W,
        event: Event,
        con: &AppContext,
        state: &dyn AppState,
    ) -> Result<Command, ProgramError> {
        let cmd = self.get_command(event, con, state);
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
                _ => false,
            }
        } else {
            false
        }
    }

    /// consume the event to
    /// - maybe change the input
    /// - build a command
    fn get_command(
        &mut self,
        event: Event,
        con: &AppContext,
        state: &dyn AppState,
    ) -> Command {
        match event {
            Event::Click(x, y, ..) => {
                return if self.input_field.apply_event(&event) {
                    Command::empty()
                } else {
                    Command::Click(x, y)
                };
            }
            Event::DoubleClick(x, y) => {
                return Command::DoubleClick(x, y);
            }
            Event::Key(key) => {
                // value of raw and parts before any key related change
                let raw = self.input_field.get_content();
                let parts = CommandParts::from(raw.clone());

                // we first handle the cases that MUST absolutely
                // not be overriden by configuration

                if key == keys::ESC {
                    self.tab_cycle_count = 0;
                    if let Some(raw) = self.input_before_cycle.take() {
                        // we cancel the tab cycling
                        self.input_field.set_content(&raw);
                        self.input_before_cycle = None;
                        return Command::from_raw(raw, false);
                    } else {
                        self.input_field.set_content("");
                        let internal = Internal::back;
                        return Command::Internal {
                            internal,
                            input_invocation: parts.verb_invocation,
                        };
                    }
                }

                // tab completion
                if key == keys::TAB {
                    if parts.verb_invocation.is_some() {
                        let parts_before_cycle;
                        let completable_parts = if let Some(s) = &self.input_before_cycle {
                            parts_before_cycle = CommandParts::from(s.clone());
                            &parts_before_cycle
                        } else {
                            &parts
                        };
                        let completions = Completions::for_input(completable_parts, con, state);
                        let added = match completions {
                            Completions::None => {
                                debug!("nothing to complete!"); // where to tell this ? input field or status ?
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
                            let mut raw = self.input_before_cycle.as_ref().map_or(raw, |s| s.to_string());
                            raw.push_str(&added);
                            self.input_field.set_content(&raw);
                            return Command::from_raw(raw, false);
                        } else {
                            return Command::None;
                        }
                    }
                } else {
                    self.tab_cycle_count = 0;
                    self.input_before_cycle = None;
                }

                if key == keys::ENTER && parts.verb_invocation.is_some() {
                    return Command::from_parts(parts, true);
                }

                if key == keys::QUESTION && (raw.is_empty() || parts.verb_invocation.is_some()) {
                    // a '?' opens the help when it's the first char
                    // or when it's part of the verb invocation
                    return Command::Internal {
                        internal: Internal::help,
                        input_invocation: parts.verb_invocation,
                    };
                }

                // we now check if the key is the trigger key of one of the verbs
                let selection_type = state.selection_type();
                for (index, verb) in con.verb_store.verbs.iter().enumerate() {
                    for verb_key in &verb.keys {
                        if *verb_key == key {
                            if self.handle_input_related_verb(verb, con) {
                                return Command::from_raw(self.input_field.get_content(), false);
                            }
                            if selection_type.respects(verb.selection_condition) {
                                return Command::VerbTrigger {
                                    index,
                                    input_invocation: parts.verb_invocation,
                                };
                            } else {
                                debug!("verb not allowed on current selection");
                            }
                        }
                    }
                }

                if key == keys::LEFT && raw.is_empty() {
                    let internal = Internal::back;
                    return Command::Internal {
                        internal,
                        input_invocation: parts.verb_invocation,
                    };
                }

                if key == keys::RIGHT && raw.is_empty() {
                    return Command::Internal {
                        internal: Internal::open_stay,
                        input_invocation: None,
                    };
                }

                // input field management
                if self.input_field.apply_event(&event) {
                    return Command::from_raw(self.input_field.get_content(), false);
                }
            }
            Event::Wheel(lines_count) => {
                let internal = if lines_count > 0 {
                    Internal::line_down
                } else {
                    Internal::line_up
                };
                return Command::Internal {
                    internal,
                    input_invocation: None,
                };
            }
            _ => {}
        }
        Command::None
    }
}
