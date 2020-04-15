use {
    crate::{
        app::{
            AppContext,
            AppState,
            AppStateCmdResult,
        },
        command::{Action, Command},
        display::{
            DisplayableTree,
            FLAGS_AREA_WIDTH,
            Screen,
            Status,
            W,
        },
        errors::{ProgramError, TreeBuildError},
        launchable::Launchable,
        flat_tree::{LineType, Tree},
        git,
        help::HelpState,
        pattern::Pattern,
        task_sync::Dam,
        tree_build::TreeBuilder,
        tree_options::TreeOptions,
        verb::{
            CD,
            PrefixSearchResult,
            VerbExecutor,
        },
    },
    minimad::Composite,
    open,
    std::{
        fs::OpenOptions,
        io::Write,
        path::PathBuf,
    },
    termimad::{
        Area,
    },
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
            BrowserState::new(
                tree.root().clone(),
                options,
                screen,
                &Dam::unlimited(),
            ),
            Command::from_pattern(&tree.options.pattern),
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
                    Command::new(),
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
                    CD.to_cmd_result(
                        &line.target(),
                        &None,
                        con,
                    )?
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

    pub fn go_to_parent(
        &mut self,
        screen: &mut Screen,
        in_new_panel: bool,
    ) -> AppStateCmdResult {
        match &self.displayed_tree().selected_line().path.parent() {
            Some(path) => AppStateCmdResult::from_optional_state(
                BrowserState::new(
                    path.to_path_buf(),
                    self.displayed_tree().options.without_pattern(),
                    screen,
                    &Dam::unlimited(),
                ),
                Command::new(),
                in_new_panel,
            ),
            None => AppStateCmdResult::DisplayError("no parent found".to_string()),
        }
    }

    fn normal_status_message(&self, has_pattern: bool) -> Composite<'static> {
        let tree = self.displayed_tree();
        if tree.selection == 0 {
            if has_pattern {
                mad_inline!("Hit *esc* to remove the filter, *enter* to go up, '?' for help")
            } else {
                mad_inline!("Hit *esc* to go back, *enter* to go up, *?* for help, or a few letters to search")
            }
        } else {
            let line = &tree.lines[tree.selection];
            if has_pattern {
                if line.is_dir() {
                    mad_inline!("Hit *enter* to focus, *alt*-*enter* to cd, *esc* to clear filter, or a space then a verb")
                } else {
                    mad_inline!("Hit *enter* to open, *alt*-*enter* to open and quit, *esc* to clear filter, or *:* + verb")
                }
            } else {
                if line.is_dir() {
                    mad_inline!("Hit *enter* to focus, *alt*-*enter* to cd, or a space then a verb")
                } else {
                    mad_inline!("Hit *enter* to open the file, *alt*-*enter* to open and quit, or a space then a verb")
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
    fn has_pending_task(&self) -> bool {
        self.pending_pattern.is_some()
            || self.displayed_tree().has_dir_missing_size()
            || self.displayed_tree().is_missing_git_status_computation()
    }

    fn write_status(
        &self,
        w: &mut W,
        cmd: &Command,
        screen: &Screen,
        con: &AppContext,
    ) -> Result<(), ProgramError> {
        let task = if self.pending_pattern.is_some() {
            Some("searching")
        } else if self.displayed_tree().has_dir_missing_size() {
            Some("computing sizes")
        } else {
            None
        };
        match &cmd.action {
            Action::FuzzyPatternEdit(s) if !s.is_empty() => {
                Status::new(task, self.normal_status_message(true), false).display(w, screen)
            }
            Action::RegexEdit(s, _) if !s.is_empty() => {
                Status::new(task, self.normal_status_message(true), false).display(w, screen)
            }
            Action::VerbEdit(invocation) => {
                if invocation.name.is_empty() {
                    Status::new(
                        task,
                        mad_inline!(
                            "Type a verb then *enter* to execute it (*?* for the list of verbs)"
                        ),
                        false,
                    )
                    .display(w, screen)
                } else {
                    match con.verb_store.search(&invocation.name) {
                        PrefixSearchResult::NoMatch => Status::new(
                            task,
                            mad_inline!("No matching verb (*?* for the list of verbs)"),
                            true,
                        )
                        .display(w, screen),
                        PrefixSearchResult::Match(verb) => {
                            let line = self.displayed_tree().selected_line();
                            verb.write_status(w, task, line.path.clone(), invocation, screen)
                        }
                        PrefixSearchResult::TooManyMatches(completions) => Status::new(
                            task,
                            Composite::from_inline(&format!(
                                "Possible verbs: {}",
                                completions
                                    .iter()
                                    .map(|c| format!("*{}*", c))
                                    .collect::<Vec<String>>()
                                    .join(", "),
                            )),
                            false,
                        )
                        .display(w, screen),
                    }
                }
            }
            _ => {
                Status::new(task, self.normal_status_message(false), false)
                    .display(w, screen)
            }
        }
    }

    fn can_execute(&self, verb_index: usize, con: &AppContext) -> bool {
        self.displayed_tree()
            .selected_line()
            .is_of(con.verb_store.verbs[verb_index].selection_condition)
    }

    fn apply(
        &mut self,
        cmd: &mut Command,
        screen: &mut Screen,
        con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError> {
        self.pending_pattern = Pattern::None;
        let page_height = BrowserState::page_height(screen);
        match &cmd.action {
            Action::Back => {
                if self.filtered_tree.is_some() {
                    self.filtered_tree = None;
                    cmd.raw.clear();
                    Ok(AppStateCmdResult::Keep)
                } else if self.tree.selection > 0 {
                    self.tree.selection = 0;
                    cmd.raw.clear();
                    Ok(AppStateCmdResult::Keep)
                } else {
                    Ok(AppStateCmdResult::PopState)
                }
            }
            Action::MoveSelection(dy) => {
                self.displayed_tree_mut().move_selection(*dy, page_height);
                Ok(AppStateCmdResult::Keep)
            }
            Action::Click(_, y) => {
                let y = *y as i32;
                self.displayed_tree_mut().try_select_y(y);
                Ok(AppStateCmdResult::Keep)
            }
            Action::DoubleClick(_, y) => {
                if self.displayed_tree().selection == *y as usize {
                    self.open_selection_stay_in_broot(screen, con, false)
                } else {
                    // A double click always come after a simple click at
                    // same position. If it's not the selected line, it means
                    // the click wasn't on a selectable/openable tree line
                    Ok(AppStateCmdResult::Keep)
                }
            }
            Action::OpenSelection => self.open_selection_stay_in_broot(screen, con, false),
            Action::AltOpenSelection => self.open_selection_quit_broot(con),
            Action::FuzzyPatternEdit(pat) => {
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
            Action::Help => Ok(AppStateCmdResult::NewState {
                state: Box::new(HelpState::new(screen, con)),
                cmd: Command::new(),
                in_new_panel: false,
            }),
            Action::Next => {
                self.displayed_tree_mut().try_select_next_match();
                self.displayed_tree_mut().make_selection_visible(page_height);
                Ok(AppStateCmdResult::Keep)
            }
            Action::Previous => {
                if let Some(tree) = &mut self.filtered_tree {
                    tree.try_select_previous_match();
                    tree.make_selection_visible(page_height);
                }
                Ok(AppStateCmdResult::Keep)
            }
            Action::RegexEdit(pat, flags) => Ok(match Pattern::regex(pat, flags) {
                Ok(regex_pattern) => {
                    self.pending_pattern = regex_pattern;
                    AppStateCmdResult::Keep
                }
                Err(e) => {
                    // FIXME details
                    AppStateCmdResult::DisplayError(format!("{}", e))
                }
            }),
            Action::Resize(w, h) => {
                screen.set_terminal_size(*w, *h, con);
                Ok(AppStateCmdResult::RefreshState { clear_cache: false })
            }
            Action::VerbIndex(index) => {
                let verb = &con.verb_store.verbs[*index];
                self.execute_verb(verb, None, screen, con)
            }
            Action::VerbInvocate(invocation) => match con.verb_store.search(&invocation.name) {
                PrefixSearchResult::Match(verb) => {
                    self.execute_verb(verb, Some(&invocation), screen, con)
                }
                _ => Ok(AppStateCmdResult::verb_not_found(&invocation.name)),
            },
            _ => Ok(AppStateCmdResult::Keep),
        }
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
        debug!("drawing tree in {:?}", &area);
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
        screen.goto_clear(
            w,
            screen.width - total_char_size - 1,
            screen.height - 1,
        )?;
        let h_value = if tree.options.show_hidden { 'y' } else { 'n' };
        let gi_value = if tree.options.respect_git_ignore { 'y' } else { 'n' };
        screen.skin.flag_label.queue_str(w, " h:")?;
        screen.skin.flag_value.queue(w, h_value)?;
        screen.skin.flag_label.queue_str(w, "   gi:")?;
        screen.skin.flag_value.queue(w, gi_value)?;
        Ok(())
    }
}
