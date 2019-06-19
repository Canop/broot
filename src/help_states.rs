use std::io;

use crossterm::{Terminal, ClearType};
use termimad::{Area, MadView};

use crate::app::{AppState, AppStateCmdResult};
use crate::app_context::AppContext;
use crate::commands::{Action, Command};
use crate::conf::Conf;
use crate::help_content;
use crate::screens::Screen;
use crate::status::Status;
use crate::task_sync::TaskLifetime;
use crate::verb_store::PrefixSearchResult;
use crate::verbs::VerbExecutor;

/// an application state dedicated to help
pub struct HelpState {
    view: MadView,
}

impl HelpState {
    pub fn new(screen: &Screen, con: &AppContext) -> HelpState {
        let area = Area::uninitialized(); // will be fixed at drawing time
        Terminal::new().clear(ClearType::All).unwrap();
        let markdown = help_content::build_markdown(con);
        let view = MadView::from(markdown, area, screen.skin.to_mad_skin());
        HelpState { view }
    }

    fn resize_area(&mut self, screen: &Screen) {
        let mut area = Area::new(0, 0, screen.w, screen.h - 2);
        area.pad_for_max_width(100);
        self.view.resize(&area);
    }
}

impl AppState for HelpState {
    fn apply(
        &mut self,
        cmd: &mut Command,
        screen: &mut Screen,
        con: &AppContext,
    ) -> io::Result<AppStateCmdResult> {
        self.resize_area(screen);
        Ok(match &cmd.action {
            Action::Back => AppStateCmdResult::PopState,
            Action::Verb(invocation) => match con.verb_store.search(&invocation.key) {
                PrefixSearchResult::Match(verb) => {
                    self.execute_verb(verb, &invocation, screen, con)?
                }
                _ => AppStateCmdResult::verb_not_found(&invocation.key),
            },
            Action::MoveSelection(dy) => {
                self.view.try_scroll_lines(*dy);
                AppStateCmdResult::Keep
            }
            Action::ScrollPage(dp) => {
                self.view.try_scroll_pages(*dp);
                AppStateCmdResult::Keep
            }
            Action::Refresh => AppStateCmdResult::RefreshState,
            Action::Quit => AppStateCmdResult::Quit,
            _ => AppStateCmdResult::Keep,
        })
    }

    fn refresh(&mut self, _screen: &Screen, _con: &AppContext) -> Command {
        Command::new()
    }

    fn has_pending_tasks(&self) -> bool {
        false
    }

    fn do_pending_task(&mut self, _screen: &mut Screen, _tl: &TaskLifetime) {
        unreachable!();
    }

    fn display(&mut self, screen: &mut Screen, _con: &AppContext) -> io::Result<()> {
        self.resize_area(screen);
        self.view.write()
    }

    fn write_status(&self, screen: &mut Screen, cmd: &Command, con: &AppContext) -> io::Result<()> {
        match &cmd.action {
            Action::VerbEdit(invocation) => match con.verb_store.search(&invocation.key) {
                PrefixSearchResult::NoMatch => screen.write_status_err("No matching verb)"),
                PrefixSearchResult::Match(verb) => {
                    if let Some(err) = verb.match_error(invocation) {
                        screen.write_status_err(&err)
                    } else {
                        screen.write_status_text(
                            &format!(
                                "Hit <enter> to {} : {}",
                                &verb.invocation.key,
                                &verb.description_for(Conf::default_location(), &invocation.args)
                            )
                            .to_string(),
                        )
                    }
                }
                PrefixSearchResult::TooManyMatches => {
                    screen.write_status_text("Type a verb then <enter> to execute it")
                }
            },
            _ => screen
                .write_status_text("Hit <esc> to get back to the tree, or a space to start a verb"),
        }
    }

    fn write_flags(&self, _screen: &mut Screen, _con: &AppContext) -> io::Result<()> {
        Ok(())
    }
}
