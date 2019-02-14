//! An application state dedicated to displaying a tree.
//! It's the first and main screen of broot.

use std::io::{self, Write};
use std::path::PathBuf;
use std::result::Result;
use std::time::Instant;
use termion::color;

use crate::app::{AppState, AppStateCmdResult};
use crate::app_context::AppContext;
use crate::commands::{Action, Command};
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
use crate::tree_views::TreeView;
use crate::verbs::{PrefixSearchResult, VerbExecutor};

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
        change_options: &Fn(&mut TreeOptions),
    ) -> AppStateCmdResult {
        let tree = match &self.filtered_tree {
            Some(tree) => &tree,
            None => &self.tree,
        };
        let mut options = tree.options.clone();
        change_options(&mut options);
        AppStateCmdResult::from_optional_state(BrowserState::new(
            tree.root().clone(),
            options,
            screen,
            &TaskLifetime::unlimited(),
        ))
    }
    fn page_height(screen: &Screen) -> i32 {
        i32::from(screen.h) - 2
    }
    pub fn displayed_tree(&self) -> &Tree {
        match &self.filtered_tree {
            Some(tree) => &tree,
            None => &self.tree,
        }
    }
}

impl AppState for BrowserState {
    fn apply(
        &mut self,
        cmd: &mut Command,
        screen: &Screen,
        con: &AppContext,
    ) -> io::Result<AppStateCmdResult> {
        self.pending_pattern = Pattern::None;
        let page_height = BrowserState::page_height(screen);
        Ok(match &cmd.action {
            Action::Back => {
                if self.filtered_tree.is_some() {
                    self.filtered_tree = None;
                    cmd.raw.clear();
                    AppStateCmdResult::Keep
                } else if self.tree.selection > 0 {
                    self.tree.selection = 0;
                    cmd.raw.clear();
                    AppStateCmdResult::Keep
                } else {
                    AppStateCmdResult::PopState
                }
            }
            Action::MoveSelection(dy) => {
                match self.filtered_tree {
                    Some(ref mut tree) => {
                        tree.move_selection(*dy, page_height);
                    }
                    None => {
                        self.tree.move_selection(*dy, page_height);
                    }
                };
                AppStateCmdResult::Keep
            }
            Action::ScrollPage(dp) => {
                if page_height < self.displayed_tree().lines.len() as i32 {
                    let dy = dp * page_height;
                    match self.filtered_tree {
                        Some(ref mut tree) => {
                            tree.try_scroll(dy, page_height);
                        }
                        None => {
                            self.tree.try_scroll(dy, page_height);
                        }
                    };
                }
                AppStateCmdResult::Keep
            }
            Action::OpenSelection => {
                let tree = match &self.filtered_tree {
                    Some(tree) => tree,
                    None => &self.tree,
                };
                if tree.selection == 0 {
                    AppStateCmdResult::Quit
                } else {
                    let line = tree.selected_line();
                    let tl = TaskLifetime::unlimited();
                    match &line.line_type {
                        LineType::File => {
                            AppStateCmdResult::Launch(Launchable::opener(&line.path)?)
                        }
                        LineType::Dir | LineType::SymLinkToDir(_) => {
                            AppStateCmdResult::from_optional_state(BrowserState::new(
                                line.target(),
                                tree.options.without_pattern(),
                                screen,
                                &tl,
                            ))
                        }
                        LineType::SymLinkToFile(target) => {
                            AppStateCmdResult::Launch(Launchable::opener(&PathBuf::from(target))?)
                        }
                        _ => {
                            unreachable!();
                        }
                    }
                }
            }
            Action::AltOpenSelection => {
                let tree = match &self.filtered_tree {
                    Some(tree) => tree,
                    None => &self.tree,
                };
                let line = tree.selected_line();
                let cd_idx = con.verb_store.index_of("cd");
                con.verb_store.verbs[cd_idx].to_cmd_result(&line.target(), con)?
            }
            Action::Verb(verb_key) => match con.verb_store.search(&verb_key) {
                PrefixSearchResult::Match(verb) => self.execute_verb(verb, screen, con)?,
                _ => AppStateCmdResult::verb_not_found(&verb_key),
            },
            Action::FuzzyPatternEdit(pat) => match pat.len() {
                0 => {
                    self.filtered_tree = None;
                    AppStateCmdResult::Keep
                }
                _ => {
                    self.pending_pattern = Pattern::fuzzy(pat);
                    AppStateCmdResult::Keep
                }
            },
            Action::RegexEdit(pat, flags) => {
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
            }
            Action::Help => AppStateCmdResult::NewState(Box::new(HelpState::new(screen))),
            Action::Next => {
                if let Some(ref mut tree) = self.filtered_tree {
                    tree.try_select_next_match();
                    tree.make_selection_visible(page_height);
                }
                AppStateCmdResult::Keep
            }
            _ => AppStateCmdResult::Keep,
        })
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
        screen.write_tree(&self.displayed_tree())
    }

    fn write_status(&self, screen: &mut Screen, cmd: &Command, con: &AppContext) -> io::Result<()> {
        match &cmd.action {
            Action::FuzzyPatternEdit(_) => {
                screen.write_status_text("Hit <enter> to select, <esc> to remove the filter")
            }
            Action::RegexEdit(_, _) => {
                screen.write_status_text("Hit <enter> to select, <esc> to remove the filter")
            }
            Action::VerbEdit(verb_key) => {
                match con.verb_store.search(&verb_key) {
                    PrefixSearchResult::NoMatch => {
                        screen.write_status_err("No matching verb (':?' for the list of verbs)")
                    }
                    PrefixSearchResult::Match(verb) => {
                        let line = match &self.filtered_tree {
                            Some(tree) => tree.selected_line(),
                            None => self.tree.selected_line(),
                        };
                        screen.write_status_text(
                            &format!(
                                "Hit <enter> to {} : {}",
                                &verb.name,
                                verb.description_for(line.target())
                            )
                            .to_string(),
                        )
                    }
                    PrefixSearchResult::TooManyMatches => screen.write_status_text(
                        // TODO show what verbs start with the currently edited verb key
                        "Type a verb then <enter> to execute it (':?' for the list of verbs)",
                    ),
                }
            }
            _ => {
                let tree = self.displayed_tree();
                if tree.selection == 0 {
                    screen.write_status_text(
                        "Hit <enter> to quit, '?' for help, or a few letters to search",
                    )
                } else {
                    let line = &tree.lines[tree.selection];
                    screen.write_status_text(if line.is_dir() {
                        "Hit <enter> to focus, or type a space then a verb"
                    } else {
                        "Hit <enter> to open the file, or type a space then a verb"
                    })
                }
            }
        }
    }

    fn write_flags(&self, screen: &mut Screen, _con: &AppContext) -> io::Result<()> {
        let tree = match &self.filtered_tree {
            Some(tree) => &tree,
            None => &self.tree,
        };
        let total_char_size = 9;
        write!(
            screen.stdout,
            "{}{}{}{} h:{}  gi:{}{}{}",
            termion::cursor::Goto(screen.w - total_char_size, screen.h),
            color::Bg(color::AnsiValue::grayscale(1)),
            termion::clear::UntilNewline,
            color::Fg(color::AnsiValue::grayscale(15)),
            if tree.options.show_hidden { 'y' } else { 'n' },
            match tree.options.respect_git_ignore {
                OptionBool::Auto => 'a',
                OptionBool::Yes => 'y',
                OptionBool::No => 'n',
            },
            color::Bg(color::Reset),
            color::Fg(color::Reset),
        )?;
        Ok(())
    }
}
