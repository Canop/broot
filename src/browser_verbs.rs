use std::io;
use std::path::PathBuf;
use directories::UserDirs;

use crate::app::AppStateCmdResult;
use crate::app_context::AppContext;
use crate::browser_states::BrowserState;
use crate::commands::Command;
use crate::external;
use crate::flat_tree::Tree;
use crate::help_states::HelpState;
use crate::screens::Screen;
use crate::task_sync::TaskLifetime;
use crate::tree_options::{OptionBool, TreeOptions};
use crate::verb_invocation::VerbInvocation;
use crate::verbs::{Verb, VerbExecutor};

fn focus_path(
    path: PathBuf,
    screen: &mut Screen,
    tree: &Tree,
) -> AppStateCmdResult {
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

impl VerbExecutor for BrowserState {
    fn execute_verb(
        &mut self,
        verb: &Verb,
        invocation: &VerbInvocation,
        screen: &mut Screen,
        con: &AppContext,
    ) -> io::Result<AppStateCmdResult> {
        if let Some(err) = verb.match_error(invocation) {
            return Ok(AppStateCmdResult::DisplayError(err));
        }
        let page_height = BrowserState::page_height(screen);
        Ok(match verb.execution.as_ref() {
            ":back" => AppStateCmdResult::PopState,
            ":focus" => {
                let tree = self.displayed_tree_mut();
                let line = &tree.selected_line();
                let mut path = line.target();
                if !path.is_dir() {
                    path = path.parent().unwrap().to_path_buf();
                }
                focus_path(path, screen, tree)
            }
            ":focus_root" => focus_path(PathBuf::from("/"), screen, self.displayed_tree()),
            ":focus_user_home" => match UserDirs::new() {
                Some(ud) => focus_path(ud.home_dir().to_path_buf(), screen, self.displayed_tree()),
                None => AppStateCmdResult::DisplayError("no user home directory found".to_string()), // does this happen ?
            }
            ":help" => AppStateCmdResult::NewState(Box::new(HelpState::new(screen, con)), Command::new()),
            //":open" => AppStateCmdResult::from(Launchable::opener(self.displayed_tree().selected_line().target())),
            ":open_stay" => self.open_selection_stay_in_broot(screen, con)?,
            ":open_leave" => self.open_selection_quit_broot(screen, con)?,
            ":line_down" => {
                self.displayed_tree_mut().move_selection(1, page_height);
                AppStateCmdResult::Keep
            }
            ":line_up" => {
                self.displayed_tree_mut().move_selection(-1, page_height);
                AppStateCmdResult::Keep
            }
            ":page_down" => {
                let tree = self.displayed_tree_mut();
                if page_height < tree.lines.len() as i32 {
                    tree.try_scroll(page_height, page_height);
                }
                AppStateCmdResult::Keep
            }
            ":page_up" => {
                let tree = self.displayed_tree_mut();
                if page_height < tree.lines.len() as i32 {
                    tree.try_scroll(-page_height, page_height);
                }
                AppStateCmdResult::Keep
            }
            ":parent" => match &self.displayed_tree().selected_line().path.parent() {
                Some(path) => focus_path(path.to_path_buf(), screen, self.displayed_tree()),
                None => AppStateCmdResult::DisplayError("no parent found".to_string()),
            }
            ":print_path" => external::print_path(&self.displayed_tree().selected_line().target(), con)?,
            ":print_tree" => external::print_tree(&self.displayed_tree(), screen, con)?,
            ":refresh" => AppStateCmdResult::RefreshState,
            ":select_first" => {
                self.displayed_tree_mut().try_select_first();
                AppStateCmdResult::Keep
            }
            ":select_last" => {
                self.displayed_tree_mut().try_select_last();
                AppStateCmdResult::Keep
            }
            ":toggle_dates" => self.with_new_options(screen, &|o| o.show_dates ^= true),
            ":toggle_files" => self.with_new_options(screen, &|o: &mut TreeOptions| o.only_folders ^= true),
            ":toggle_hidden" => self.with_new_options(screen, &|o| o.show_hidden ^= true),
            ":toggle_git_ignore" => self.with_new_options(screen, &|options| {
                options.respect_git_ignore = match options.respect_git_ignore {
                    OptionBool::Auto => {
                        if self.displayed_tree().nb_gitignored > 0 {
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
            _ => verb.to_cmd_result(&self.displayed_tree().selected_line().path.clone(), &invocation.args, screen, con)?,
        })
    }
}
