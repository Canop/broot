#![warn(clippy::all)]

//! an application state dedicated to help

use regex::Regex;
use std::io;
use termion::{color, style};

use crate::app::{AppState, AppStateCmdResult};
use crate::app_context::AppContext;
use crate::commands::{Action, Command};
use crate::conf::Conf;
use crate::screens::{Screen, ScreenArea};
use crate::status::Status;
use crate::task_sync::TaskLifetime;
use crate::verbs::VerbExecutor;

pub struct HelpState {
    area: ScreenArea, // where the help is drawn
}

impl HelpState {
    pub fn new(_about: &str) -> HelpState {
        let (_, h) = termion::terminal_size().unwrap();
        let area = ScreenArea::new(1, h - 2);
        HelpState { area }
    }
}

impl AppState for HelpState {
    fn apply(&mut self, cmd: &mut Command, con: &AppContext) -> io::Result<AppStateCmdResult> {
        Ok(match &cmd.action {
            Action::Back => AppStateCmdResult::PopState,
            Action::Verb(verb_key) => match con.verb_store.get(&verb_key) {
                Some(verb) => self.execute_verb(verb, con)?,
                None => AppStateCmdResult::verb_not_found(&verb_key),
            },
            Action::MoveSelection(dy) => {
                self.area.try_scroll(*dy);
                AppStateCmdResult::Keep
            }
            _ => AppStateCmdResult::Keep,
        })
    }

    fn has_pending_tasks(&self) -> bool {
        false
    }

    fn do_pending_task(&mut self, _tl: &TaskLifetime) {
        // can't happen
    }

    fn display(&mut self, screen: &mut Screen, con: &AppContext) -> io::Result<()> {
        let mut text = HelpText::new();
        text.md("");
        text.md(r#" **broot** (pronounce "b-root") lets you explore directory trees"#);
        text.md(r#"    and launch various commands on files."#);
        text.md("");
        text.md(r#" `<esc>` gets you back to the previous state."#);
        text.md(r#" Typing some letters searches the tree and selects the most relevant file."#);
        text.md(r#" Typing a search, a space or `:`, then a verb executes the verb on the file."#);
        text.md("");
        text.md(" Current Verbs:");
        for (key, verb) in con.verb_store.verbs.iter() {
            text.md(&format!(
                "{: >17} : `{}` => {}",
                &verb.name,
                key,
                verb.description()
            ));
        }
        text.md("");
        text.md(&format!(
            " Verbs are configured in {:?}.",
            Conf::default_location()
        ));
        text.md("");
        text.md(" Some options can be set on launch:");
        text.md("  `-h` or `--hidden` : show hidden files");
        text.md("  `-f` or `--only-folders` : only show folders");
        text.md("  `-s` or `--sizes` : display sizes");
        text.md("");
        text.md(" Flags are displayed at bottom right:");
        text.md("  `h:y` or `h:n` : whether hidden files are shown");
        text.md("  `gi:a`, `gi:y`, `gi:n` : gitignore on auto, yes or no");
        text.md("  When gitignore is auto, .gitignore rules are respected if");
        text.md("   the displayed root is a git repository or in one.");

        self.area.content_length = text.lines.len() as i32;
        screen.write_lines(&self.area, &text.lines)?;
        Ok(())
    }

    fn write_status(
        &self,
        screen: &mut Screen,
        _cmd: &Command,
        _con: &AppContext,
    ) -> io::Result<()> {
        screen.write_status_text("Hit <esc> to get back to the tree")
    }

    fn write_flags(&self, _screen: &mut Screen, _con: &AppContext) -> io::Result<()> {
        Ok(())
    }
}

struct HelpText {
    lines: Vec<String>,
}
impl HelpText {
    pub fn new() -> HelpText {
        HelpText { lines: Vec::new() }
    }
    pub fn md(&mut self, line: &str) {
        lazy_static! {
            static ref bold_regex: Regex = Regex::new(r"\*\*([^*]+)\*\*").unwrap();
            static ref bold_repl: String = format!("{}$1{}", style::Bold, style::Reset);
            static ref code_regex: Regex = Regex::new(r"`([^`]+)`").unwrap();
            static ref code_repl: String = format!(
                "{} $1 {}",
                color::Bg(color::AnsiValue::grayscale(2)),
                color::Bg(color::Reset)
            );
        }
        let line = bold_regex.replace_all(line, &*bold_repl as &str); // TODO how to avoid this complex casting ?
        let line = code_regex.replace_all(&line, &*code_repl as &str);
        self.lines.push(line.to_string());
    }
}
