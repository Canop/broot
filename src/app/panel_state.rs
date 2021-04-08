use {
    super::*,
    crate::{
        command::*,
        display::{Screen, W},
        errors::ProgramError,
        flag::Flag,
        help::HelpState,
        pattern::*,
        preview::{PreviewMode, PreviewState},
        print,
        task_sync::Dam,
        tree::*,
        verb::*,
    },
    std::{
        path::{Path, PathBuf},
        str::FromStr,
    },
};

/// a panel state, stackable to allow reverting
///  to a previous one
pub trait PanelState {

    fn set_mode(&mut self, mode: Mode);
    fn get_mode(&self) -> Mode;

    /// called on start of on_command
    fn clear_pending(&mut self) {}

    fn on_click(
        &mut self,
        _x: u16,
        _y: u16,
        _screen: Screen,
        _con: &AppContext,
    ) -> Result<CmdResult, ProgramError> {
        Ok(CmdResult::Keep)
    }

    fn on_double_click(
        &mut self,
        _x: u16,
        _y: u16,
        _screen: Screen,
        _con: &AppContext,
    ) -> Result<CmdResult, ProgramError> {
        Ok(CmdResult::Keep)
    }

    fn on_pattern(
        &mut self,
        _pat: InputPattern,
        _con: &AppContext,
    ) -> Result<CmdResult, ProgramError> {
        Ok(CmdResult::Keep)
    }

    fn on_mode_verb(
        &mut self,
        mode: Mode,
        con: &AppContext,
    ) -> CmdResult {
        if con.modal {
            self.set_mode(mode);
            CmdResult::Keep
        } else {
            CmdResult::DisplayError(
                "modal mode not enabled in configuration".to_string()
            )
        }
    }

    /// execute the internal with the optional given invocation.
    ///
    /// The invocation comes from the input and may be related
    /// to a different verb (the verb may have been triggered
    /// by a key shorctut)
    fn on_internal(
        &mut self,
        w: &mut W,
        internal_exec: &InternalExecution,
        input_invocation: Option<&VerbInvocation>,
        trigger_type: TriggerType,
        cc: &CmdContext,
    ) -> Result<CmdResult, ProgramError>;

    /// a generic implementation of on_internal which may be
    /// called by states when they don't have a specific
    /// behavior to execute
    fn on_internal_generic(
        &mut self,
        _w: &mut W,
        internal_exec: &InternalExecution,
        input_invocation: Option<&VerbInvocation>,
        _trigger_type: TriggerType,
        cc: &CmdContext,
    ) -> Result<CmdResult, ProgramError> {
        let con = &cc.app.con;
        let screen = cc.app.screen;
        let bang = input_invocation
            .map(|inv| inv.bang)
            .unwrap_or(internal_exec.bang);
        Ok(match internal_exec.internal {
            Internal::back => CmdResult::PopState,
            Internal::copy_line | Internal::copy_path => {
                #[cfg(not(feature = "clipboard"))]
                {
                    CmdResult::DisplayError(
                        "Clipboard feature not enabled at compilation".to_string(),
                    )
                }
                #[cfg(feature = "clipboard")]
                {
                    if let Some(path) = self.selected_path() {
                        let path = path.to_string_lossy().to_string();
                        match terminal_clipboard::set_string(path) {
                            Ok(()) => CmdResult::Keep,
                            Err(_) => CmdResult::DisplayError(
                                "Clipboard error while copying path".to_string(),
                            ),
                        }
                    } else {
                        CmdResult::DisplayError(
                            "Nothing to copy".to_string(),
                        )
                    }
                }
            }
            Internal::close_panel_ok => CmdResult::ClosePanel {
                validate_purpose: true,
                panel_ref: PanelReference::Active,
            },
            Internal::close_panel_cancel => CmdResult::ClosePanel {
                validate_purpose: false,
                panel_ref: PanelReference::Active,
            },
            #[cfg(unix)]
            Internal::filesystems => {
                let fs_state = crate::filesystems::FilesystemState::new(
                    self.selected_path(),
                    self.tree_options(),
                    con,
                );
                match fs_state {
                    Ok(state) => {
                        let bang = input_invocation
                            .map(|inv| inv.bang)
                            .unwrap_or(internal_exec.bang);
                        if bang && cc.has_no_preview() {
                            CmdResult::NewPanel {
                                state: Box::new(state),
                                purpose: PanelPurpose::None,
                                direction: HDir::Right,
                            }
                        } else {
                            CmdResult::NewState(Box::new(state))
                        }
                    }
                    Err(e) => CmdResult::DisplayError(format!("{}", e)),
                }
            }
            Internal::help => {
                let bang = input_invocation
                    .map(|inv| inv.bang)
                    .unwrap_or(internal_exec.bang);
                if bang && cc.has_no_preview() {
                    CmdResult::NewPanel {
                        state: Box::new(HelpState::new(self.tree_options(), screen, con)),
                        purpose: PanelPurpose::None,
                        direction: HDir::Right,
                    }
                } else {
                    CmdResult::NewState(Box::new(
                            HelpState::new(self.tree_options(), screen, con)
                    ))
                }
            }
            Internal::mode_input => self.on_mode_verb(Mode::Input, con),
            Internal::mode_command => self.on_mode_verb(Mode::Command, con),
            Internal::open_leave => {
                if let Some(selection) = self.selection() {
                    selection.to_opener(con)?
                } else {
                    CmdResult::DisplayError("no selection to open".to_string())
                }
            }
            Internal::open_preview => self.open_preview(None, false, cc),
            Internal::preview_image => self.open_preview(Some(PreviewMode::Image), false, cc),
            Internal::preview_text => self.open_preview(Some(PreviewMode::Text), false, cc),
            Internal::preview_binary => self.open_preview(Some(PreviewMode::Hex), false, cc),
            Internal::toggle_preview => self.open_preview(None, true, cc),
            Internal::sort_by_count => self.with_new_options(
                screen,
                &|o| {
                    if o.sort == Sort::Count {
                        o.sort = Sort::None;
                        o.show_counts = false;
                    } else {
                        o.sort = Sort::Count;
                        o.show_counts = true;
                    }
                },
                bang,
                con,
            ),
            Internal::sort_by_date => self.with_new_options(
                screen,
                &|o| {
                    if o.sort == Sort::Date {
                        o.sort = Sort::None;
                        o.show_dates = false;
                    } else {
                        o.sort = Sort::Date;
                        o.show_dates = true;
                    }
                },
                bang,
                con,
            ),
            Internal::sort_by_size => self.with_new_options(
                screen,
                &|o| {
                    if o.sort == Sort::Size {
                        o.sort = Sort::None;
                        o.show_sizes = false;
                    } else {
                        o.sort = Sort::Size;
                        o.show_sizes = true;
                        o.show_root_fs = true;
                    }
                },
                bang,
                con,
            ),
            Internal::no_sort => self.with_new_options(screen, &|o| o.sort = Sort::None, bang, con),
            Internal::toggle_counts => {
                self.with_new_options(screen, &|o| o.show_counts ^= true, bang, con)
            }
            Internal::toggle_dates => {
                self.with_new_options(screen, &|o| o.show_dates ^= true, bang, con)
            }
            Internal::toggle_files => {
                self.with_new_options(screen, &|o: &mut TreeOptions| o.only_folders ^= true, bang, con)
            }
            Internal::toggle_hidden => {
                self.with_new_options(screen, &|o| o.show_hidden ^= true, bang, con)
            }
            Internal::toggle_root_fs => {
                self.with_new_options(screen, &|o| o.show_root_fs ^= true, bang, con)
            }
            Internal::toggle_git_ignore => {
                self.with_new_options(screen, &|o| o.respect_git_ignore ^= true, bang, con)
            }
            Internal::toggle_git_file_info => {
                self.with_new_options(screen, &|o| o.show_git_file_info ^= true, bang, con)
            }
            Internal::toggle_git_status => {
                self.with_new_options(
                    screen, &|o| {
                        if o.filter_by_git_status {
                            o.filter_by_git_status = false;
                        } else {
                            o.filter_by_git_status = true;
                            o.show_hidden = true;
                        }
                    }, bang, con
                )
            }
            Internal::toggle_perm => {
                self.with_new_options(screen, &|o| o.show_permissions ^= true, bang, con)
            }
            Internal::toggle_sizes => self.with_new_options(
                screen,
                &|o| {
                    if o.show_sizes {
                        o.show_sizes = false;
                        o.show_root_fs = false;
                    } else {
                        o.show_sizes = true;
                        o.show_root_fs = true;
                    }
                },
                bang,
                con,
            ),
            Internal::toggle_trim_root => {
                self.with_new_options(screen, &|o| o.trim_root ^= true, bang, con)
            }
            Internal::close_preview => {
                if let Some(id) = cc.app.preview {
                    CmdResult::ClosePanel {
                        validate_purpose: false,
                        panel_ref: PanelReference::Id(id),
                    }
                } else {
                    CmdResult::Keep
                }
            }
            Internal::panel_left => {
                CmdResult::HandleInApp(Internal::panel_left)
            }
            Internal::panel_right => {
                CmdResult::HandleInApp(Internal::panel_right)
            }
            Internal::clear_stage => {
                CmdResult::HandleInApp(Internal::clear_stage)
            }
            Internal::stage => {
                CmdResult::HandleInApp(Internal::stage)
            }
            Internal::unstage => {
                CmdResult::HandleInApp(Internal::unstage)
            }
            Internal::toggle_stage => {
                if let Some(path) = self.selected_path() {
                    if cc.app.app_state.stage.paths.iter().any(|p| p==path) {
                        CmdResult::HandleInApp(Internal::unstage)
                    } else {
                        CmdResult::HandleInApp(Internal::stage)
                    }
                } else {
                    CmdResult::error("no selection")
                }
            }
            Internal::print_path => {
                if let Some(path) = self.selected_path() {
                    print::print_path(path, con)?
                } else {
                    CmdResult::DisplayError("no selection to print".to_string())
                }
            }
            Internal::print_relative_path => {
                if let Some(path) = self.selected_path() {
                    print::print_relative_path(path, con)?
                } else {
                    CmdResult::DisplayError("no selection to print".to_string())
                }
            }
            Internal::refresh => CmdResult::RefreshState { clear_cache: true },
            Internal::quit => CmdResult::Quit,
            _ => CmdResult::Keep,
        })
    }

    fn execute_verb(
        &mut self,
        w: &mut W, // needed because we may want to switch from alternate in some externals
        verb: &Verb,
        invocation: Option<&VerbInvocation>,
        trigger_type: TriggerType,
        cc: &CmdContext,
    ) -> Result<CmdResult, ProgramError> {
        if verb.needs_selection && !self.has_at_least_one_selection(cc) {
            return Ok(CmdResult::DisplayError(
                "This verb needs a selection".to_string()
            ));
        }
        if verb.needs_another_panel && cc.app.other_path.is_none() {
            return Ok(CmdResult::DisplayError(
                "This verb needs another panel".to_string()
            ));
        }
        match &verb.execution {
            VerbExecution::Internal(internal_exec) => {
                self.on_internal(w, internal_exec, invocation, trigger_type, cc)
            }
            VerbExecution::External(external) => {
                self.execute_external(w, verb, external, invocation, cc)
            }
            VerbExecution::Sequence(seq_ex) => {
                self.execute_sequence(w, verb, seq_ex, invocation, cc)
            }
        }
    }

    fn execute_external(
        &mut self,
        w: &mut W,
        verb: &Verb,
        external_execution: &ExternalExecution,
        invocation: Option<&VerbInvocation>,
        cc: &CmdContext,
    ) -> Result<CmdResult, ProgramError> {
        let selection = match self.selection() {
            Some(selection) => selection,
            None => {
                // this function is overriden for the state which doesn't have a
                // single selection (stage_state), so this error should not happen
                return Ok(CmdResult::error("no selection (EV)"));
            }
        };
        let exec_builder = ExecutionStringBuilder::from_invocation(
            &verb.invocation_parser,
            Some(selection),
            &cc.app.other_path,
            if let Some(inv) = invocation {
                &inv.args
            } else {
                &None
            },
        );
        external_execution.to_cmd_result(w, exec_builder, &cc.app.con)
    }

    fn execute_sequence(
        &mut self,
        w: &mut W,
        verb: &Verb,
        seq_ex: &SequenceExecution,
        invocation: Option<&VerbInvocation>,
        cc: &CmdContext,
    ) -> Result<CmdResult, ProgramError> {
        let selection = match self.selection() {
            Some(selection) => selection,
            None => {
                // this function is overriden for the state which doesn't have a
                // single selection (stage_state), so this error should not happen
                return Ok(CmdResult::DisplayError("no selection (EV)".to_string()));
            }
        };
        let exec_builder = || {
            ExecutionStringBuilder::from_invocation(
                &verb.invocation_parser,
                Some(selection),
                &cc.app.other_path,
                if let Some(inv) = invocation {
                    &inv.args
                } else {
                    &None
                },
            )
        };
        let sequence = Sequence {
            raw: exec_builder().shell_exec_string(&ExecPattern::from_string(&seq_ex.sequence.raw)),
            separator: seq_ex.sequence.separator.clone(),
        };
        Ok(CmdResult::ExecuteSequence { sequence })
    }

    /// change the state, does no rendering
    fn on_command(
        &mut self,
        w: &mut W,
        cc: &CmdContext,
    ) -> Result<CmdResult, ProgramError> {
        self.clear_pending();
        let con = &cc.app.con;
        let screen = cc.app.screen;
        match &cc.cmd {
            Command::Click(x, y) => self.on_click(*x, *y, screen, con),
            Command::DoubleClick(x, y) => self.on_double_click(*x, *y, screen, con),
            Command::PatternEdit { raw, expr } => {
                match InputPattern::new(raw.clone(), expr, con) {
                    Ok(pattern) => self.on_pattern(pattern, con),
                    Err(e) => Ok(CmdResult::DisplayError(format!("{}", e))),
                }
            }
            Command::VerbTrigger {
                index,
                input_invocation,
            } => self.execute_verb(
                w,
                &con.verb_store.verbs[*index],
                input_invocation.as_ref(),
                TriggerType::Other,
                cc,
            ),
            Command::Internal {
                internal,
                input_invocation,
            } => self.on_internal(
                w,
                &InternalExecution::from_internal(*internal),
                input_invocation.as_ref(),
                TriggerType::Other,
                cc,
            ),
            Command::VerbInvocate(invocation) => {
                let selection = self.selection();
                match con.verb_store.search(
                    &invocation.name,
                    self.selection().map(|s| s.stype),
                ) {
                    PrefixSearchResult::Match(_, verb) => {
                        self.execute_verb(
                            w,
                            verb,
                            Some(invocation),
                            TriggerType::Input,
                            cc,
                        )
                    }
                    _ => Ok(CmdResult::verb_not_found(&invocation.name)),
                }
            }
            Command::None | Command::VerbEdit(_) => {
                // we do nothing here, the real job is done in get_status
                Ok(CmdResult::Keep)
            }
        }
    }

    /// return a cmdresult asking for the opening of a preview
    fn open_preview(
        &mut self,
        prefered_mode: Option<PreviewMode>,
        close_if_open: bool,
        cc: &CmdContext,
    ) -> CmdResult {
        if let Some(id) = cc.app.preview {
            if close_if_open {
                CmdResult::ClosePanel {
                    validate_purpose: false,
                    panel_ref: PanelReference::Id(id),
                }
            } else {
                if prefered_mode.is_some() {
                    // we'll make the preview mode change be
                    // applied on the preview panel
                    CmdResult::ApplyOnPanel { id }
                } else {
                    CmdResult::Keep
                }
            }
        } else {
            if let Some(path) = self.selected_path() {
                if path.is_file() {
                    CmdResult::NewPanel {
                        state: Box::new(PreviewState::new(
                            path.to_path_buf(),
                            InputPattern::none(),
                            prefered_mode,
                            self.tree_options(),
                            &cc.app.con,
                        )),
                        purpose: PanelPurpose::Preview,
                        direction: HDir::Right,
                    }
                } else {
                    CmdResult::DisplayError(
                        "only regular files can be previewed".to_string()
                    )
                }
            } else {
                CmdResult::DisplayError(
                    "no selected file".to_string()
                )
            }
        }
    }

    fn selected_path(&self) -> Option<&Path>;

    fn selection(&self) -> Option<Selection<'_>>;

    fn sel_info<'c>(&'c self, app_state: &'c AppState) -> SelInfo<'c> {
        // overloaded in stage_state
        match self.selection() {
            None => SelInfo::None,
            Some(selection) => SelInfo::One(selection),
        }
    }

    fn has_at_least_one_selection(&self, cc: &CmdContext) -> bool {
        true // overloaded in stage_state
    }

    fn refresh(&mut self, screen: Screen, con: &AppContext) -> Command;

    fn tree_options(&self) -> TreeOptions;

    /// build a cmdResult in response to a command being a change of
    /// tree options. This may or not be a new state
    fn with_new_options(
        &mut self,
        screen: Screen,
        change_options: &dyn Fn(&mut TreeOptions),
        in_new_panel: bool,
        con: &AppContext,
    ) -> CmdResult;

    fn do_pending_task(
        &mut self,
        _screen: Screen,
        _con: &AppContext,
        _dam: &mut Dam,
    ) {
        // no pending task in default impl
        unreachable!();
    }

    fn get_pending_task(&self) -> Option<&'static str> {
        None
    }

    fn display(
        &mut self,
        w: &mut W,
        disc: &DisplayContext,
    ) -> Result<(), ProgramError>;

    /// return the flags to display
    fn get_flags(&self) -> Vec<Flag> {
        vec![]
    }

    fn get_starting_input(&self) -> String {
        String::new()
    }

    fn set_selected_path(&mut self, _path: PathBuf, _con: &AppContext) {
        // this function is useful for preview states
    }

    /// return the status which should be used when there's no verb edited
    fn no_verb_status(
        &self,
        _has_previous_state: bool,
        _con: &AppContext,
    ) -> Status {
        Status::from_message(
            "Hit *esc* to get back, or a space to start a verb"
        )
    }

    fn get_status(
        &self,
        cc: &CmdContext,
        has_previous_state: bool,
    ) -> Status {
        match &cc.cmd {
            Command::PatternEdit { .. } => self.no_verb_status(has_previous_state, cc.app.con),
            Command::VerbEdit(invocation) => {
                if invocation.name.is_empty() {
                    Status::new(
                        "Type a verb then *enter* to execute it (*?* for the list of verbs)",
                        false,
                    )
                } else {
                    match cc.app.con.verb_store.search(
                        &invocation.name,
                        self.selection().map(|s| s.stype), // FIXME is that always OK for stage ?
                    ) {
                        PrefixSearchResult::NoMatch => {
                            Status::new("No matching verb (*?* for the list of verbs)", true)
                        }
                        PrefixSearchResult::Match(_, verb) => {
                            self.get_verb_status(verb, invocation, cc)
                        }
                        PrefixSearchResult::Matches(completions) => Status::new(
                            format!(
                                "Possible verbs: {}",
                                completions
                                    .iter()
                                    .map(|c| format!("*{}*", c))
                                    .collect::<Vec<String>>()
                                    .join(", "),
                            ),
                            false,
                        ),
                    }
                }
            }
            _ => self.no_verb_status(has_previous_state, cc.app.con),
        }
    }

    // overloaded in stage_state
    fn get_verb_status(
        &self,
        verb: &Verb,
        invocation: &VerbInvocation,
        cc: &CmdContext,
    ) -> Status {
        let selection = self.selection();
        if let Some(err) = verb.check_args(&selection, invocation, &cc.app.other_path) {
            Status::new(err, true)
        } else {
            Status::new(
                verb.get_status_markdown(
                    selection,
                    &cc.app.other_path,
                    invocation,
                ),
                false,
            )
        }
    }
}


pub fn get_arg<T: Copy + FromStr>(
    verb_invocation: Option<&VerbInvocation>,
    internal_exec: &InternalExecution,
    default: T,
) -> T {
    verb_invocation
        .and_then(|vi| vi.args.as_ref())
        .or_else(|| internal_exec.arg.as_ref())
        .and_then(|s| s.parse::<T>().ok())
        .unwrap_or(default)
}

pub fn initial_mode(con: &AppContext) -> Mode {
    if con.modal {
        Mode::Command
    } else {
        Mode::Input
    }
}
