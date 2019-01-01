//! an application state dedicated to displaying a tree

use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Instant;
use termion::color;

use crate::app::{AppState, AppStateCmdResult};
use crate::app_context::AppContext;
use crate::commands::{Action, Command};
use crate::external::Launchable;
use crate::flat_tree::Tree;
use crate::help_states::HelpState;
use crate::patterns::Pattern;
use crate::screens::{self, Screen};
use crate::status::Status;
use crate::task_sync::TaskLifetime;
use crate::tree_build::TreeBuilder;
use crate::tree_options::{OptionBool, TreeOptions};
use crate::tree_views::TreeView;
use crate::verbs::VerbExecutor;

pub struct BrowserState {
    pub tree: Tree,
    pub filtered_tree: Option<Tree>,
    pending_pattern: Option<Pattern>, // a pattern which has not yet be applied
}

impl BrowserState {
    pub fn new(path: PathBuf, options: TreeOptions, tl: &TaskLifetime) -> Option<BrowserState> {
        let builder = TreeBuilder::from(path, options.clone());
        match builder.build(screens::max_tree_height() as usize, tl) {
            Some(tree) => Some(BrowserState {
                tree,
                filtered_tree: None,
                pending_pattern: None,
            }),
            None => None, // interrupted
        }
    }
    pub fn displayed_tree(&self) -> &Tree {
        match &self.filtered_tree {
            Some(tree) => &tree,
            None => &self.tree,
        }
    }
}

impl AppState for BrowserState {
    fn apply(&mut self, cmd: &mut Command, con: &AppContext) -> io::Result<AppStateCmdResult> {
        self.pending_pattern = None;
        let (_, page_height) = termion::terminal_size().unwrap();
        let mut page_height = page_height as i32;
        page_height -= 2;
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
                // this should not be computed here
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
                if tree.selected_line().is_dir() {
                    let tl = TaskLifetime::unlimited();
                    AppStateCmdResult::from_optional_state(BrowserState::new(
                        tree.selected_line().path.clone(),
                        tree.options.without_pattern(),
                        &tl,
                    ))
                } else {
                    AppStateCmdResult::Launch(Launchable::opener(&tree.selected_line().path)?)
                }
            }
            Action::Verb(verb_key) => match con.verb_store.get(&verb_key) {
                Some(verb) => self.execute_verb(verb, con)?,
                None => AppStateCmdResult::verb_not_found(&verb_key),
            },
            Action::PatternEdit(pat) => match pat.len() {
                0 => {
                    self.filtered_tree = None;
                    AppStateCmdResult::Keep
                }
                _ => {
                    self.pending_pattern = Some(Pattern::from(pat));
                    AppStateCmdResult::Keep
                }
            },
            Action::Help(about) => AppStateCmdResult::NewState(Box::new(HelpState::new(&about))),
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

    fn do_pending_task(&mut self, tl: &TaskLifetime) {
        if let Some(pat) = &mut self.pending_pattern {
            let start = Instant::now();
            let mut options = self.tree.options.clone();
            options.pattern = Some(pat.clone());
            let root = self.tree.root().clone();
            let len = self.tree.lines.len() as u16;
            let mut filtered_tree = TreeBuilder::from(root, options).build(len as usize, tl);
            if let Some(ref mut filtered_tree) = filtered_tree {
                info!("Tree search took {:?}", start.elapsed());
                filtered_tree.try_select_best_match();
                let (_, page_height) = termion::terminal_size().unwrap();
                let mut page_height = page_height as i32;
                page_height -= 2;
                filtered_tree.make_selection_visible(page_height);
            } // if none: task was cancelled from elsewhere
            self.filtered_tree = filtered_tree;
            self.pending_pattern = None;
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
            Action::PatternEdit(_) => {
                screen.write_status_text("Hit <enter> to select, <esc> to remove the filter")
            }
            Action::VerbEdit(verb_key) => match con.verb_store.get(&verb_key) {
                Some(verb) => screen.write_status_text(
                    &format!(
                        "Hit <enter> to {} : {}",
                        &verb.name,
                        verb.description_for(&self)
                    )
                    .to_string(),
                ),
                None => screen.write_status_text(
                    // TODO show what verbs start with the currently edited verb key
                    "Type a verb then <enter> to execute it (hit '?' for the list of verbs)",
                ),
            },
            _ => {
                let tree = self.displayed_tree();
                if tree.selection == 0 {
                    screen.write_status_text(
                        "Hit <enter> to quit, '?' for help, or type a few file's letters to navigate",
                    )
                } else {
                    let line = &tree.lines[tree.selection];
                    screen.write_status_text(match line.is_dir() {
                        true => "Hit <enter> to focus, or type a space then a verb",
                        false => "Hit <enter> to open the file, or type a space then a verb",
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
        debug!("respect_git_ignore: {:?}", tree.options.respect_git_ignore);
        write!(
            screen.stdout,
            "{}{}{}{} h:{}  gi:{}{}{}",
            termion::cursor::Goto(screen.w - total_char_size, screen.h),
            color::Bg(color::AnsiValue::grayscale(1)),
            termion::clear::UntilNewline,
            color::Fg(color::AnsiValue::grayscale(15)),
            match tree.options.show_hidden {
                true => 'y',
                false => 'n',
            },
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
