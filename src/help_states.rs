//! an application state dedicated to help

use regex::Regex;
use std::io;
use termion::{color, style};

use app::{AppState, AppStateCmdResult};
use commands::{Action, Command};
use conf::Conf;
use screens::{Screen, ScreenArea};
use status::Status;
use task_sync::TaskLifetime;
use verbs::VerbStore;

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
    fn apply(
        &mut self,
        cmd: &mut Command,
        _verb_store: &VerbStore,
    ) -> io::Result<AppStateCmdResult> {
        Ok(match &cmd.action {
            Action::Back => AppStateCmdResult::PopState,
            Action::MoveSelection(dy) => {
                self.area.try_scroll(*dy);
                AppStateCmdResult::Keep
            }
            _ => AppStateCmdResult::Keep,
        })
    }

    fn reapply_interruptible(
        &mut self,
        _cmd: &mut Command,
        _verb_store: &VerbStore,
        _tl: TaskLifetime,
    ) {
        warn!("unexpected call of reapply in help state");
    }

    fn display(&mut self, screen: &mut Screen, verb_store: &VerbStore) -> io::Result<()> {
        let mut text = HelpText::new();
        text.md("");
        text.md(r#" **broot** (pronounce "b-root") lets you explore directory trees"#);
        text.md(r#"    and launch various commands on files."#);
        text.md("");
        text.md(r#" `<esc>` gets you back to the previous state."#);
        text.md(r#" Typing some letters searches the tree and selects the most relevant file."#);
        text.md(r#"    Use `<enter>` to freeze the filtering."#);
        text.md(r#" Typing a search, space, then a verb executes the verb on the file."#);
        text.md("");
        text.md(" Current Verbs:");
        for (key, verb) in verb_store.verbs.iter() {
            text.md(&format!(
                "{: >14} : `{}` => {}",
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
        self.area.content_length = text.lines.len() as i32;
        screen.write_lines(&self.area, &text.lines)?;
        Ok(())
    }

    fn write_status(&self, screen: &mut Screen, _cmd: &Command, _verb_store: &VerbStore) -> io::Result<()> {
        screen.write_status_text("Hit <esc> to get back to the tree")
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
            static ref bold_repl: String =
                String::from(format!("{}$1{}", style::Bold, style::Reset));
            static ref code_regex: Regex = Regex::new(r"`([^`]+)`").unwrap();
            static ref code_repl: String = String::from(format!(
                "{} $1 {}",
                color::Bg(color::AnsiValue::grayscale(2)),
                color::Bg(color::Reset)
            ));
        }
        let line = bold_regex.replace_all(line, &*bold_repl as &str); // TODO how to avoid this complex casting ?
        let line = code_regex.replace_all(&line, &*code_repl as &str);
        self.lines.push(line.to_string());
    }
}
