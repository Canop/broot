use std::fs::OpenOptions;
use std::io::{self, Write};

use crate::app::AppStateCmdResult;
use crate::app_context::AppContext;
use crate::browser_states::BrowserState;
use crate::commands::Command;
use crate::external::Launchable;
use crate::help_states::HelpState;
use crate::screens::Screen;
use crate::task_sync::TaskLifetime;
use crate::tree_options::{OptionBool, TreeOptions};
use crate::verbs::{Verb, VerbExecutor};

impl VerbExecutor for BrowserState {
    fn execute_verb(
        &self,
        verb: &Verb,
        screen: &Screen,
        con: &AppContext,
    ) -> io::Result<AppStateCmdResult> {
        let tree = match &self.filtered_tree {
            Some(tree) => &tree,
            None => &self.tree,
        };
        let line = &tree.selected_line();
        Ok(match verb.exec_pattern.as_ref() {
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
                AppStateCmdResult::NewState(Box::new(HelpState::new(screen)), Command::new())
            }
            ":open" => AppStateCmdResult::Launch(Launchable::opener(&line.target())?),
            ":parent" => match &line.target().parent() {
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
            ":print_path" => {
                if let Some(ref output_path) = con.launch_args.file_export_path {
                    // an output path was provided, we write to it
                    let f = OpenOptions::new().append(true).open(output_path)?;
                    writeln!(&f, "{}", line.target().to_string_lossy())?;
                    AppStateCmdResult::Quit
                } else {
                    // no output path provided. We write on stdout, but we must
                    // do it after app closing to have the normal terminal
                    let mut launchable =
                        Launchable::from(vec![line.target().to_string_lossy().to_string()])?;
                    launchable.just_print = true;
                    AppStateCmdResult::Launch(launchable)
                }
            }
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
            _ => verb.to_cmd_result(&line.target(), con)?,
        })
    }
}
