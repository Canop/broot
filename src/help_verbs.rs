use std::fs::OpenOptions;
use std::io::{self, Write};

use crate::verbs::{Verb, VerbExecutor};
use crate::app::AppStateCmdResult;
use crate::app_context::AppContext;
use crate::browser_states::BrowserState;
use crate::conf::{self, Conf};
use crate::external::Launchable;
use crate::help_states::HelpState;
use crate::screens::Screen;
use crate::task_sync::TaskLifetime;
use crate::tree_options::TreeOptions;

impl VerbExecutor for HelpState {
    fn execute_verb(
        &self,
        verb: &Verb,
        screen: &Screen,
        con: &AppContext,
    ) -> io::Result<AppStateCmdResult> {
        Ok(match verb.exec_pattern.as_ref() {
            ":back" => AppStateCmdResult::PopState,
            ":focus" | ":parent" => {
                AppStateCmdResult::from_optional_state(BrowserState::new(
                    conf::dir(),
                    TreeOptions::new(),
                    screen,
                    &TaskLifetime::unlimited(),
                ))
            }
            ":help" => AppStateCmdResult::Keep,
            ":open" => AppStateCmdResult::Launch(Launchable::opener(&Conf::default_location())?),
            ":print_path" => {
                let path = Conf::default_location().to_string_lossy().to_string();
                if let Some(ref output_path) = con.launch_args.file_export_path {
                    // an output path was provided, we write to it
                    let f = OpenOptions::new().append(true).open(output_path)?;
                    writeln!(&f, "{}", path)?;
                    AppStateCmdResult::Quit
                } else {
                    // no output path provided. We write on stdout, but we must
                    // do it after app closing to have the normal terminal
                    let mut launchable = Launchable::from(vec![path])?;
                    launchable.just_print = true;
                    AppStateCmdResult::Launch(launchable)
                }
            }
            ":quit" => AppStateCmdResult::Quit,
            _ => {
                if verb.exec_pattern.starts_with(":toggle") {
                    AppStateCmdResult::PopStateAndReapply
                } else {
                    AppStateCmdResult::Launch(Launchable::from(
                        verb.exec_token(&Conf::default_location()),
                    )?)
                }
            }
        })
    }
}
