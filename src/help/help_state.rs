use {
    super::help_content,
    crate::{
        app::{AppContext, AppState, AppStateCmdResult, Status},
        browser::BrowserState,
        command::{Command, TriggerType},
        conf::{self, Conf},
        display::{Screen, W},
        errors::ProgramError,
        launchable::Launchable,
        print,
        selection_type::SelectionType,
        task_sync::Dam,
        tree::TreeOptions,
        verb::*,
    },
    crossterm::{
        terminal::{Clear, ClearType},
        QueueableCommand,
    },
    std::path::Path,
    termimad::{Area, FmtText, TextView},
};

/// an application state dedicated to help
pub struct HelpState {
    pub scroll: i32, // scroll position
    pub text_area: Area,
    dirty: bool, // background must be cleared
}

impl HelpState {
    pub fn new(_screen: &Screen, _con: &AppContext) -> HelpState {
        let text_area = Area::uninitialized(); // will be fixed at drawing time
        HelpState {
            text_area,
            scroll: 0,
            dirty: true,
        }
    }
}

impl AppState for HelpState {
    fn get_pending_task(&self) -> Option<&'static str> {
        None
    }

    fn selected_path(&self) -> &Path {
        Conf::default_location()
    }

    fn selection_type(&self) -> SelectionType {
        SelectionType::Any
    }

    fn refresh(&mut self, _screen: &Screen, _con: &AppContext) -> Command {
        self.dirty = true;
        Command::empty()
    }

    fn do_pending_task(&mut self, _screen: &mut Screen, _dam: &mut Dam) {
        unreachable!();
    }

    fn display(
        &mut self,
        w: &mut W,
        screen: &Screen,
        state_area: Area,
        con: &AppContext,
    ) -> Result<(), ProgramError> {
        if self.dirty {
            screen.skin.default.queue_bg(w)?;
            screen.clear(w)?;
            self.text_area = state_area;
            self.text_area.pad_for_max_width(110);
            self.dirty = false;
        }
        let text = help_content::build_text(con);
        let fmt_text = FmtText::from_text(
            &screen.help_skin,
            text,
            Some((self.text_area.width - 1) as usize),
        );
        let mut text_view = TextView::from(&self.text_area, &fmt_text);
        self.scroll = text_view.set_scroll(self.scroll);
        Ok(text_view.write_on(w)?)
    }

    fn get_status(&self, cmd: &Command, con: &AppContext) -> Status {
        match cmd {
            Command::VerbEdit(invocation) => {
                if invocation.name.is_empty() {
                    Status::from_message(
                        "Type a verb then *enter* to execute it (*?* for the list of verbs)",
                    )
                } else {
                    match con.verb_store.search(&invocation.name) {
                        PrefixSearchResult::NoMatch => Status::from_error("No matching verb"),
                        PrefixSearchResult::Match(verb) => {
                            verb.get_status(Conf::default_location(), invocation)
                        }
                        PrefixSearchResult::TooManyMatches(completions) => {
                            Status::from_message(format!(
                                "Possible completions: {}",
                                completions
                                    .iter()
                                    .map(|c| format!("*{}*", c))
                                    .collect::<Vec<String>>()
                                    .join(", "),
                            ))
                        }
                    }
                }
            }
            _ => Status::from_message(
                "Hit *esc* to get back to the tree, or a space to start a verb",
            ),
        }
    }

    /// there's no meaningful flags here
    fn write_flags(
        &self,
        w: &mut W,
        screen: &mut Screen,
        _con: &AppContext,
    ) -> Result<(), ProgramError> {
        screen.skin.default.queue_bg(w)?;
        w.queue(Clear(ClearType::UntilNewLine))?;
        Ok(())
    }

    fn on_internal(
        &mut self,
        internal_exec: &InternalExecution,
        input_invocation: Option<&VerbInvocation>,
        trigger_type: TriggerType,
        screen: &mut Screen,
        con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError> {
        use Internal::*;
        let bang = input_invocation
            .map(|inv| inv.bang)
            .unwrap_or(internal_exec.bang);
        Ok(match internal_exec.internal {
            back => AppStateCmdResult::PopState,
            focus | parent => AppStateCmdResult::from_optional_state(
                BrowserState::new(
                    conf::dir(),
                    TreeOptions::default(),
                    screen,
                    &Dam::unlimited(),
                ),
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
            open_leave => {
                AppStateCmdResult::from(Launchable::opener(
                    Conf::default_location().to_path_buf()
                ))
            }
            page_down => {
                self.scroll += self.text_area.height as i32;
                AppStateCmdResult::Keep
            }
            page_up => {
                self.scroll -= self.text_area.height as i32;
                AppStateCmdResult::Keep
            }
            print_path => print::print_path(&Conf::default_location(), con)?,
            print_relative_path => print::print_relative_path(&Conf::default_location(), con)?,
            quit => AppStateCmdResult::Quit,
            toggle_dates | toggle_files | toggle_hidden | toggle_git_ignore
            | toggle_git_file_info | toggle_git_status | toggle_perm | toggle_sizes
            | toggle_trim_root => AppStateCmdResult::PopStateAndReapply,
            _ => AppStateCmdResult::Keep,
        })
    }
}
