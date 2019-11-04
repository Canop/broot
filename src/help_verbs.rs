//! this modules defines the execution of verbs on the help screen

use crate::{
    app::AppStateCmdResult,
    app_context::AppContext,
    browser_states::BrowserState,
    commands::Command,
    conf::{self, Conf},
    errors::ProgramError,
    external::{self, Launchable},
    help_states::HelpState,
    screens::Screen,
    task_sync::TaskLifetime,
    tree_options::TreeOptions,
    verb_invocation::VerbInvocation,
    verbs::{Verb, VerbExecutor},
};

impl VerbExecutor for HelpState {
    fn execute_verb(
        &mut self,
        verb: &Verb,
        invocation: &VerbInvocation,
        screen: &mut Screen,
        con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError> {
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
            ":line_down" => {
                self.view.try_scroll_lines(1);
                AppStateCmdResult::Keep
            }
            ":line_up" => {
                self.view.try_scroll_lines(-1);
                AppStateCmdResult::Keep
            }
            ":open" => AppStateCmdResult::from(Launchable::opener(Conf::default_location())),
            ":page_down" => {
                self.view.try_scroll_pages(1);
                AppStateCmdResult::Keep
            }
            ":page_up" => {
                self.view.try_scroll_pages(-1);
                AppStateCmdResult::Keep
            }
            ":print_path" => external::print_path(&Conf::default_location(), con)?,
            ":quit" => AppStateCmdResult::Quit,
            ":focus_user_home" | ":focus_root" => AppStateCmdResult::PopStateAndReapply,
            _ if verb.execution.starts_with(":toggle") => AppStateCmdResult::PopStateAndReapply,
            _ if verb.execution.starts_with(':') => AppStateCmdResult::Keep, // other internal verbs do nothing
            _ => verb.to_cmd_result(
                &Conf::default_location(),
                &invocation.args,
                screen,
                con,
            )?,
        })
    }
}
