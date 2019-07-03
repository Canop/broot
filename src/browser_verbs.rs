use std::io;

use crate::app::AppStateCmdResult;
use crate::app_context::AppContext;
use crate::browser_states::BrowserState;
use crate::commands::Command;
use crate::external::{self, Launchable};
use crate::help_states::HelpState;
use crate::screens::Screen;
use crate::task_sync::TaskLifetime;
use crate::tree_options::{OptionBool, TreeOptions};
use crate::verb_invocation::VerbInvocation;
use crate::verbs::{Verb, VerbExecutor};

impl VerbExecutor for BrowserState {
    fn execute_verb(
        &self,
        verb: &Verb,
        invocation: &VerbInvocation,
        screen: &mut Screen,
        con: &AppContext,
    ) -> io::Result<AppStateCmdResult> {
        if let Some(err) = verb.match_error(invocation) {
            return Ok(AppStateCmdResult::DisplayError(err));
        }
        let tree = self.displayed_tree();
        let line = &tree.selected_line();
        Ok(match verb.execution.as_ref() {
            ":back" => AppStateCmdResult::PopState,
            ":focus" => {
                let mut path = tree.selected_line().target();
                if !path.is_dir() {
                    path = path.parent().unwrap().to_path_buf();
                }
                AppStateCmdResult::from_optional_state(
                    BrowserState::new(
                        path,
                        tree.options.clone(),
                        screen,
                        &TaskLifetime::unlimited(),
                    ),
                    tree.options.pattern.to_command(),
                )
            }
            ":help" => {
                AppStateCmdResult::NewState(Box::new(HelpState::new(screen, con)), Command::new())
            }
            ":open" => AppStateCmdResult::from(Launchable::opener(line.target())),
            ":parent" => match &line.path.parent() {
                Some(path) => AppStateCmdResult::from_optional_state(
                    BrowserState::new(
                        path.to_path_buf(),
                        tree.options.clone(),
                        screen,
                        &TaskLifetime::unlimited(),
                    ),
                    tree.options.pattern.to_command(),
                ),
                None => AppStateCmdResult::DisplayError("no parent found".to_string()),
            },
            ":print_path" => external::print_path(&line.target(), con)?,
            ":print_tree" => external::print_tree(&tree, screen, con)?,
            ":toggle_dates" => self.with_new_options(screen, &|o| o.show_dates ^= true),
            ":toggle_files" => {
                self.with_new_options(screen, &|o: &mut TreeOptions| o.only_folders ^= true)
            }
            ":toggle_hidden" => self.with_new_options(screen, &|o| o.show_hidden ^= true),
            ":toggle_git_ignore" => self.with_new_options(screen, &|options| {
                options.respect_git_ignore = match options.respect_git_ignore {
                    OptionBool::Auto => {
                        if tree.nb_gitignored > 0 {
                            OptionBool::No
                        } else {
                            OptionBool::Yes
                        }
                    }
                    OptionBool::Yes => OptionBool::No,
                    OptionBool::No => OptionBool::Yes,
                };
            }),
            ":toggle_perm" => self.with_new_options(screen, &|o| o.show_permissions ^= true),
            ":toggle_sizes" => self.with_new_options(screen, &|o| o.show_sizes ^= true),
            ":toggle_trim_root" => self.with_new_options(screen, &|o| o.trim_root ^= true),
            ":quit" => AppStateCmdResult::Quit,
            _ => verb.to_cmd_result(&line.path.clone(), &invocation.args, screen, con)?,
        })
    }
}
