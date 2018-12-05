//! an application state dedicated to displaying a tree

use std::io;
use std::path::PathBuf;
use std::time::Instant;

use app::{AppState, AppStateCmdResult};
use commands::{Action, Command};
use external::Launchable;
use flat_tree::Tree;
use help_states::HelpState;
use patterns::Pattern;
use screens::{self, Screen};
use status::Status;
use task_sync::TaskLifetime;
use tree_build::TreeBuilder;
use tree_options::TreeOptions;
use tree_views::TreeView;
use verbs::VerbStore;

pub struct BrowserState {
    pub tree: Tree,
    pub options: TreeOptions,
    pub filtered_tree: Option<Tree>,
}

impl BrowserState {
    pub fn new(
        path: PathBuf,
        options: TreeOptions,
        tl: TaskLifetime,
    ) -> Option<BrowserState> {
        match TreeBuilder::from(path, options.clone(), tl).build(screens::max_tree_height()) {
            Some(tree) => Some(BrowserState {
                tree,
                options,
                filtered_tree: None,
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
    fn apply(
        &mut self,
        cmd: &mut Command,
        verb_store: &VerbStore,
        tl: TaskLifetime,
    ) -> io::Result<AppStateCmdResult> {
        Ok(match &cmd.action {
            Action::Back => {
                if let Some(_) = self.filtered_tree {
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
            Action::FixPattern => {
                // stop pattern editing, either making it non existing, of fixing
                // the tree on the pattern (until back)
                cmd.raw = match self.filtered_tree {
                    Some(ref mut tree) => tree.key(),
                    None => self.tree.key(),
                };
                AppStateCmdResult::Keep
            }
            Action::MoveSelection(dy) => {
                match self.filtered_tree {
                    Some(ref mut tree) => {
                        tree.move_selection(*dy);
                        cmd.raw = tree.key();
                    }
                    None => {
                        self.tree.move_selection(*dy);
                        cmd.raw = self.tree.key();
                    }
                };
                AppStateCmdResult::Keep
            }
            Action::Select(key) => {
                match self.filtered_tree {
                    Some(ref mut tree) => {
                        if !tree.try_select(key) {
                            tree.selection = 0;
                        }
                    }
                    None => {
                        if !self.tree.try_select(key) {
                            self.tree.selection = 0;
                        }
                    }
                };
                AppStateCmdResult::Keep
            }
            Action::OpenSelection => {
                let line = match &self.filtered_tree {
                    Some(tree) => tree.selected_line(),
                    None => self.tree.selected_line(),
                };
                match line.is_dir() {
                    true => AppStateCmdResult::from_optional_state(BrowserState::new(
                        line.path.clone(),
                        self.options.clone(),
                        tl,
                    )),
                    false => AppStateCmdResult::Launch(Launchable::opener(&line.path)?),
                }
            }
            Action::Verb(verb_key) => match verb_store.get(&verb_key) {
                Some(verb) => verb.execute(&self)?,
                None => AppStateCmdResult::verb_not_found(&verb_key),
            },
            Action::Quit => AppStateCmdResult::Quit,
            Action::PatternEdit(pat) => {
                self.filtered_tree = match pat.len() {
                    0 => None,
                    _ => {
                        let start = Instant::now();
                        let pat = Pattern::from(pat);
                        let mut options = self.options.clone();
                        options.pattern = Some(pat.clone());
                        let root = self.tree.root().clone();
                        let len = self.tree.lines.len() as u16;
                        let mut filtered_tree = TreeBuilder::from(root, options, tl).build(len);
                        if let Some(ref mut filtered_tree) = filtered_tree {
                            debug!("Tree search took {:?}", start.elapsed());
                            filtered_tree.try_select_best_match(); // TODO make part of build ?
                        } // if none: task was cancelled from elsewhere
                        filtered_tree
                    }
                };
                AppStateCmdResult::Keep
            }
            Action::Help(about) => AppStateCmdResult::NewState(Box::new(HelpState::new(&about))),
            Action::Next => {
                if let Some(ref mut tree) = self.filtered_tree {
                    tree.try_select_next_match();
                }
                AppStateCmdResult::Keep
            }
            _ => AppStateCmdResult::Keep,
        })
    }

    fn display(&mut self, screen: &mut Screen, _verb_store: &VerbStore) -> io::Result<()> {
        screen.write_tree(&self.displayed_tree())
    }

    fn write_status(&self, screen: &mut Screen, cmd: &Command) -> io::Result<()> {
        match &cmd.action {
            Action::FixPattern => screen.write_status_text("Hit <esc> to remove the filter"),
            Action::PatternEdit(_) => {
                screen.write_status_text("Hit <enter> to freeze the fiter, <esc> to remove it")
            }
            _ => {
                let tree = self.displayed_tree();
                if tree.selection == 0 {
                    screen.write_status_text(
                        "Hit <enter> to quit, '?' for help, or type a file's key to navigate",
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
}
