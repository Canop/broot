//! this modules defines the execution of verbs on the help screen

use {
    crate::{
        app::{
            AppContext,
            AppStateCmdResult,
        },
        browser::BrowserState,
        command::Command,
        conf::{self, Conf},
        errors::ProgramError,
        external::{self, Launchable},
        screens::Screen,
        task_sync::Dam,
        tree_options::TreeOptions,
        verb_invocation::VerbInvocation,
        verbs::{Verb, VerbExecutor},
    },
    super::HelpState,
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
                    &Dam::unlimited(),
                ),
                Command::new(),
            ),
            ":help" => AppStateCmdResult::Keep,
            ":line_down" => {
                self.scroll += 1;
                AppStateCmdResult::Keep
            }
            ":line_up" => {
                self.scroll -= 1;
                AppStateCmdResult::Keep
            }
            ":open_stay" => match open::that(&Conf::default_location()) {
                Ok(exit_status) => {
                    info!("open returned with exit_status {:?}", exit_status);
                    AppStateCmdResult::Keep
                }
                Err(e) => AppStateCmdResult::DisplayError(format!("{:?}", e)),
            },
            ":open_leave" => AppStateCmdResult::from(Launchable::opener(Conf::default_location())),
            ":page_down" => {
                self.scroll += self.area.height as i32;
                AppStateCmdResult::Keep
            }
            ":page_up" => {
                self.scroll -= self.area.height as i32;
                AppStateCmdResult::Keep
            }
            ":print_path" => external::print_path(&Conf::default_location(), con)?,
            ":print_relative_path" => {
                external::print_relative_path(&Conf::default_location(), con)?
            }
            ":quit" => AppStateCmdResult::Quit,
            ":focus_user_home" | ":focus_root" => AppStateCmdResult::PopStateAndReapply,
            _ if verb.execution.starts_with(":toggle") => AppStateCmdResult::PopStateAndReapply,
            _ if verb.execution.starts_with(':') => AppStateCmdResult::Keep,
            _ => verb.to_cmd_result(
                &Conf::default_location(),
                &invocation.args,
                screen,
                con,
            )?,
        })
    }
}
