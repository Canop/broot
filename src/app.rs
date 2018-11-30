use std::io::{self, stdin, Write};
use std::path::PathBuf;
use termion::input::TermRead;

use commands::{Action, Command};
use external::Launchable;
use flat_tree::Tree;
use input::Input;
use patterns::Pattern;
use screens::Screen;
use status::Status;
use tree_build::TreeBuilder;
use tree_options::TreeOptions;
use tree_views::TreeView;
use verbs::VerbStore;

pub enum AppStateCmdResult {
    Quit,
    Keep,
    Launch(Launchable),
    DisplayError(String),
    NewRoot(PathBuf),
    NewOptions(TreeOptions),
    PopState,
}

impl AppStateCmdResult {
    fn verb_not_found(text: &str) -> AppStateCmdResult {
        AppStateCmdResult::DisplayError(format!("verb not found: {:?}", &text))
    }
}

pub struct AppState {
    pub tree: Tree,
    pub options: TreeOptions,
    // TODO pattern and filtered_tree are now always defined
    //         at the same time. So we need only one option.
    //         Either put the pattern in the tree, or both in
    //         a sub structure.
    pub pattern: Option<Pattern>,
    pub filtered_tree: Option<Tree>,
}

pub struct App {
    pub w: u16,
    pub h: u16,
    pub states: Vec<AppState>, // stack: the last one is current
}

impl AppState {
    pub fn new(tree: Tree, options: TreeOptions) -> AppState {
        AppState {
            tree,
            options,
            pattern: None,
            filtered_tree: None,
        }
    }
    pub fn own_displayed_tree(self) -> Tree {
        match self.filtered_tree {
            Some(tree) => tree,
            None => self.tree,
        }
    }
    pub fn displayed_tree(&self) -> &Tree {
        match &self.filtered_tree {
            Some(tree) => &tree,
            None => &self.tree,
        }
    }
    fn apply(
        &mut self,
        cmd: &mut Command,
        verb_store: &VerbStore,
    ) -> io::Result<AppStateCmdResult> {
        Ok(match &cmd.action {
            Action::Back => {
                if let Some(_) = self.pattern {
                    self.pattern = None;
                    self.filtered_tree = None;
                    // TODO try to keep selection ?
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
                    true => AppStateCmdResult::NewRoot(line.path.clone()),
                    false => AppStateCmdResult::Launch(Launchable::opener(&line.path)?),
                }
            },
            Action::Verb(verb_key) => match verb_store.get(&verb_key) {
                Some(verb) => verb.execute(&self)?,
                None => AppStateCmdResult::verb_not_found(&verb_key),
            },
            Action::Quit => AppStateCmdResult::Quit,
            Action::PatternEdit(pat) => {
                self.pattern = match pat.len() {
                    0 => {
                        self.filtered_tree = None;
                        None
                    }
                    _ => {
                        // TODO do the filtering in a separate thread as an interruptible task
                        let pat = Pattern::from(pat);
                        let mut options = self.options.clone();
                        options.pattern = Some(pat.clone());
                        let mut filtered_tree = TreeBuilder::from(
                            self.tree.root().clone(),
                            options
                        )?.build(self.tree.lines.len() as u16)?;
                        filtered_tree.try_select_best_match(&pat);
                        self.filtered_tree = Some(filtered_tree);
                        Some(pat)
                    }
                };
                AppStateCmdResult::Keep
            }
            Action::Next => {
                if let Some(ref mut tree) = self.filtered_tree {
                    if let Some(pattern) = &self.pattern {
                        tree.try_select_next_match(&pattern);
                    }
                }
                AppStateCmdResult::Keep
            }
            _ => AppStateCmdResult::Keep,
        })
    }
}

impl App {
    pub fn new() -> io::Result<App> {
        let (w, h) = termion::terminal_size()?;
        let states = Vec::new();
        Ok(App { w, h, states })
    }

    pub fn push(&mut self, path: PathBuf, options: TreeOptions) -> io::Result<()> {
        let tree = TreeBuilder::from(path, options.clone())?.build(self.h - 2)?;
        self.states.push(AppState::new(tree, options));
        Ok(())
    }

    pub fn mut_state(&mut self) -> &mut AppState {
        match self.states.last_mut() {
            Some(s) => s,
            None => {
                panic!("No path has been pushed");
            }
        }
    }
    pub fn state(&self) -> &AppState {
        match self.states.last() {
            Some(s) => s,
            None => {
                panic!("No path has been pushed");
            }
        }
    }

    pub fn run(mut self, verb_store: &VerbStore) -> io::Result<Option<Launchable>> {
        let mut screen = Screen::new(self.w, self.h)?;
        write!(
            screen.stdout,
            "{}{}",
            termion::clear::All,
            termion::cursor::Hide
        )?;
        screen.write_tree(&self.state().tree, &self.state().pattern)?;
        screen.write_status_initial()?;
        let stdin = stdin();
        let keys = stdin.keys();
        let mut cmd = Command::new();
        for c in keys {
            //debug!("key: {:?}", &c);
            cmd.add_key(c?)?;
            info!("{:?}", &cmd.action);
            match self.mut_state().apply(&mut cmd, &verb_store)? {
                AppStateCmdResult::Quit => {
                    break;
                }
                AppStateCmdResult::Launch(launchable) => {
                    return Ok(Some(launchable));
                }
                AppStateCmdResult::NewRoot(path) => {
                    let options = self.state().options.clone();
                    self.push(path, options)?;
                    cmd = Command::new();
                    screen.write_status(&self.state())?;
                }
                AppStateCmdResult::NewOptions(options) => {
                    let path = self.state().tree.root().clone();
                    self.push(path, options)?;
                    cmd = Command::new();
                    screen.write_status(&self.state())?;
                }
                AppStateCmdResult::PopState => {
                    self.states.pop();
                    //cmd = Command::from(&self.state().tree.key()); // doesn't really seem convenient
                    cmd = Command::new();
                    screen.write_status(&self.state())?;
                }
                AppStateCmdResult::DisplayError(txt) => {
                    screen.write_status_err(&txt)?;
                }
                AppStateCmdResult::Keep => {
                    screen.write_status(&self.state())?;
                }
            }
            screen.write_tree(self.state().displayed_tree(), &self.state().pattern)?;
            screen.write_input(&cmd)?;
        }
        Ok(None)
    }
}
