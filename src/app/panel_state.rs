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
        stage::*,
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

    fn get_type(&self) -> PanelStateType;

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
        _app_state: &AppState,
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
            CmdResult::error("modal mode not enabled in configuration")
        }
    }

    /// execute the internal with the optional given invocation.
    ///
    /// The invocation comes from the input and may be related
    /// to a different verb (the verb may have been triggered
    /// by a key shortcut)
    fn on_internal(
        &mut self,
        w: &mut W,
        internal_exec: &InternalExecution,
        input_invocation: Option<&VerbInvocation>,
        trigger_type: TriggerType,
        app_state: &mut AppState,
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
        app_state: &mut AppState,
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
                    CmdResult::error("Clipboard feature not enabled at compilation")
                }
                #[cfg(feature = "clipboard")]
                {
                    if let Some(path) = self.selected_path() {
                        let path = path.to_string_lossy().to_string();
                        match terminal_clipboard::set_string(path) {
                            Ok(()) => CmdResult::Keep,
                            Err(_) => CmdResult::error("Clipboard error while copying path"),
                        }
                    } else {
                        CmdResult::error("Nothing to copy")
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
                        if bang && cc.app.preview_panel.is_none() {
                            CmdResult::NewPanel {
                                state: Box::new(state),
                                purpose: PanelPurpose::None,
                                direction: HDir::Right,
                            }
                        } else {
                            CmdResult::new_state(Box::new(state))
                        }
                    }
                    Err(e) => CmdResult::DisplayError(format!("{e}")),
                }
            }
            Internal::help => {
                let bang = input_invocation
                    .map(|inv| inv.bang)
                    .unwrap_or(internal_exec.bang);
                if bang && cc.app.preview_panel.is_none() {
                    CmdResult::NewPanel {
                        state: Box::new(HelpState::new(self.tree_options(), screen, con)),
                        purpose: PanelPurpose::None,
                        direction: HDir::Right,
                    }
                } else {
                    CmdResult::new_state(Box::new(
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
                    CmdResult::error("no selection to open")
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
                        "*not sorting anymore*"
                    } else {
                        o.sort = Sort::Count;
                        o.show_counts = true;
                        "*now sorting by file count*"
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
                        "*not sorting anymore*"
                    } else {
                        o.sort = Sort::Date;
                        o.show_dates = true;
                        "*now sorting by last modified date*"
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
                        "*not sorting anymore*"
                    } else {
                        o.sort = Sort::Size;
                        o.show_sizes = true;
                        o.show_root_fs = true;
                        "*now sorting files and directories by total size*"
                    }
                },
                bang,
                con,
            ),
            Internal::sort_by_type => self.with_new_options(
                screen,
                &|o| {
                    match o.sort {
                        Sort::TypeDirsFirst => {
                           o.sort = Sort::TypeDirsLast;
                           "*sorting by type, directories last*"
                        }
                        Sort::TypeDirsLast => {
                            o.sort = Sort::None;
                            "*not sorting anymore*"
                        }
                        _ => {
                            o.sort = Sort::TypeDirsFirst;
                           "*sorting by type, directories first*"
                        }
                    }
                },
                bang,
                con,
            ),
            Internal::sort_by_type_dirs_first => self.with_new_options(
                screen,
                &|o| {
                    if o.sort == Sort::TypeDirsFirst {
                        o.sort = Sort::None;
                        "*not sorting anymore*"
                    } else {
                        o.sort = Sort::TypeDirsFirst;
                        "*now sorting by type, directories first*"
                    }
                },
                bang,
                con,
            ),
            Internal::sort_by_type_dirs_last => self.with_new_options(
                screen,
                &|o| {
                    if o.sort == Sort::TypeDirsLast {
                        o.sort = Sort::None;
                        "*not sorting anymore*"
                    } else {
                        o.sort = Sort::TypeDirsLast;
                        "*now sorting by type, directories last*"
                    }
                },
                bang,
                con,
            ),
            Internal::no_sort => self.with_new_options(
                screen,
                &|o| {
                    if o.sort == Sort::None {
                        "*still not searching*"
                    } else {
                        o.sort = Sort::None;
                        "*not sorting anymore*"
                    }
                },
                bang,
                con,
            ),
            Internal::toggle_counts => {
                self.with_new_options(
					screen,
					&|o| {
						o.show_counts ^= true;
                        if o.show_counts {
                            "*displaying file counts*"
                        } else {
                            "*hiding file counts*"
                        }
					},
					bang,
					con,
				)
            }
            Internal::toggle_tree => {
                self.with_new_options(
                    screen,
                    &|o| {
                        o.show_tree ^= true;
                        if o.show_tree {
                            "*displaying tree structure (if possible)*"
                        } else {
                            "*displaying only current directory*"
                        }
                    },
                    bang,
                    con,
                )
            }
            Internal::toggle_dates => {
                self.with_new_options(
					screen,
					&|o| {
						o.show_dates ^= true;
                        if o.show_dates {
                            "*displaying last modified dates*"
                        } else {
                            "*hiding last modified dates*"
                        }
					},
					bang,
					con,
				)
            }
            Internal::toggle_device_id => {
                self.with_new_options(
					screen,
					&|o| {
						o.show_device_id ^= true;
                        if o.show_device_id {
                            "*displaying device id*"
                        } else {
                            "*hiding device id*"
                        }
					},
					bang,
					con,
				)
            }
            Internal::toggle_files => {
                self.with_new_options(
					screen,
					&|o| {
                        o.only_folders ^= true;
                        if o.only_folders {
                            "*displaying only directories*"
                        } else {
                            "*displaying both files and directories*"
                        }
                    },
					bang,
					con,
				)
            }
            Internal::toggle_hidden => {
                self.with_new_options(
					screen,
					&|o| {
						o.show_hidden ^= true;
                        if o.show_hidden {
                            "h:**y** - *Hidden files displayed*"
                        } else {
                            "h:**n** - *Hidden files not displayed*"
                        }
					},
					bang,
					con,
				)
            }
            Internal::toggle_root_fs => {
                self.with_new_options(
					screen,
					&|o| {
						o.show_root_fs ^= true;
                        if o.show_root_fs {
                            "*displaying filesystem info for the tree's root directory*"
                        } else {
                            "*removing filesystem info*"
                        }
					},
					bang,
					con,
				)
            }
            Internal::toggle_git_ignore => {
                self.with_new_options(
					screen,
					&|o| {
						o.respect_git_ignore ^= true;
                        if o.respect_git_ignore {
                            "gi:**y** - *applying gitignore rules*"
                        } else {
                            "gi:**n** - *not applying gitignore rules*"
                        }
					},
					bang,
					con,
				)
            }
            Internal::toggle_git_file_info => {
                self.with_new_options(
					screen,
					&|o| {
						o.show_git_file_info ^= true;
                        if o.show_git_file_info {
                            "*displaying git info next to files*"
                        } else {
                            "*removing git file info*"
                        }
					},
					bang,
					con,
				)
            }
            Internal::toggle_git_status => {
                self.with_new_options(
                    screen, &|o| {
                        if o.filter_by_git_status {
                            o.filter_by_git_status = false;
                            "*not filtering according to git status anymore*"
                        } else {
                            o.filter_by_git_status = true;
                            o.show_hidden = true;
                            "*only displaying new or modified files*"
                        }
                    }, bang, con
                )
            }
            Internal::toggle_perm => {
                self.with_new_options(
					screen,
					&|o| {
						o.show_permissions ^= true;
                        if o.show_permissions {
                            "*displaying file permissions*"
                        } else {
                            "*removing file permissions*"
                        }
					},
					bang,
					con,
				)
            }
            Internal::toggle_sizes => self.with_new_options(
                screen,
                &|o| {
                    if o.show_sizes {
                        o.show_sizes = false;
                        o.show_root_fs = false;
                        "*removing sizes of files and directories*"
                    } else {
                        o.show_sizes = true;
                        o.show_root_fs = true;
                        "*now displaying sizes of files and directories*"
                    }
                },
                bang,
                con,
            ),
            Internal::toggle_trim_root => {
                self.with_new_options(
					screen,
					&|o| {
						o.trim_root ^= true;
                        if o.trim_root {
                            "*now trimming root from excess files*"
                        } else {
                            "*not trimming root files anymore*"
                        }
					},
					bang,
					con,
				)
            }
            Internal::close_preview => {
                if let Some(id) = cc.app.preview_panel {
                    CmdResult::ClosePanel {
                        validate_purpose: false,
                        panel_ref: PanelReference::Id(id),
                    }
                } else {
                    CmdResult::Keep
                }
            }
            Internal::escape => {
                CmdResult::HandleInApp(Internal::escape)
            }
            Internal::panel_left | Internal::panel_left_no_open => {
                CmdResult::HandleInApp(Internal::panel_left_no_open)
            }
            Internal::panel_right | Internal::panel_right_no_open => {
                CmdResult::HandleInApp(Internal::panel_right_no_open)
            }
            Internal::toggle_second_tree => {
                CmdResult::HandleInApp(Internal::toggle_second_tree)
            }
            Internal::clear_stage => {
                app_state.stage.clear();
                if let Some(panel_id) = cc.app.stage_panel {
                    CmdResult::ClosePanel {
                        validate_purpose: false,
                        panel_ref: PanelReference::Id(panel_id),
                    }
                } else {
                    CmdResult::Keep
                }
            }
            Internal::stage => self.stage(app_state, cc, con),
            Internal::unstage => self.unstage(app_state, cc, con),
            Internal::toggle_stage => self.toggle_stage(app_state, cc, con),
            Internal::close_staging_area => {
                if let Some(id) = cc.app.stage_panel {
                    CmdResult::ClosePanel {
                        validate_purpose: false,
                        panel_ref: PanelReference::Id(id),
                    }
                } else {
                    CmdResult::Keep
                }
            }
            Internal::open_staging_area => {
                if cc.app.stage_panel.is_none() {
                    CmdResult::NewPanel {
                        state: Box::new(StageState::new(app_state, self.tree_options(), con)),
                        purpose: PanelPurpose::None,
                        direction: HDir::Right,
                    }
                } else {
                    CmdResult::Keep
                }
            }
            Internal::toggle_staging_area => {
                if let Some(id) = cc.app.stage_panel {
                    CmdResult::ClosePanel {
                        validate_purpose: false,
                        panel_ref: PanelReference::Id(id),
                    }
                } else {
                    CmdResult::NewPanel {
                        state: Box::new(StageState::new(app_state, self.tree_options(), con)),
                        purpose: PanelPurpose::None,
                        direction: HDir::Right,
                    }
                }
            }
            Internal::set_syntax_theme => CmdResult::HandleInApp(Internal::set_syntax_theme),
            Internal::print_path => print::print_paths(self.sel_info(app_state), con)?,
            Internal::print_relative_path => print::print_relative_paths(self.sel_info(app_state), con)?,
            Internal::refresh => CmdResult::RefreshState { clear_cache: true },
            Internal::quit => CmdResult::Quit,
            _ => CmdResult::Keep,
        })
    }

    fn stage(
        &self,
        app_state: &mut AppState,
        cc: &CmdContext,
        con: &AppContext,
    ) -> CmdResult {
        if let Some(path) = self.selected_path() {
            let path = path.to_path_buf();
            app_state.stage.add(path);
            if cc.app.stage_panel.is_none() {
                return CmdResult::NewPanel {
                    state: Box::new(StageState::new(app_state, self.tree_options(), con)),
                    purpose: PanelPurpose::None,
                    direction: HDir::Right,
                };
            }
        } else {
            // TODO display error ?
            warn!("no path in state");
        }
        CmdResult::Keep
    }

    fn unstage(
        &self,
        app_state: &mut AppState,
        cc: &CmdContext,
        _con: &AppContext,
    ) -> CmdResult {
        if let Some(path) = self.selected_path() {
            if app_state.stage.remove(path) && app_state.stage.is_empty() {
                if let Some(panel_id) = cc.app.stage_panel {
                    return CmdResult::ClosePanel {
                        validate_purpose: false,
                        panel_ref: PanelReference::Id(panel_id),
                    };
                }
            }
        }
        CmdResult::Keep
    }

    fn toggle_stage(
        &self,
        app_state: &mut AppState,
        cc: &CmdContext,
        con: &AppContext,
    ) -> CmdResult {
        if let Some(path) = self.selected_path() {
            if app_state.stage.contains(path) {
                self.unstage(app_state, cc, con)
            } else {
                self.stage(app_state, cc, con)
            }
        } else {
            CmdResult::error("no selection")
        }
    }

    fn execute_verb(
        &mut self,
        w: &mut W, // needed because we may want to switch from alternate in some externals
        verb: &Verb,
        invocation: Option<&VerbInvocation>,
        trigger_type: TriggerType,
        app_state: &mut AppState,
        cc: &CmdContext,
    ) -> Result<CmdResult, ProgramError> {
        if verb.needs_selection && !self.has_at_least_one_selection(app_state) {
            return Ok(CmdResult::error("This verb needs a selection"));
        }
        if verb.needs_another_panel && app_state.other_panel_path.is_none() {
            return Ok(CmdResult::error("This verb needs another panel"));
        }
        let res = match &verb.execution {
            VerbExecution::Internal(internal_exec) => {
                self.on_internal(
                    w,
                    internal_exec,
                    invocation,
                    trigger_type,
                    app_state,
                    cc,
                )
            }
            VerbExecution::External(external) => {
                self.execute_external(w, verb, external, invocation, app_state, cc)
            }
            VerbExecution::Sequence(seq_ex) => {
                self.execute_sequence(w, verb, seq_ex, invocation, app_state, cc)
            }
        };
        if res.is_ok() {
            // if the stage has been emptied by the operation (eg a "rm"), we
            // close it
            app_state.stage.refresh();
            if app_state.stage.is_empty() {
                if let Some(id) = cc.app.stage_panel {
                    return Ok(CmdResult::ClosePanel {
                        validate_purpose: false,
                        panel_ref: PanelReference::Id(id),
                    });
                }
            }
        }
        res
    }

    fn execute_external(
        &mut self,
        w: &mut W,
        verb: &Verb,
        external_execution: &ExternalExecution,
        invocation: Option<&VerbInvocation>,
        app_state: &mut AppState,
        cc: &CmdContext,
    ) -> Result<CmdResult, ProgramError> {
        let sel_info = self.sel_info(app_state);
        if let Some(invocation) = &invocation {
            if let Some(error) = verb.check_args(sel_info, invocation, &app_state.other_panel_path) {
                debug!("verb.check_args prevented execution: {:?}", &error);
                return Ok(CmdResult::error(error));
            }
        }
        let exec_builder = ExecutionStringBuilder::with_invocation(
            &verb.invocation_parser,
            sel_info,
            app_state,
            if let Some(inv) = invocation {
                inv.args.as_ref()
            } else {
                None
            },
        );
        external_execution.to_cmd_result(w, exec_builder, cc.app.con)
    }

    fn execute_sequence(
        &mut self,
        _w: &mut W,
        verb: &Verb,
        seq_ex: &SequenceExecution,
        invocation: Option<&VerbInvocation>,
        app_state: &mut AppState,
        _cc: &CmdContext,
    ) -> Result<CmdResult, ProgramError> {
        let sel_info = self.sel_info(app_state);
        if matches!(sel_info, SelInfo::More(_)) {
            // sequences would be hard to execute as the execution on a file can change the
            // state in too many ways (changing selection, focused panel, parent, unstage or
            // stage files, removing the staged paths, etc.)
            return Ok(CmdResult::error("sequences can't be executed on multiple selections"));
        }
        let exec_builder = ExecutionStringBuilder::with_invocation(
            &verb.invocation_parser,
            sel_info,
            app_state,
            if let Some(inv) = invocation {
                inv.args.as_ref()
            } else {
                None
            },
        );
        // TODO what follows is dangerous: if an inserted group value contains the separator,
        // the parsing will cut on this separator
        let sequence = Sequence {
            raw: exec_builder.shell_exec_string(&ExecPattern::from_string(&seq_ex.sequence.raw)),
            separator: seq_ex.sequence.separator.clone(),
        };
        Ok(CmdResult::ExecuteSequence { sequence })
    }

    /// change the state, does no rendering
    fn on_command(
        &mut self,
        w: &mut W,
        app_state: &mut AppState,
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
                    Ok(pattern) => self.on_pattern(pattern, app_state, con),
                    Err(e) => Ok(CmdResult::DisplayError(format!("{e}"))),
                }
            }
            Command::VerbTrigger {
                verb_id,
                input_invocation,
            } => self.execute_verb(
                w,
                con.verb_store.verb(*verb_id),
                input_invocation.as_ref(),
                TriggerType::Other,
                app_state,
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
                app_state,
                cc,
            ),
            Command::VerbInvocate(invocation) => {
                let sel_info = self.sel_info(app_state);
                match con.verb_store.search_sel_info(
                    &invocation.name,
                    sel_info,
                ) {
                    PrefixSearchResult::Match(_, verb) => {
                        self.execute_verb(
                            w,
                            verb,
                            Some(invocation),
                            TriggerType::Input(verb),
                            app_state,
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
        preferred_mode: Option<PreviewMode>,
        close_if_open: bool,
        cc: &CmdContext,
    ) -> CmdResult {
        if let Some(id) = cc.app.preview_panel {
            if close_if_open {
                CmdResult::ClosePanel {
                    validate_purpose: false,
                    panel_ref: PanelReference::Id(id),
                }
            } else if preferred_mode.is_some() {
                // we'll make the preview mode change be
                // applied on the preview panel
                CmdResult::ApplyOnPanel { id }
            } else {
                CmdResult::Keep
            }
        } else if let Some(path) = self.selected_path() {
            CmdResult::NewPanel {
                state: Box::new(PreviewState::new(
                    path.to_path_buf(),
                    InputPattern::none(),
                    preferred_mode,
                    self.tree_options(),
                    cc.app.con,
                )),
                purpose: PanelPurpose::Preview,
                direction: HDir::Right,
            }
        } else {
            CmdResult::error("no selected file")
        }
    }

    /// must return None if the state doesn't display a file tree
    fn tree_root(&self) -> Option<&Path> {
        None
    }

    fn selected_path(&self) -> Option<&Path>;

    fn selection(&self) -> Option<Selection<'_>>;

    fn sel_info<'c>(&'c self, _app_state: &'c AppState) -> SelInfo<'c> {
        // overloaded in stage_state
        match self.selection() {
            None => SelInfo::None,
            Some(selection) => SelInfo::One(selection),
        }
    }

    fn has_at_least_one_selection(&self, _app_state: &AppState) -> bool {
        true // overloaded in stage_state
    }

    fn refresh(&mut self, screen: Screen, con: &AppContext) -> Command;

    fn tree_options(&self) -> TreeOptions;

    /// Build a cmdResult in response to a command being a change of
    /// tree options. This may or not be a new state.
    ///
    /// The provided `change_options` function returns a status message
    /// explaining the change
    fn with_new_options(
        &mut self,
        screen: Screen,
        change_options: &dyn Fn(&mut TreeOptions) -> &'static str,
        in_new_panel: bool,
        con: &AppContext,
    ) -> CmdResult;

    fn do_pending_task(
        &mut self,
        _app_state: &mut AppState,
        _screen: Screen,
        _con: &AppContext,
        _dam: &mut Dam,
    ) -> Result<(), ProgramError> {
        // no pending task in default impl
        unreachable!();
    }

    fn get_pending_task(
        &self,
    ) -> Option<&'static str> {
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
        _width: usize, // available width
    ) -> Status {
        Status::from_message(
            "Hit *esc* to get back, or a space to start a verb"
        )
    }

    fn get_status(
        &self,
        app_state: &AppState,
        cc: &CmdContext,
        has_previous_state: bool,
        width: usize,
    ) -> Status {
        info!("get_status cc.cmd={:?}", &cc.cmd);
        match &cc.cmd {
            Command::PatternEdit { .. } => self.no_verb_status(has_previous_state, cc.app.con, width),
            Command::VerbEdit(invocation) | Command::VerbTrigger { input_invocation: Some(invocation), .. } => {
                if invocation.name.is_empty() {
                    Status::new(
                        "Type a verb then *enter* to execute it (*?* for the list of verbs)",
                        false,
                    )
                } else {
                    let sel_info = self.sel_info(app_state);
                    match cc.app.con.verb_store.search_sel_info(
                        &invocation.name,
                        sel_info,
                    ) {
                        PrefixSearchResult::NoMatch => {
                            Status::new("No matching verb (*?* for the list of verbs)", true)
                        }
                        PrefixSearchResult::Match(_, verb) => {
                            self.get_verb_status(verb, invocation, sel_info, cc, app_state)
                        }
                        PrefixSearchResult::Matches(completions) => Status::new(
                            format!(
                                "Possible verbs: {}",
                                completions
                                    .iter()
                                    .map(|c| format!("*{c}*"))
                                    .collect::<Vec<String>>()
                                    .join(", "),
                            ),
                            false,
                        ),
                    }
                }
            }
            _ => self.no_verb_status(has_previous_state, cc.app.con, width),
        }
    }

    fn get_verb_status(
        &self,
        verb: &Verb,
        invocation: &VerbInvocation,
        sel_info: SelInfo<'_>,
        _cc: &CmdContext,
        app_state: &AppState,
    ) -> Status {
        if sel_info.count_paths() > 1 {
            if let VerbExecution::External(external) = &verb.execution {
                if external.exec_mode != ExternalExecutionMode::StayInBroot {
                    return Status::new(
                        "only verbs returning to broot on end can be executed on a multi-selection".to_owned(),
                        true,
                    );
                }
            }
            // right now there's no check for sequences but they're inherently dangereous
        }
        if let Some(err) = verb.check_args(sel_info, invocation, &app_state.other_panel_path) {
            Status::new(err, true)
        } else {
            Status::new(
                verb.get_status_markdown(
                    sel_info,
                    app_state,
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
        .or(internal_exec.arg.as_ref())
        .and_then(|s| s.parse::<T>().ok())
        .unwrap_or(default)
}

