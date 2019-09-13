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
use crate::verb_invocation::VerbInvocation;
use crate::verbs::{Verb, VerbExecutor};

impl VerbExecutor for HelpState {
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
        Ok(match verb.execution.as_ref() {
            ":back" => AppStateCmdResult::PopState,
            ":focus" | ":parent" => AppStateCmdResult::from_optional_state(
                BrowserState::new(
                    conf::dir(),
                    TreeOptions::default(),
                    screen,
                    &TaskLifetime::unlimited(),
                ),
                Command::new(),
            ),
            ":help" => AppStateCmdResult::Keep,
            ":open" => AppStateCmdResult::from(Launchable::opener(Conf::default_location())),
            ":print_path" => external::print_path(&Conf::default_location(), con)?,
            ":quit" => AppStateCmdResult::Quit,
            ":focus_user_home" | ":focus_root" => AppStateCmdResult::PopStateAndReapply,
            _ if verb.execution.starts_with(":toggle") => AppStateCmdResult::PopStateAndReapply,
            _ if verb.execution.starts_with(":") => AppStateCmdResult::Keep, // other internal verbs do nothing
            _ => AppStateCmdResult::from(Launchable::program(
                verb.exec_token(&Conf::default_location(), &invocation.args),
            )?),
        })
    }
}
