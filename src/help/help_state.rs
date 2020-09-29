use {
    super::help_content,
    crate::{
        app::*,
        command::{Command, TriggerType},
        conf::Conf,
        display::{Screen, W},
        errors::ProgramError,
        launchable::Launchable,
        pattern::*,
        skin::PanelSkin,
        verb::*,
    },
    std::path::Path,
    termimad::{Area, FmtText, TextView},
};

/// an application state dedicated to help
pub struct HelpState {
    pub scroll: i32, // scroll position
    pub text_area: Area,
    dirty: bool, // background must be cleared
    pattern: Pattern,
}

impl HelpState {
    pub fn new(_screen: &Screen, _con: &AppContext) -> HelpState {
        let text_area = Area::uninitialized(); // will be fixed at drawing time
        HelpState {
            text_area,
            scroll: 0,
            dirty: true,
            pattern: Pattern::None,
        }
    }
}

impl AppState for HelpState {

    fn selected_path(&self) -> &Path {
        Conf::default_location()
    }

    fn selection(&self) -> Selection<'_> {
        Selection {
            path: Conf::default_location(),
            stype: SelectionType::File,
            is_exe: false,
            line: 0,
        }
    }

    fn refresh(&mut self, _screen: &Screen, _con: &AppContext) -> Command {
        self.dirty = true;
        Command::empty()
    }

    fn on_pattern(
        &mut self,
        pat: InputPattern,
        _con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError> {
        self.pattern = pat.pattern;
        Ok(AppStateCmdResult::Keep)
    }

    fn display(
        &mut self,
        w: &mut W,
        screen: &Screen,
        state_area: Area,
        panel_skin: &PanelSkin,
        con: &AppContext,
    ) -> Result<(), ProgramError> {
        let mut text_area = state_area.clone();
        text_area.pad_for_max_width(120);
        if text_area != self.text_area {
            self.dirty = true;
            self.text_area = text_area;
        }
        if self.dirty {
            panel_skin.styles.default.queue_bg(w)?;
            screen.clear_area_to_right(w, &state_area)?;
            self.dirty = false;
        }
        let mut expander = help_content::expander();
        expander
            .set("version", env!("CARGO_PKG_VERSION"))
            .set("config-path", &con.config_path);
        let verb_rows = help_content::matching_verb_rows(&self.pattern, con);
        for row in &verb_rows {
            let sub = expander
                .sub("verb-rows")
                .set_md("name", row.name())
                .set_md("shortcut", row.shortcut())
                .set("key", &row.verb.keys_desc);
            if row.verb.description.code {
                sub.set("description", "");
                sub.set("execution", &row.verb.description.content);
            } else {
                sub.set_md("description", &row.verb.description.content);
                sub.set("execution", "");
            }
        }
        let features = help_content::determine_features();
        expander.set(
            "features-text",
            if features.is_empty() {
                "This release was compiled with no optional feature enabled."
            } else {
                "This release was compiled with those optional features enabled:"
            },
        );
        for feature in &features {
            expander.sub("features")
                .set("feature-name", feature.0)
                .set("feature-description", feature.1);
        }
        let text = expander.expand();
        let fmt_text = FmtText::from_text(
            &panel_skin.help_skin,
            text,
            Some((self.text_area.width - 1) as usize),
        );
        let mut text_view = TextView::from(&self.text_area, &fmt_text);
        self.scroll = text_view.set_scroll(self.scroll);
        Ok(text_view.write_on(w)?)
    }

    fn no_verb_status(
        &self,
        _has_previous_state: bool,
        _con: &AppContext,
    ) -> Status {
        Status::from_message(
            "Hit *esc* to get back to the tree, or a space to start a verb"
        )
    }

    fn on_internal(
        &mut self,
        w: &mut W,
        internal_exec: &InternalExecution,
        input_invocation: Option<&VerbInvocation>,
        trigger_type: TriggerType,
        cc: &CmdContext,
        screen: &mut Screen,
    ) -> Result<AppStateCmdResult, ProgramError> {
        use Internal::*;
        Ok(match internal_exec.internal {
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
            toggle_dates | toggle_files | toggle_hidden | toggle_git_ignore
            | toggle_git_file_info | toggle_git_status | toggle_perm | toggle_sizes
            | toggle_trim_root => AppStateCmdResult::PopStateAndReapply,
            _ => self.on_internal_generic(
                w,
                internal_exec,
                input_invocation,
                trigger_type,
                cc,
                screen,
            )?,
        })
    }
}

