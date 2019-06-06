
use std::io;

use termimad::{Area, MadSkin, MadView};

use crate::app::{AppState, AppStateCmdResult};
use crate::app_context::AppContext;
use crate::commands::{Action, Command};
use crate::conf::Conf;
use crate::screens::{Screen};
use crate::status::Status;
use crate::task_sync::TaskLifetime;
use crate::verbs::VerbExecutor;
use crate::verb_store::{PrefixSearchResult};
use crate::help_content;

/// an application state dedicated to help
pub struct HelpState {
    view: MadView,
}

impl HelpState {

    pub fn new(_screen: &Screen, con: &AppContext) -> HelpState {
        let mut area = Area::uninitialized();
        area.top = 0;
        area.left = 0;
        let markdown = help_content::build_markdown(con);
        let mut skin = MadSkin::default();
        let view = MadView::from(
            markdown,
            area,
            skin
        );
        HelpState {
            view
        }
    }

    fn resize_area(&mut self, screen: &Screen) {
        let area = Area::new(0, 0, screen.w, screen.h - 3);
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
                PrefixSearchResult::Match(verb) => self.execute_verb(verb, &invocation, screen, con)?,
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

    fn refresh(
        &mut self,
        _screen: &Screen,
        _con: &AppContext,
    ) -> Command {
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
        let r = self.view.write();
        debug!("r={:?}", r);
        Ok(())
    }

    fn write_status(&self, screen: &mut Screen, cmd: &Command, con: &AppContext) -> io::Result<()> {
        debug!("help write_status");
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
                            .to_string()
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
