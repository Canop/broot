use std::io;

use crate::app::AppStateCmdResult;
use crate::app_context::AppContext;
use crate::browser_states::BrowserState;
use crate::commands::Command;
use crate::conf::{self, Conf};
use crate::external::{self, Launchable};
use crate::help_states::HelpState;
use crate::screens::Screen;
use crate::task_sync::TaskLifetime;
use crate::tree_options::TreeOptions;
use crate::verbs::{Verb, VerbExecutor};

impl VerbExecutor for HelpState {
    fn execute_verb(
        &self,
        verb: &Verb,
        args: &Option<String>,
        screen: &Screen,
        con: &AppContext,
    ) -> io::Result<AppStateCmdResult> {
        Ok(match verb.execution.as_ref() {
            ":back" => AppStateCmdResult::PopState,
            ":focus" | ":parent" => AppStateCmdResult::from_optional_state(
                BrowserState::new(
                    conf::dir(),
                    TreeOptions::new(),
                    screen,
                    &TaskLifetime::unlimited(),
                ),
                Command::new(),
            ),
            ":help" => AppStateCmdResult::Keep,
            ":open" => AppStateCmdResult::Launch(Launchable::opener(&Conf::default_location())?),
            ":print_path" => external::print_path(&Conf::default_location(), con)?,
            ":quit" => AppStateCmdResult::Quit,
            _ => {
                if verb.execution.starts_with(":toggle") {
                    AppStateCmdResult::PopStateAndReapply
                } else {
                    AppStateCmdResult::Launch(Launchable::from(
                        verb.exec_token(&Conf::default_location(), args),
                    )?)
                }
            }
        })
    }
}
