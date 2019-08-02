use std::io::{self, Write};
use std::path::PathBuf;
use std::result::Result;
use std::time::Instant;
use std::fs::OpenOptions;

use opener;
use crossterm_terminal::{ClearType, Terminal};

use crate::app::{AppState, AppStateCmdResult};
use crate::app_context::AppContext;
use crate::commands::{Action, Command};
use crate::displayable_tree::DisplayableTree;
use crate::errors::TreeBuildError;
use crate::external::Launchable;
use crate::flat_tree::{LineType, Tree};
use crate::help_states::HelpState;
use crate::patterns::Pattern;
use crate::screens::Screen;
use crate::status::Status;
use crate::task_sync::TaskLifetime;
use crate::tree_build::TreeBuilder;
use crate::tree_options::{OptionBool, TreeOptions};
use crate::verb_store::PrefixSearchResult;
use crate::verbs::VerbExecutor;

/// An application state dedicated to displaying a tree.
/// It's the first and main screen of broot.
pub struct BrowserState {
    pub tree: Tree,
    pub filtered_tree: Option<Tree>,
    pending_pattern: Pattern, // a pattern (or not) which has not yet be applied
}

impl BrowserState {
    pub fn new(
        path: PathBuf,
        mut options: TreeOptions,
        screen: &Screen,
        tl: &TaskLifetime,
    ) -> Result<Option<BrowserState>, TreeBuildError> {
        let pending_pattern = options.pattern;
        options.pattern = Pattern::None;
        let builder = TreeBuilder::from(path, options, BrowserState::page_height(screen) as usize)?;
        Ok(match builder.build(tl) {
            Some(tree) => Some(BrowserState {
                tree,
                filtered_tree: None,
                pending_pattern,
            }),
            None => None, // interrupted
        })
    }

    pub fn with_new_options(
        &self,
        screen: &Screen,
        change_options: &dyn Fn(&mut TreeOptions),
    ) -> AppStateCmdResult {
        let tree = self.displayed_tree();
        let mut options = tree.options.clone();
        change_options(&mut options);
        AppStateCmdResult::from_optional_state(
            BrowserState::new(
                tree.root().clone(),
                options,
                screen,
                &TaskLifetime::unlimited(),
            ),
            tree.options.pattern.to_command(),
        )
    }

    fn page_height(screen: &Screen) -> i32 {
        i32::from(screen.h) - 2
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

    fn open_selection_stay_in_broot(
        &mut self,
        screen: &mut Screen,
        _con: &AppContext,
    ) -> io::Result<AppStateCmdResult> {
        let tree = self.displayed_tree();
        let line = tree.selected_line();
        let tl = TaskLifetime::unlimited();
        match &line.line_type {
            LineType::File => {
                opener::open(&line.path).unwrap();
                Ok(AppStateCmdResult::Keep)
            }
            LineType::Dir | LineType::SymLinkToDir(_) => {
                let mut target = line.target();
                if tree.selection == 0 {
                    // opening the root would be going to where we already are.
                    // We go up one level instead
                    if let Some(parent) = target.parent() {
                        target = PathBuf::from(parent);
                    }
                }
                Ok(AppStateCmdResult::from_optional_state(
                    BrowserState::new(
                        target,
                        tree.options.without_pattern(),
                        screen,
                        &tl,
                    ),
                    Command::new(),
                ))
            }
            LineType::SymLinkToFile(target) => {
                let path = PathBuf::from(target);
                opener::open(&path).unwrap();
                Ok(AppStateCmdResult::Keep)
            }
            _ => {
                unreachable!();
            }
        }
    }

    fn open_selection_quit_broot(
        &mut self,
        screen: &mut Screen,
        con: &AppContext,
    ) -> io::Result<AppStateCmdResult> {
        let tree = self.displayed_tree();
        let line = tree.selected_line();
        match &line.line_type {
            LineType::File => {
                make_opener(line.path.clone(), line.is_exe(), con)
            }
            LineType::Dir | LineType::SymLinkToDir(_) => {
                if con.launch_args.cmd_export_path.is_some() {
                    let cd_idx = con.verb_store.index_of("cd");
                    con.verb_store.verbs[cd_idx].to_cmd_result(&line.target(), &None, screen, con)
                } else {
                    Ok(AppStateCmdResult::DisplayError(
                        "This feature needs broot to be launched with the `br` script".to_owned()
                    ))
                }
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

    fn write_status_normal(&self, screen: &mut Screen, has_pattern: bool) -> io::Result<()> {
        let tree = self.displayed_tree();
        screen.write_status_text(
            if tree.selection == 0 {
                if has_pattern {
                    "Hit <esc> to remove the filter, <enter> to go up, '?' for help"
                } else {
                    "Hit <esc> to go back, <enter> to go up, '?' for help, or a few letters to search"
                }
            } else {
                let line = &tree.lines[tree.selection];
                if has_pattern {
                    if line.is_dir() {
                        "Hit <enter> to focus, <alt><enter> to cd, <esc> to remove the filter, or a space then a verb"
                    } else {
                        "Hit <enter> to open, <alt><enter> to open and quit, <esc> to clear the filter, or a space then a verb"
                    }
                } else {
                    if line.is_dir() {
                        "Hit <enter> to focus, <alt><enter> to cd, or a space then a verb"
                    } else {
                        "Hit <enter> to open the file, <alt><enter> to open and quit, or type a space then a verb"
                    }
                }
            }
        )
    }

}

/// build a AppStateCmdResult with a launchable which will be used to
///  1/ quit broot
///  2/ open the relevant file the best possible way
fn make_opener(path: PathBuf, is_exe: bool, con: &AppContext) -> io::Result<AppStateCmdResult> {
    Ok(
        if is_exe {
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
        }
    )
}

impl AppState for BrowserState {
    fn apply(
        &mut self,
        cmd: &mut Command,
        screen: &mut Screen,
        con: &AppContext,
    ) -> io::Result<AppStateCmdResult> {
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
            Action::ScrollPage(dp) => {
                let tree = self.displayed_tree_mut();
                if page_height < tree.lines.len() as i32 {
                    let dy = dp * page_height;
                    tree.try_scroll(dy, page_height);
                }
                Ok(AppStateCmdResult::Keep)
            }
            Action::Click(_, y) => {
                let y = *y as i32 - 1; // click position starts at (1, 1)
                self.displayed_tree_mut().try_select_y(y);
                Ok(AppStateCmdResult::Keep)
            }
            Action::DoubleClick(_, y) => {
                if self.displayed_tree().selection + 1 == *y as usize {
                    self.open_selection_stay_in_broot(screen, con)
                } else {
                    // A double click always come after a simple click at
                    // same position. If it's not the selected line, it means
                    // the click wasn't on a selectable/openable tree line
                    Ok(AppStateCmdResult::Keep)
                }
            }
            Action::OpenSelection => self.open_selection_stay_in_broot(screen, con),
            Action::AltOpenSelection => self.open_selection_quit_broot(screen, con),
            Action::Verb(invocation) => match con.verb_store.search(&invocation.key) {
                PrefixSearchResult::Match(verb) => {
                    self.execute_verb(verb, &invocation, screen, con)
                }
                _ => Ok(AppStateCmdResult::verb_not_found(&invocation.key)),
            },
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
            Action::RegexEdit(pat, flags) => Ok(
                match Pattern::regex(pat, flags) {
                    Ok(regex_pattern) => {
                        self.pending_pattern = regex_pattern;
                        AppStateCmdResult::Keep
                    }
                    Err(e) => {
                        // FIXME details
                        AppStateCmdResult::DisplayError(format!("{}", e))
                    }
                }
            ),
            Action::Help => Ok(
                AppStateCmdResult::NewState(Box::new(HelpState::new(screen, con)), Command::new())
            ),
            Action::Refresh => Ok(AppStateCmdResult::RefreshState),
            Action::Quit => Ok(AppStateCmdResult::Quit),
            Action::Next => {
                if let Some(tree) = &mut self.filtered_tree {
                    tree.try_select_next_match();
                    tree.make_selection_visible(page_height);
                }
                Ok(AppStateCmdResult::Keep)
            },
            Action::Previous => {
                if let Some(tree) = &mut self.filtered_tree {
                    tree.try_select_previous_match();
                    tree.make_selection_visible(page_height);
                }
                Ok(AppStateCmdResult::Keep)
            }
            _ => Ok(AppStateCmdResult::Keep),
        }
    }

    fn has_pending_tasks(&self) -> bool {
        if self.pending_pattern.is_some() {
            return true;
        }
        if self.displayed_tree().has_dir_missing_size() {
            return true;
        }
        false
    }

    /// do some work, totally or partially, if there's some to do.
    /// Stop as soon as the lifetime is expired.
    fn do_pending_task(&mut self, screen: &mut Screen, tl: &TaskLifetime) {
        if self.pending_pattern.is_some() {
            let start = Instant::now();
            let mut options = self.tree.options.clone();
            options.pattern = self.pending_pattern.take();
            let root = self.tree.root().clone();
            let len = self.tree.lines.len() as u16;
            let mut filtered_tree = match TreeBuilder::from(root, options, len as usize) {
                Ok(builder) => builder.build(tl),
                Err(e) => {
                    let _ = screen.write_status_err(&e.to_string());
                    warn!("Error while building tree: {:?}", e);
                    return;
                }
            };
            if let Some(ref mut filtered_tree) = filtered_tree {
                info!(
                    "Tree search with pattern {} took {:?}",
                    &filtered_tree.options.pattern,
                    start.elapsed()
                );
                filtered_tree.try_select_best_match();
                filtered_tree.make_selection_visible(BrowserState::page_height(screen));
            } // if none: task was cancelled from elsewhere
            self.filtered_tree = filtered_tree;
            return;
        }
        if let Some(ref mut tree) = self.filtered_tree {
            tree.fetch_some_missing_dir_size(tl);
        } else {
            self.tree.fetch_some_missing_dir_size(tl);
        }
    }

    fn display(&mut self, screen: &mut Screen, _con: &AppContext) -> io::Result<()> {
        let dp = DisplayableTree {
            tree: &self.displayed_tree(),
            skin: &screen.skin,
            area: termimad::Area {
                left: 0,
                top: 0,
                width: screen.w,
                height: screen.h - 2,
            },
            in_app: true,
        };
        screen.goto(1, 1);
        print!("{}", dp);
        Ok(())
    }


    fn write_status(&self, screen: &mut Screen, cmd: &Command, con: &AppContext) -> io::Result<()> {
        match &cmd.action {
            Action::FuzzyPatternEdit(s) if s.len() > 0 => self.write_status_normal(screen, true),
            Action::RegexEdit(s, _) if s.len() > 0 => self.write_status_normal(screen, true),
            Action::VerbEdit(invocation) => {
                match con.verb_store.search(&invocation.key) {
                    PrefixSearchResult::NoMatch => {
                        screen.write_status_err("No matching verb ('?' for the list of verbs)")
                    }
                    PrefixSearchResult::Match(verb) => {
                        if let Some(err) = verb.match_error(invocation) {
                            screen.write_status_err(&err)
                        } else {
                            let line = self.displayed_tree().selected_line();
                            screen.write_status_text(
                                &format!(
                                    "Hit <enter> to {} : {}",
                                    &verb.invocation.key,
                                    verb.description_for(line.path.clone(), &invocation.args)
                                )
                                .to_string(),
                            )
                        }
                    }
                    PrefixSearchResult::TooManyMatches => screen.write_status_text(
                        "Type a verb then <enter> to execute it ('?' for the list of verbs)",
                    ),
                }
            }
            _ => self.write_status_normal(screen, false),
        }
    }

    fn refresh(&mut self, screen: &Screen, _con: &AppContext) -> Command {
        let page_height = BrowserState::page_height(screen) as usize;
        // refresh the base tree
        if let Err(e) = self.tree.refresh(page_height) {
            warn!("refreshing base tree failed : {:?}", e);
        }
        // refresh the filtered tree, if any
        if let Some(ref mut tree) = self.filtered_tree {
            if let Err(e) = tree.refresh(page_height) {
                warn!("refreshing filtered tree failed : {:?}", e);
            }
            tree.options.pattern.to_command()
        } else {
            self.tree.options.pattern.to_command()
        }
    }

    fn write_flags(&self, screen: &mut Screen, _con: &AppContext) -> io::Result<()> {
        let tree = self.displayed_tree();
        let total_char_size = 9;
        screen.goto(screen.w - total_char_size, screen.h);
        let terminal = Terminal::new();
        terminal.clear(ClearType::UntilNewLine)?;
        let h_value = if tree.options.show_hidden { 'y' } else { 'n' };
        let gi_value = match tree.options.respect_git_ignore {
            OptionBool::Auto => 'a',
            OptionBool::Yes => 'y',
            OptionBool::No => 'n',
        };
        print!(
            "{}{}  {}{}",
            screen.skin.flag_label.apply_to(" h:"),
            screen.skin.flag_value.apply_to(h_value),
            screen.skin.flag_label.apply_to(" gi:"),
            screen.skin.flag_value.apply_to(gi_value),
        );
        Ok(())
    }
}
