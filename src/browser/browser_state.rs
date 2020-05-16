use {
    crate::{
        app::*,
        command::{Command, TriggerType},
        display::{DisplayableTree, Screen, FLAGS_AREA_WIDTH, W},
        errors::{ProgramError, TreeBuildError},
        git,
        help::HelpState,
        launchable::Launchable,
        pattern::Pattern,
        path,
        print,
        selection_type::SelectionType,
        skin::PanelSkin,
        task_sync::Dam,
        tree::*,
        tree_build::TreeBuilder,
        verb::*,
    },
    open,
    std::{
        fs::OpenOptions,
        io::Write,
        path::{Path, PathBuf},
    },
    termimad::Area,
};

/// An application state dedicated to displaying a tree.
/// It's the first and main screen of broot.
pub struct BrowserState {
    pub tree: Tree,
    pub filtered_tree: Option<Tree>,
    pub pending_pattern: Pattern, // a pattern (or not) which has not yet be applied
    pub total_search_required: bool, // whether the pending pattern should be in total search mode
}

impl BrowserState {

    /// build a new tree state if there's no error and there's no cancellation.
    ///
    /// In case of cancellation return `Ok(None)`. If dam is `unlimited()` then
    /// this can't be returned.
    pub fn new(
        path: PathBuf,
        mut options: TreeOptions,
        screen: &Screen,
        dam: &Dam,
    ) -> Result<Option<BrowserState>, TreeBuildError> {
        let pending_pattern = options.pattern.take();
        let builder = TreeBuilder::from(path, options, BrowserState::page_height(screen) as usize)?;
        Ok(builder.build(false, dam).map(move |tree| BrowserState {
            tree,
            filtered_tree: None,
            pending_pattern,
            total_search_required: false,
        }))
    }

    pub fn with_new_options(
        &self,
        screen: &Screen,
        change_options: &dyn Fn(&mut TreeOptions),
        in_new_panel: bool,
    ) -> AppStateCmdResult {
        let tree = self.displayed_tree();
        let mut options = tree.options.clone();
        change_options(&mut options);
        AppStateCmdResult::from_optional_state(
            BrowserState::new(tree.root().clone(), options, screen, &Dam::unlimited()),
            in_new_panel,
        )
    }

    pub fn root(&self) -> &Path {
        self.tree.root()
    }

    pub fn page_height(screen: &Screen) -> i32 {
        i32::from(screen.height) - 2
    }

    /// return a reference to the currently displayed tree, which
    /// is the filtered tree if there's one, the base tree if not.
    pub fn displayed_tree(&self) -> &Tree {
        self.filtered_tree.as_ref().unwrap_or(&self.tree)
    }

    /// return a mutable reference to the currently displayed tree, which
    /// is the filtered tree if there's one, the base tree if not.
    pub fn displayed_tree_mut(&mut self) -> &mut Tree {
        self.filtered_tree.as_mut().unwrap_or(&mut self.tree)
    }

    pub fn open_selection_stay_in_broot(
        &mut self,
        screen: &mut Screen,
        _con: &AppContext,
        in_new_panel: bool,
    ) -> Result<AppStateCmdResult, ProgramError> {
        let tree = self.displayed_tree();
        let line = tree.selected_line();
        match &line.line_type {
            TreeLineType::File => match open::that(&line.path) {
                Ok(exit_status) => {
                    info!("open returned with exit_status {:?}", exit_status);
                    Ok(AppStateCmdResult::Keep)
                }
                Err(e) => Ok(AppStateCmdResult::DisplayError(format!("{:?}", e))),
            },
            TreeLineType::Dir | TreeLineType::SymLinkToDir(_) => {
                let mut target = line.target();
                if tree.selection == 0 {
                    // opening the root would be going to where we already are.
                    // We go up one level instead
                    if let Some(parent) = target.parent() {
                        target = PathBuf::from(parent);
                    }
                }
                let dam = Dam::unlimited();
                Ok(AppStateCmdResult::from_optional_state(
                    BrowserState::new(target, tree.options.without_pattern(), screen, &dam),
                    in_new_panel,
                ))
            }
            TreeLineType::SymLinkToFile(target) => {
                let path = PathBuf::from(target);
                open::that(&path)?;
                Ok(AppStateCmdResult::Keep)
            }
            _ => {
                unreachable!();
            }
        }
    }

    pub fn open_selection_quit_broot(
        &mut self,
        con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError> {
        let tree = self.displayed_tree();
        let line = tree.selected_line();
        match &line.line_type {
            TreeLineType::File => make_opener(line.path.clone(), line.is_exe(), con),
            TreeLineType::Dir | TreeLineType::SymLinkToDir(_) => {
                Ok(if con.launch_args.cmd_export_path.is_some() {
                    CD.to_cmd_result(&line.target(), &None, con)?
                } else {
                    AppStateCmdResult::DisplayError(
                        "This feature needs broot to be launched with the `br` script".to_owned(),
                    )
                })
            }
            TreeLineType::SymLinkToFile(target) => {
                make_opener(
                    PathBuf::from(target),
                    line.is_exe(), // today this always return false
                    con,
                )
            }
            _ => {
                unreachable!();
            }
        }
    }

    pub fn go_to_parent(&mut self, screen: &mut Screen, in_new_panel: bool) -> AppStateCmdResult {
        match &self.displayed_tree().selected_line().path.parent() {
            Some(path) => AppStateCmdResult::from_optional_state(
                BrowserState::new(
                    path.to_path_buf(),
                    self.displayed_tree().options.without_pattern(),
                    screen,
                    &Dam::unlimited(),
                ),
                in_new_panel,
            ),
            None => AppStateCmdResult::DisplayError("no parent found".to_string()),
        }
    }

    fn normal_status_message(&self, has_pattern: bool) -> &'static str {
        let tree = self.displayed_tree();
        if tree.selection == 0 {
            if has_pattern {
                "Hit *esc* to remove the filter, *enter* to go up, '?' for help"
            } else {
                "Hit *esc* to go back, *enter* to go up, *?* for help, or a few letters to search"
            }
        } else {
            let line = &tree.lines[tree.selection];
            if has_pattern {
                if line.is_dir() {
                    "Hit *enter* to focus, *alt*-*enter* to cd, *esc* to clear filter, or a space then a verb"
                } else {
                    "Hit *enter* to open, *alt*-*enter* to open and quit, *esc* to clear filter, or *:* + verb"
                }
            } else {
                if line.is_dir() {
                    "Hit *enter* to focus, *alt*-*enter* to cd, or a space then a verb"
                } else {
                    "Hit *enter* to open the file, *alt*-*enter* to open and quit, or a space then a verb"
                }
            }
        }
    }

    /// draw the flags at the bottom right of the screen
    /// TODO call this method
    fn write_flags(
        &self,
        w: &mut W,
        screen: &mut Screen,
        panel_skin: &PanelSkin,
        _con: &AppContext,
    ) -> Result<(), ProgramError> {
        let tree = self.displayed_tree();
        let total_char_size = FLAGS_AREA_WIDTH;
        screen.goto_clear(w, screen.width - total_char_size - 1, screen.height - 1)?;
        let h_value = if tree.options.show_hidden { 'y' } else { 'n' };
        let gi_value = if tree.options.respect_git_ignore {
            'y'
        } else {
            'n'
        };
        panel_skin.styles.flag_label.queue_str(w, " h:")?;
        panel_skin.styles.flag_value.queue(w, h_value)?;
        panel_skin.styles.flag_label.queue_str(w, "   gi:")?;
        panel_skin.styles.flag_value.queue(w, gi_value)?;
        Ok(())
    }
}

/// build a AppStateCmdResult with a launchable which will be used to
///  1/ quit broot
///  2/ open the relevant file the best possible way
fn make_opener(
    path: PathBuf,
    is_exe: bool,
    con: &AppContext,
) -> Result<AppStateCmdResult, ProgramError> {
    Ok(if is_exe {
        let path = path.to_string_lossy().to_string();
        if let Some(export_path) = &con.launch_args.cmd_export_path {
            // broot was launched as br, we can launch the executable from the shell
            let f = OpenOptions::new().append(true).open(export_path)?;
            writeln!(&f, "{}", path)?;
            AppStateCmdResult::Quit
        } else {
            AppStateCmdResult::from(Launchable::program(vec![path])?)
        }
    } else {
        AppStateCmdResult::from(Launchable::opener(path))
    })
}

impl AppState for BrowserState {
    fn get_pending_task(&self) -> Option<&'static str> {
        if self.pending_pattern.is_some() {
            Some("searching")
        } else if self.displayed_tree().has_dir_missing_size() {
            Some("computing sizes")
        } else if self.displayed_tree().is_missing_git_status_computation() {
            Some("computing git status")
        } else {
            None
        }
    }

    fn get_status(&self, cmd: &Command, con: &AppContext) -> Status {
        match cmd {
            Command::FuzzyPatternEdit(s) if !s.is_empty() => {
                Status::new(self.normal_status_message(true), false)
            }
            Command::RegexEdit(s, _) if !s.is_empty() => {
                Status::new(self.normal_status_message(true), false)
            }
            Command::VerbEdit(invocation) => {
                if invocation.name.is_empty() {
                    Status::new(
                        "Type a verb then *enter* to execute it (*?* for the list of verbs)",
                        false,
                    )
                } else {
                    match con.verb_store.search(&invocation.name) {
                        PrefixSearchResult::NoMatch => {
                            Status::new("No matching verb (*?* for the list of verbs)", true)
                        }
                        PrefixSearchResult::Match(verb) => {
                            let line = self.displayed_tree().selected_line();
                            verb.get_status(&line.path, invocation)
                        }
                        PrefixSearchResult::TooManyMatches(completions) => Status::new(
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
            _ => Status::new(self.normal_status_message(false), false),
        }
    }

    fn selected_path(&self) -> &Path {
        &self.displayed_tree().selected_line().path
    }

    fn selection_type(&self) -> SelectionType {
        self.displayed_tree().selected_line().selection_type()
    }

    fn clear_pending(&mut self) {
        self.pending_pattern = Pattern::None;
    }

    fn on_click(
        &mut self,
        _x: u16,
        y: u16,
        _screen: &mut Screen,
        _con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError> {
        self.displayed_tree_mut().try_select_y(y as i32);
        Ok(AppStateCmdResult::Keep)
    }

    fn on_double_click(
        &mut self,
        _x: u16,
        y: u16,
        screen: &mut Screen,
        con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError> {
        if self.displayed_tree().selection == y as usize {
            self.open_selection_stay_in_broot(screen, con, false)
        } else {
            // A double click always come after a simple click at
            // same position. If it's not the selected line, it means
            // the click wasn't on a selectable/openable tree line
            Ok(AppStateCmdResult::Keep)
        }
    }

    fn on_fuzzy_pattern_edit(
        &mut self,
        pat: &str,
        _con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError> {
        match pat.len() {
            0 => {
                self.filtered_tree = None;
            }
            _ => {
                self.pending_pattern = Pattern::fuzzy(pat);
            }
        }
        Ok(AppStateCmdResult::Keep)
    }

    fn on_regex_pattern_edit(
        &mut self,
        pat: &str,
        flags: &str,
        _con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError> {
        Ok(match Pattern::regex(pat, flags) {
            Ok(regex_pattern) => {
                self.pending_pattern = regex_pattern;
                AppStateCmdResult::Keep
            }
            Err(e) => {
                // FIXME details
                AppStateCmdResult::DisplayError(format!("{}", e))
            }
        })
    }

    fn on_internal(
        &mut self,
        internal_exec: &InternalExecution,
        input_invocation: Option<&VerbInvocation>,
        trigger_type: TriggerType,
        screen: &mut Screen,
        panel_skin: &PanelSkin,
        con: &AppContext,
        panel_purpose: PanelPurpose,
    ) -> Result<AppStateCmdResult, ProgramError> {
        let page_height = BrowserState::page_height(screen);
        debug!("internal_exec: {:?}", internal_exec);
        let bang = input_invocation
            .map(|inv| inv.bang)
            .unwrap_or(internal_exec.bang);
        debug!("bang: {:?}", bang);
        Ok(match internal_exec.internal {
            Internal::back => {
                if self.filtered_tree.is_some() {
                    self.filtered_tree = None;
                    AppStateCmdResult::Keep
                } else if self.tree.selection > 0 {
                    self.tree.selection = 0;
                    AppStateCmdResult::Keep
                } else {
                    AppStateCmdResult::PopState
                }
            }
            Internal::close_panel_ok => AppStateCmdResult::ClosePanel {
                validate_purpose: true,
            },
            Internal::close_panel_cancel => AppStateCmdResult::ClosePanel {
                validate_purpose: false,
            },
            Internal::complete => {
                AppStateCmdResult::DisplayError("not yet implemented".to_string())
            }
            Internal::focus => internal_focus::on_internal(
                internal_exec,
                input_invocation,
                trigger_type,
                self.selected_path(),
                screen,
                con,
                self.displayed_tree().options.clone(),
            ),
            Internal::up_tree => match self.displayed_tree().root().parent() {
                Some(path) => internal_focus::on_path(
                    path.to_path_buf(),
                    screen,
                    self.displayed_tree().options.clone(),
                    bang,
                ),
                None => AppStateCmdResult::DisplayError("no parent found".to_string()),
            },
            Internal::help => {
                if bang {
                    AppStateCmdResult::NewPanel {
                        state: Box::new(HelpState::new(screen, con)),
                        purpose: PanelPurpose::None,
                    }
                } else {
                    AppStateCmdResult::NewState(Box::new(HelpState::new(screen, con)))
                }
            }
            Internal::open_stay => self.open_selection_stay_in_broot(screen, con, bang)?,
            Internal::open_leave => self.open_selection_quit_broot(con)?,
            Internal::line_down => {
                self.displayed_tree_mut().move_selection(1, page_height);
                AppStateCmdResult::Keep
            }
            Internal::line_up => {
                self.displayed_tree_mut().move_selection(-1, page_height);
                AppStateCmdResult::Keep
            }
            Internal::page_down => {
                let tree = self.displayed_tree_mut();
                if page_height < tree.lines.len() as i32 {
                    tree.try_scroll(page_height, page_height);
                }
                AppStateCmdResult::Keep
            }
            Internal::page_up => {
                let tree = self.displayed_tree_mut();
                if page_height < tree.lines.len() as i32 {
                    tree.try_scroll(-page_height, page_height);
                }
                AppStateCmdResult::Keep
            }
            Internal::parent => self.go_to_parent(screen, bang),
            Internal::print_path => {
                let path = &self.displayed_tree().selected_line().target();
                print::print_path(path, con)?
            }
            Internal::print_relative_path => {
                let path = &self.displayed_tree().selected_line().target();
                print::print_relative_path(path, con)?
            }
            Internal::print_tree => {
                print::print_tree(&self.displayed_tree(), screen, panel_skin, con)?
            }
            Internal::refresh => AppStateCmdResult::RefreshState { clear_cache: true },
            Internal::select_first => {
                self.displayed_tree_mut().try_select_first();
                AppStateCmdResult::Keep
            }
            Internal::select_last => {
                self.displayed_tree_mut().try_select_last();
                AppStateCmdResult::Keep
            }
            Internal::start_end_panel => {
                if panel_purpose.is_arg_edition() {
                    debug!("start_end understood as end");
                    AppStateCmdResult::ClosePanel {
                        validate_purpose: true,
                    }
                } else {
                    debug!("start_end understood as start");
                    let tree_options = self.displayed_tree().options.clone();
                    if let Some(input_invocation) = input_invocation {
                        // we'll go for input arg editing
                        let path = if let Some(input_arg) = &input_invocation.args {
                            let path = self.root().to_string_lossy();
                            let path = path::path_from(&path, input_arg);
                            PathBuf::from(path)
                        } else {
                            self.root().to_path_buf()
                        };
                        let arg_type = SelectionType::Any; // We might do better later
                        let purpose = PanelPurpose::ArgEdition { arg_type };
                        internal_focus::new_panel_on_path(path, screen, tree_options, purpose)
                    } else {
                        // we just open a new panel on the selected path,
                        // without purpose
                        internal_focus::new_panel_on_path(
                            self.selected_path().to_path_buf(),
                            screen,
                            tree_options,
                            PanelPurpose::None,
                        )
                    }
                }
            }
            Internal::toggle_dates => {
                self.with_new_options(screen, &|o| o.show_dates ^= true, bang)
            }
            Internal::toggle_files => {
                self.with_new_options(screen, &|o: &mut TreeOptions| o.only_folders ^= true, bang)
            }
            Internal::toggle_hidden => {
                self.with_new_options(screen, &|o| o.show_hidden ^= true, bang)
            }
            Internal::toggle_git_ignore => {
                self.with_new_options(screen, &|o| o.respect_git_ignore ^= true, bang)
            }
            Internal::toggle_git_file_info => {
                self.with_new_options(screen, &|o| o.show_git_file_info ^= true, bang)
            }
            Internal::toggle_git_status => {
                self.with_new_options(screen, &|o| o.filter_by_git_status ^= true, bang)
            }
            Internal::toggle_perm => {
                self.with_new_options(screen, &|o| o.show_permissions ^= true, bang)
            }
            Internal::toggle_sizes => {
                self.with_new_options(screen, &|o| o.show_sizes ^= true, bang)
            }
            Internal::toggle_trim_root => {
                self.with_new_options(screen, &|o| o.trim_root ^= true, bang)
            }
            Internal::total_search => {
                if let Some(tree) = &self.filtered_tree {
                    if tree.total_search {
                        AppStateCmdResult::DisplayError(
                            "search was already total - all children have been rated".to_owned(),
                        )
                    } else {
                        self.pending_pattern = tree.options.pattern.clone();
                        self.total_search_required = true;
                        AppStateCmdResult::Keep
                    }
                } else {
                    AppStateCmdResult::DisplayError(
                        "this verb can be used only after a search".to_owned(),
                    )
                }
            }
            Internal::quit => AppStateCmdResult::Quit,
        })
    }

    /// do some work, totally or partially, if there's some to do.
    /// Stop as soon as the dam asks for interruption
    fn do_pending_task(&mut self, screen: &mut Screen, dam: &mut Dam) {
        if self.pending_pattern.is_some() {
            let pattern_str = self.pending_pattern.to_string();
            let mut options = self.tree.options.clone();
            options.pattern = self.pending_pattern.take();
            let root = self.tree.root().clone();
            let len = self.tree.lines.len() as u16;
            let builder = match TreeBuilder::from(root, options, len as usize) {
                Ok(builder) => builder,
                Err(e) => {
                    warn!("Error while preparing tree builder: {:?}", e);
                    return;
                }
            };
            let mut filtered_tree = time!(
                Info,
                "tree filtering",
                pattern_str,
                builder.build(self.total_search_required, dam),
            ); // can be None if a cancellation was required
            self.total_search_required = false;
            if let Some(ref mut ft) = filtered_tree {
                ft.try_select_best_match();
                ft.make_selection_visible(BrowserState::page_height(screen));
                self.filtered_tree = filtered_tree;
            }
        } else if self.displayed_tree().is_missing_git_status_computation() {
            let root_path = self.displayed_tree().root();
            let git_status = git::get_tree_status(root_path, dam);
            self.displayed_tree_mut().git_status = git_status;
        } else {
            self.displayed_tree_mut().fetch_some_missing_dir_size(dam);
        }
    }

    fn display(
        &mut self,
        w: &mut W,
        _screen: &Screen,
        area: Area,
        panel_skin: &PanelSkin,
        _con: &AppContext,
    ) -> Result<(), ProgramError> {
        let dp = DisplayableTree {
            tree: &self.displayed_tree(),
            skin: &panel_skin.styles,
            area,
            in_app: true,
        };
        dp.write_on(w)
        // TODO display flags here if panel active
    }

    fn refresh(&mut self, screen: &Screen, _con: &AppContext) -> Command {
        let page_height = BrowserState::page_height(screen) as usize;
        // refresh the base tree
        if let Err(e) = self.tree.refresh(page_height) {
            warn!("refreshing base tree failed : {:?}", e);
        }
        // refresh the filtered tree, if any
        Command::from_pattern(match self.filtered_tree {
            Some(ref mut tree) => {
                if let Err(e) = tree.refresh(page_height) {
                    warn!("refreshing filtered tree failed : {:?}", e);
                }
                &tree.options.pattern
            }
            None => &self.tree.options.pattern,
        })
    }
}
