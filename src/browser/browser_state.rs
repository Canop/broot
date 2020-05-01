use {
    crate::{
        app::{AppContext, AppState, AppStateCmdResult, Status},
        command::Command,
        display::{DisplayableTree, Screen, FLAGS_AREA_WIDTH, W},
        errors::{ProgramError, TreeBuildError},
        flat_tree::{LineType, Tree},
        git,
        help::HelpState,
        launchable::Launchable,
        pattern::Pattern,
        path,
        print,
        selection_type::SelectionType,
        task_sync::Dam,
        tree_build::TreeBuilder,
        tree_options::TreeOptions,
        verb::{Internal, PrefixSearchResult, VerbInvocation, CD},
    },
    directories::UserDirs,
    open,
    std::{
        fs::OpenOptions,
        io::Write,
        path::{Path, PathBuf},
    },
    termimad::Area,
};

fn focus_path(
    path: PathBuf,
    screen: &mut Screen,
    tree: &Tree,
    in_new_panel: bool,
) -> AppStateCmdResult {
    AppStateCmdResult::from_optional_state(
        BrowserState::new(path, tree.options.clone(), screen, &Dam::unlimited()),
        in_new_panel,
    )
}

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
            LineType::File => match open::that(&line.path) {
                Ok(exit_status) => {
                    info!("open returned with exit_status {:?}", exit_status);
                    Ok(AppStateCmdResult::Keep)
                }
                Err(e) => Ok(AppStateCmdResult::DisplayError(format!("{:?}", e))),
            },
            LineType::Dir | LineType::SymLinkToDir(_) => {
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
            LineType::SymLinkToFile(target) => {
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
            LineType::File => make_opener(line.path.clone(), line.is_exe(), con),
            LineType::Dir | LineType::SymLinkToDir(_) => {
                Ok(if con.launch_args.cmd_export_path.is_some() {
                    CD.to_cmd_result(&line.target(), &None, con)?
                } else {
                    AppStateCmdResult::DisplayError(
                        "This feature needs broot to be launched with the `br` script".to_owned(),
                    )
                })
            }
            LineType::SymLinkToFile(target) => {
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
        internal: Internal,
        bang: bool,
        input_invocation: Option<&VerbInvocation>,
        screen: &mut Screen,
        con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError> {
        let page_height = BrowserState::page_height(screen);
        use Internal::*;
        Ok(match internal {
            back => {
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
            close_panel => AppStateCmdResult::PopPanel,
            complete => AppStateCmdResult::DisplayError("not yet implemented".to_string()),
            focus => {
                let tree = self.displayed_tree();
                let line = &tree.selected_line();
                let mut path = line.target();
                if !path.is_dir() {
                    path = path.parent().unwrap().to_path_buf();
                }
                focus_path(path, screen, tree, bang)
            }
            focus_root => focus_path(PathBuf::from("/"), screen, self.displayed_tree(), bang),
            up_tree => match self.displayed_tree().root().parent() {
                Some(path) => focus_path(path.to_path_buf(), screen, self.displayed_tree(), bang),
                None => AppStateCmdResult::DisplayError("no parent found".to_string()),
            },
            focus_user_home => match UserDirs::new() {
                Some(ud) => focus_path(
                    ud.home_dir().to_path_buf(),
                    screen,
                    self.displayed_tree(),
                    bang,
                ),
                None => AppStateCmdResult::DisplayError("no user home directory found".to_string()),
            },
            help => AppStateCmdResult::NewState {
                state: Box::new(HelpState::new(screen, con)),
                in_new_panel: bang,
            },
            open_panel => {
                if let Some(invocation) = input_invocation {
                    if let Some(arg) = &invocation.args {
                        if invocation.name == internal.name() {
                            // FIXME the name test is a hack, we should
                            // use the trigger type of the command
                            debug!("case A");
                            let tree = self.displayed_tree();
                            let base_dir = tree.selected_line().path.to_string_lossy();
                            let path = path::path_from(&arg, &base_dir);
                            let new_state = BrowserState::new(
                                PathBuf::from(&path),
                                tree.options.clone(),
                                screen,
                                &Dam::unlimited(),
                            )?.unwrap();
                            AppStateCmdResult::NewState {
                                state: Box::new(new_state),
                                in_new_panel: true,
                            }
                        } else {
                            // user would like to open for arg edition
                            // the current arg as a tree panel
                            debug!("case B");
                            AppStateCmdResult::DisplayError("not yet implemented".to_string())
                        }
                    } else {
                        // user wants to open for arg edition the selected
                        // tree line
                        debug!("case C");
                        AppStateCmdResult::DisplayError("not yet implemented".to_string())
                    }
                } else {
                    // just opening a new panel on the selected tree line
                    debug!("case D");
                    let tree = self.displayed_tree();
                    let line = &tree.selected_line();
                    let mut path = line.target();
                    if !path.is_dir() {
                        path = path.parent().unwrap().to_path_buf();
                    }
                    focus_path(path, screen, tree, true)
                }
            }
            open_stay => self.open_selection_stay_in_broot(screen, con, bang)?,
            open_leave => self.open_selection_quit_broot(con)?,
            line_down => {
                self.displayed_tree_mut().move_selection(1, page_height);
                AppStateCmdResult::Keep
            }
            line_up => {
                self.displayed_tree_mut().move_selection(-1, page_height);
                AppStateCmdResult::Keep
            }
            page_down => {
                let tree = self.displayed_tree_mut();
                if page_height < tree.lines.len() as i32 {
                    tree.try_scroll(page_height, page_height);
                }
                AppStateCmdResult::Keep
            }
            page_up => {
                let tree = self.displayed_tree_mut();
                if page_height < tree.lines.len() as i32 {
                    tree.try_scroll(-page_height, page_height);
                }
                AppStateCmdResult::Keep
            }
            parent => self.go_to_parent(screen, bang),
            print_path => print::print_path(&self.displayed_tree().selected_line().target(), con)?,
            print_relative_path => {
                print::print_relative_path(&self.displayed_tree().selected_line().target(), con)?
            }
            print_tree => print::print_tree(&self.displayed_tree(), screen, con)?,
            refresh => AppStateCmdResult::RefreshState { clear_cache: true },
            select_first => {
                self.displayed_tree_mut().try_select_first();
                AppStateCmdResult::Keep
            }
            select_last => {
                self.displayed_tree_mut().try_select_last();
                AppStateCmdResult::Keep
            }
            toggle_dates => self.with_new_options(screen, &|o| o.show_dates ^= true, bang),
            toggle_files => {
                self.with_new_options(screen, &|o: &mut TreeOptions| o.only_folders ^= true, bang)
            }
            toggle_hidden => self.with_new_options(screen, &|o| o.show_hidden ^= true, bang),
            toggle_git_ignore => {
                self.with_new_options(screen, &|o| o.respect_git_ignore ^= true, bang)
            }
            toggle_git_file_info => {
                self.with_new_options(screen, &|o| o.show_git_file_info ^= true, bang)
            }
            toggle_git_status => {
                self.with_new_options(screen, &|o| o.filter_by_git_status ^= true, bang)
            }
            toggle_perm => self.with_new_options(screen, &|o| o.show_permissions ^= true, bang),
            toggle_sizes => self.with_new_options(screen, &|o| o.show_sizes ^= true, bang),
            toggle_trim_root => self.with_new_options(screen, &|o| o.trim_root ^= true, bang),
            total_search => {
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
            quit => AppStateCmdResult::Quit,
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
        screen: &Screen,
        area: Area,
        _con: &AppContext,
    ) -> Result<(), ProgramError> {
        let dp = DisplayableTree {
            tree: &self.displayed_tree(),
            skin: &screen.skin,
            area,
            in_app: true,
        };
        dp.write_on(w)
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

    /// draw the flags at the bottom right of the screen
    fn write_flags(
        &self,
        w: &mut W,
        screen: &mut Screen,
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
        screen.skin.flag_label.queue_str(w, " h:")?;
        screen.skin.flag_value.queue(w, h_value)?;
        screen.skin.flag_label.queue_str(w, "   gi:")?;
        screen.skin.flag_value.queue(w, gi_value)?;
        Ok(())
    }
}
