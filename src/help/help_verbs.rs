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
        verb::{
            Internal,
            Verb,
            VerbExecutor,
            VerbExecution,
            VerbInvocation,
        },
    },
    super::HelpState,
};

impl VerbExecutor for HelpState {
    fn execute_verb(
        &mut self,
        verb: &Verb,
        user_invocation: Option<&VerbInvocation>,
        screen: &mut Screen,
        con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError> {
        if let Some(err) = user_invocation.and_then(|invocation| verb.match_error(invocation)) {
            return Ok(AppStateCmdResult::DisplayError(err));
        }
        Ok(match &verb.execution {
            VerbExecution::Internal{ internal, bang } => {
                use Internal::*;
                let bang = user_invocation.map(|inv| inv.bang).unwrap_or(*bang);
                match internal {
                    back => AppStateCmdResult::PopState,
                    focus | parent => AppStateCmdResult::from_optional_state(
                        BrowserState::new(
                            conf::dir(),
                            TreeOptions::default(),
                            screen,
                            &Dam::unlimited(),
                        ),
                        Command::new(),
                        bang,
                    ),
                    help => AppStateCmdResult::Keep,
                    line_down => {
                        self.scroll += 1;
                        AppStateCmdResult::Keep
                    }
                    line_up => {
                        self.scroll -= 1;
                        AppStateCmdResult::Keep
                    }
                    open_stay => match open::that(&Conf::default_location()) {
                        Ok(exit_status) => {
                            info!("open returned with exit_status {:?}", exit_status);
                            AppStateCmdResult::Keep
                        }
                        Err(e) => AppStateCmdResult::DisplayError(format!("{:?}", e)),
                    },
                    open_leave => AppStateCmdResult::from(Launchable::opener(Conf::default_location())),
                    page_down => {
                        self.scroll += self.text_area.height as i32;
                        AppStateCmdResult::Keep
                    }
                    page_up => {
                        self.scroll -= self.text_area.height as i32;
                        AppStateCmdResult::Keep
                    }
                    print_path => external::print_path(&Conf::default_location(), con)?,
                    print_relative_path => {
                        external::print_relative_path(&Conf::default_location(), con)?
                    }
                    quit => AppStateCmdResult::Quit,
                    focus_user_home
                    | focus_root
                    | toggle_dates
                    | toggle_files
                    | toggle_hidden
                    | toggle_git_ignore
                    | toggle_git_file_info
                    | toggle_git_status
                    | toggle_perm
                    | toggle_sizes
                    | toggle_trim_root
                        => AppStateCmdResult::PopStateAndReapply,
                    _  => AppStateCmdResult::Keep,
                }
            }
            VerbExecution::External(_) => verb.to_cmd_result(
                &Conf::default_location(),
                if let Some(inv) = &user_invocation {
                    &inv.args
                } else {
                    &None
                },
                screen,
                con,
            )?,
        })
    }
}
