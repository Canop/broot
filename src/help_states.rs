//! an application state dedicated to help

use std::io;

use crate::app::{AppState, AppStateCmdResult};
use crate::app_context::AppContext;
use crate::commands::{Action, Command};
use crate::conf::Conf;
use crate::screen_text::{Text, TextTable};
use crate::screens::{Screen, ScreenArea};
use crate::status::Status;
use crate::task_sync::TaskLifetime;
use crate::verbs::{PrefixSearchResult, Verb, VerbExecutor};

pub struct HelpState {
    area: ScreenArea, // where the help is drawn
}

impl HelpState {
    pub fn new(screen: &Screen) -> HelpState {
        let mut state = HelpState {
            area: ScreenArea::new(1, 1, 1),
        };
        state.resize_area(screen);
        state
    }
    fn resize_area(&mut self, screen: &Screen) {
        self.area.bottom = screen.h - 2;
        self.area.width = screen.w;
    }
}

impl AppState for HelpState {
    fn apply(
        &mut self,
        cmd: &mut Command,
        screen: &Screen,
        con: &AppContext,
    ) -> io::Result<AppStateCmdResult> {
        self.resize_area(screen);
        Ok(match &cmd.action {
            Action::Back => AppStateCmdResult::PopState,
            Action::Verb(verb_key) => match con.verb_store.search(&verb_key) {
                PrefixSearchResult::Match(verb) => self.execute_verb(verb, screen, con)?,
                _ => AppStateCmdResult::verb_not_found(&verb_key),
            },
            Action::MoveSelection(dy) => {
                self.area.try_scroll(*dy);
                AppStateCmdResult::Keep
            }
            Action::ScrollPage(dp) => {
                self.area.try_scroll(*dp * (self.area.height()-1));
                AppStateCmdResult::Keep
            }
            _ => AppStateCmdResult::Keep,
        })
    }

    fn has_pending_tasks(&self) -> bool {
        false
    }

    fn do_pending_task(&mut self, _screen: &mut Screen, _tl: &TaskLifetime) {
        unreachable!();
    }

    fn display(&mut self, screen: &mut Screen, con: &AppContext) -> io::Result<()> {
        let mut text = Text::new(&screen.skin);
        text.md("");
        text.md(r#" **broot** lets you explore directory trees and launch commands."#);
        text.md(r#" site: https://github.com/Canop/broot."#);
        text.md(r#" broot is best used when launched as `br`."#);
        text.md("");
        text.md(r#" `<esc>` gets you back to the previous state."#);
        text.md(r#" Typing some letters searches the tree and selects the most relevant file."#);
        text.md(r#" To use a regular expression, use a slash eg `/j(ava|s)$`."#);
        text.md("");
        text.md(r#" To execute a verb, type a space or `:` then start of its name or shortcut."#);
        text.md(" Verbs:");
        let mut tbl: TextTable<Verb> = TextTable::new(&screen.skin);
        tbl.add_col("name", &|verb| &verb.name);
        tbl.add_col("shortcut", &|verb| {
            if let Some(sk) = &verb.short_key {
                &sk
            } else {
                ""
            }
        });
        tbl.write(&con.verb_store.verbs, &mut text);
        text.md("");
        text.md(&format!(
            " Verb can be configured in {:?}.",
            Conf::default_location()
        ));
        text.md("");
        text.md(" Some options can be set on launch:");
        text.md("  `-h` or `--hidden` : show hidden files");
        text.md("  `-f` or `--only-folders` : only show folders");
        text.md("  `-s` or `--sizes` : display sizes");
        text.md("  (for the complete list, run `broot --help`)");
        text.md("");
        text.md(" Flags are displayed at bottom right:");
        text.md("  `h:y` or `h:n` : whether hidden files are shown");
        text.md("  `gi:a`, `gi:y`, `gi:n` : gitignore on auto, yes or no");
        text.md("  When gitignore is auto, .gitignore rules are respected if");
        text.md("   the displayed root is a git repository or in one.");
        self.area.content_length = text.height() as i32;
        screen.reset_colors()?;
        text.write(screen, &self.area)?;
        Ok(())
    }

    fn write_status(
        &self,
        screen: &mut Screen,
        cmd: &Command,
        con: &AppContext,
    ) -> io::Result<()> {
        match &cmd.action {
            Action::VerbEdit(verb_key) => {
                match con.verb_store.search(&verb_key) {
                    PrefixSearchResult::NoMatch => {
                        screen.write_status_err("No matching verb)")
                    }
                    PrefixSearchResult::Match(verb) => screen.write_status_text(
                        &format!(
                            "Hit <enter> to {} : {}",
                            &verb.name,
                            &verb.description_for(Conf::default_location())
                        )
                        .to_string(),
                    ),
                    PrefixSearchResult::TooManyMatches => screen.write_status_text(
                        "Type a verb then <enter> to execute it",
                    ),
                }
            }
            _ => {
                screen.write_status_text("Hit <esc> to get back to the tree, or a space to start a verb")
            }
        }
    }

    fn write_flags(&self, _screen: &mut Screen, _con: &AppContext) -> io::Result<()> {
        Ok(())
    }
}
