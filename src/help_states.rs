
use {
    crate::{
        app_state::{AppState, AppStateCmdResult},
        app_context::AppContext,
        commands::{Action, Command},
        conf::Conf,
        errors::ProgramError,
        help_content,
        io::W,
        screens::Screen,
        status::Status,
        task_sync::TaskLifetime,
        verb_store::PrefixSearchResult,
        verbs::VerbExecutor,
    },
    lazy_format::lazy_format,
    joinery::JoinableIterator,
    crossterm::{
        terminal::{Clear, ClearType},
        QueueableCommand,
    },
    minimad::Composite,
    termimad::{
        Area,
        FmtText,
        TextView,
    },
};

/// an application state dedicated to help
pub struct HelpState {
    pub scroll: i32, // scroll position
    pub area: Area,
    dirty: bool, // background must be cleared
}

impl HelpState {
    pub fn new(_screen: &Screen, _con: & AppContext) -> HelpState {
        let area = Area::uninitialized(); // will be fixed at drawing time
        HelpState {
            area,
            scroll: 0,
            dirty: true,
        }
    }
}

impl AppState for HelpState {
    fn has_pending_task(&self) -> bool {
        false
    }

    fn can_execute(
        &self,
        _verb_index: usize,
        _con: &AppContext,
    ) -> bool {
        true // we'll probably refine this later
    }

    fn apply(
        &mut self,
        cmd: &mut Command,
        screen: &mut Screen,
        con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError> {
        Ok(match &cmd.action {
            Action::Back => AppStateCmdResult::PopState,
            Action::MoveSelection(dy) => {
                self.scroll += *dy;
                AppStateCmdResult::Keep
            }
            Action::Resize(w, h) => {
                screen.set_terminal_size(*w, *h, con);
                self.dirty = true;
                AppStateCmdResult::RefreshState{clear_cache: false}
            }
            Action::VerbIndex(index) => {
                let verb = &con.verb_store.verbs[*index];
                self.execute_verb(verb, &verb.invocation, screen, con)?
            }
            Action::VerbInvocate(invocation) => match con.verb_store.search(&invocation.name) {
                PrefixSearchResult::Match(verb) => {
                    self.execute_verb(verb, &invocation, screen, con)?
                }
                _ => AppStateCmdResult::verb_not_found(&invocation.name),
            },
            _ => AppStateCmdResult::Keep,
        })
    }

    fn refresh(&mut self, _screen: &Screen, _con: &AppContext) -> Command {
        Command::new()
    }

    fn do_pending_task(&mut self, _screen: &mut Screen, _tl: &TaskLifetime) {
        unreachable!();
    }

    fn display(
        &mut self,
        w: &mut W,
        screen: &Screen,
        con: &AppContext
    ) -> Result<(), ProgramError> {
        if self.dirty {
            screen.skin.default.queue_bg(w)?;
            screen.clear(w)?;
            self.area = Area::new(0, 0, screen.width, screen.height - 2);
            self.area.pad_for_max_width(110);
            self.dirty = false;
        }
        let text = help_content::build_text(con);
        let fmt_text = FmtText::from_text(&screen.help_skin, text, Some((self.area.width - 1) as usize));
        let mut text_view = TextView::from(&self.area, &fmt_text);
        self.scroll = text_view.set_scroll(self.scroll);
        Ok(text_view.write_on(w)?)
    }

    fn write_status(
        &self,
        w: &mut W,
        cmd: &Command,
        screen: &Screen,
        con: &AppContext,
    ) -> Result<(), ProgramError> {
        match &cmd.action {
            Action::VerbEdit(invocation) => {
                if invocation.name.is_empty() {
                    Status::from_message(
                        mad_inline!("Type a verb then *enter* to execute it (*?* for the list of verbs)"),
                    ).display(w, screen)
                } else {
                    match con.verb_store.search(&invocation.name) {
                        PrefixSearchResult::NoMatch => {
                            Status::from_error(mad_inline!("No matching verb")).display(w, screen)
                        }
                        PrefixSearchResult::Match(verb) => {
                            verb.write_status(w, None, Conf::default_location(), invocation, screen)
                        }
                        PrefixSearchResult::TooManyMatches(completions) => Status::from_message(
                            Composite::from_inline(&format!(
                                "Possible completions: {}",
                                completions.iter().map(|c| lazy_format!("*{}*", c)).join_with(", "),
                            )),
                        ).display(w, screen)
                    }
                }
            }
            _ => Status::from_message(mad_inline!(
                "Hit *esc* to get back to the tree, or a space to start a verb"
            )).display(w, screen),
        }
    }

    /// there's no meaningful flags here
    fn write_flags(
        &self,
        w: &mut W,
        screen: &mut Screen,
        _con: &AppContext
    ) -> Result<(), ProgramError> {
        screen.skin.default.queue_bg(w)?;
        w.queue(Clear(ClearType::UntilNewLine))?;
        Ok(())
    }
}

