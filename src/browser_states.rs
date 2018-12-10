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
    pub fn new(path: PathBuf, options: TreeOptions, tl: TaskLifetime) -> Option<BrowserState> {
        match TreeBuilder::from(path, options.clone(), tl)
            .build(screens::max_tree_height() as usize)
        {
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
    ) -> io::Result<AppStateCmdResult> {
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
                        tree.move_selection(*dy);
                    }
                    None => {
                        self.tree.move_selection(*dy);
                    }
                };
                AppStateCmdResult::Keep
            }
            Action::OpenSelection => {
                let line = match &self.filtered_tree {
                    Some(tree) => tree.selected_line(),
                    None => self.tree.selected_line(),
                };
                if line.is_dir() {
                    AppStateCmdResult::from_optional_state(BrowserState::new(
                        line.path.clone(),
                        self.options.clone(),
                        TaskLifetime::unlimited(),
                    ))
                } else {
                    AppStateCmdResult::Launch(Launchable::opener(&line.path)?)
                }
            }
            Action::Verb(verb_key) => match verb_store.get(&verb_key) {
                Some(verb) => verb.execute(&self)?,
                None => AppStateCmdResult::verb_not_found(&verb_key),
            },
            Action::PatternEdit(pat) => match pat.len() {
                0 => {
                    self.filtered_tree = None;
                    AppStateCmdResult::Keep
                }
                _ => AppStateCmdResult::MustReapplyInterruptible,
            },
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

    fn reapply_interruptible(
        &mut self,
        cmd: &mut Command,
        _verb_store: &VerbStore,
        tl: TaskLifetime,
    ) {
        match &cmd.action {
            Action::PatternEdit(pat) => {
                let start = Instant::now();
                let pat = Pattern::from(pat);
                let mut options = self.options.clone();
                options.pattern = Some(pat.clone());
                let root = self.tree.root().clone();
                let len = self.tree.lines.len() as u16;
                let mut filtered_tree = TreeBuilder::from(root, options, tl).build(len as usize);
                if let Some(ref mut filtered_tree) = filtered_tree {
                    info!("Tree search took {:?}", start.elapsed());
                    filtered_tree.try_select_best_match();
                } // if none: task was cancelled from elsewhere
                self.filtered_tree = filtered_tree;
            }
            _ => {
                warn!("unexpected command in reapply");
            }
        }
    }

    fn display(&mut self, screen: &mut Screen, _verb_store: &VerbStore) -> io::Result<()> {
        screen.write_tree(&self.displayed_tree())
    }

    fn write_status(
        &self,
        screen: &mut Screen,
        cmd: &Command,
        verb_store: &VerbStore,
    ) -> io::Result<()> {
        match &cmd.action {
            //Action::FixPattern => screen.write_status_text("Hit <esc> to remove the filter"),
            Action::PatternEdit(_) => {
                screen.write_status_text("Hit <enter> to select, <esc> to remove the filter")
            }
            Action::VerbEdit(verb_key) => match verb_store.get(&verb_key) {
                Some(verb) => screen.write_status_text(
                    &format!(
                        "Hit <enter> to {} : {}",
                        &verb.name,
                        verb.description_for(&self)
                    )
                    .to_string(),
                ),
                None => screen.write_status_text(
                    // show what verbs start with the currently edited verb key
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
}
