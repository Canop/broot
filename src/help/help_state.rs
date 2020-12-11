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
        tree::TreeOptions,
        verb::*,
    },
    std::path::{Path, PathBuf},
    termimad::{Area, FmtText, TextView},
};

/// an application state dedicated to help
pub struct HelpState {
    pub scroll: i32, // scroll position
    pub text_area: Area,
    dirty: bool, // background must be cleared
    pattern: Pattern,
    tree_options: TreeOptions,
    config_path: PathBuf, // the last config path when several were used
}

impl HelpState {
    pub fn new(
        tree_options: TreeOptions,
        _screen: Screen,
        con: &AppContext,
    ) -> HelpState {
        let text_area = Area::uninitialized(); // will be fixed at drawing time
        let config_path = con.config_paths
            .last()
            .cloned()
            .unwrap_or_else(||Conf::default_location());
        HelpState {
            text_area,
            scroll: 0,
            dirty: true,
            pattern: Pattern::None,
            tree_options,
            config_path,
        }
    }
}

impl AppState for HelpState {

    fn selected_path(&self) -> &Path {
        &self.config_path
    }

    fn tree_options(&self) -> TreeOptions {
        self.tree_options.clone()
    }

    fn with_new_options(
        &mut self,
        _screen: Screen,
        change_options: &dyn Fn(&mut TreeOptions),
        _in_new_panel: bool, // TODO open a tree if true
        _con: &AppContext,
    ) -> AppStateCmdResult {
        change_options(&mut self.tree_options);
        AppStateCmdResult::Keep
    }

    fn selection(&self) -> Selection<'_> {
        Selection {
            path: &self.config_path,
            stype: SelectionType::File,
            is_exe: false,
            line: 0,
        }
    }

    fn refresh(&mut self, _screen: Screen, _con: &AppContext) -> Command {
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
        screen: Screen,
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
            .set("version", env!("CARGO_PKG_VERSION"));
        let config_paths: Vec<String> = con.config_paths.iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        for path in &config_paths {
            expander.sub("config-files")
                .set("path", path);
        }
        let verb_rows = super::help_verbs::matching_verb_rows(&self.pattern, con);
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
        let search_rows = super::help_search_modes::search_mode_rows(con);
        for row in &search_rows {
            expander
                .sub("search-mode-rows")
                .set("search-prefix", &row.prefix)
                .set("search-type", &row.description);
        }
        let features = super::help_features::list();
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

    fn on_internal(
        &mut self,
        w: &mut W,
        internal_exec: &InternalExecution,
        input_invocation: Option<&VerbInvocation>,
        trigger_type: TriggerType,
        cc: &CmdContext,
        screen: Screen,
    ) -> Result<AppStateCmdResult, ProgramError> {
        use Internal::*;
        Ok(match internal_exec.internal {
            Internal::back => {
                if self.pattern.is_some() {
                    self.pattern = Pattern::None;
                    AppStateCmdResult::Keep
                } else {
                    AppStateCmdResult::PopState
                }
            }
            help => AppStateCmdResult::Keep,
            line_down => {
                self.scroll += get_arg(input_invocation, internal_exec, 1);
                AppStateCmdResult::Keep
            }
            line_up => {
                self.scroll -= get_arg(input_invocation, internal_exec, 1);
                AppStateCmdResult::Keep
            }
            open_stay => match open::that(&Conf::default_location()) {
                Ok(exit_status) => {
                    info!("open returned with exit_status {:?}", exit_status);
                    AppStateCmdResult::Keep
                }
                Err(e) => AppStateCmdResult::DisplayError(format!("{:?}", e)),
            },
            // FIXME check we can't use the generic one
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

