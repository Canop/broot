use {
    super::{
        help_content,
        SearchModeHelp,
    },
    crate::{
        app::*,
        command::{Command, TriggerType},
        conf::Conf,
        display::{Screen, W},
        errors::ProgramError,
        launchable::Launchable,
        pattern::*,
        tree::TreeOptions,
        verb::*,
    },
    std::path::{Path, PathBuf},
    termimad::{Area, FmtText, TextView},
};

/// an application state dedicated to help
pub struct HelpState {
    pub scroll: usize,
    pub text_area: Area,
    dirty: bool, // background must be cleared
    pattern: Pattern,
    tree_options: TreeOptions,
    config_path: PathBuf, // the last config path when several were used
    mode: Mode,
}

impl HelpState {
    pub fn new(
        tree_options: TreeOptions,
        _screen: Screen,
        con: &AppContext,
    ) -> HelpState {
        let text_area = Area::uninitialized(); // will be fixed at drawing time
        let config_path = con.config_paths
            .first()
            .cloned()
            .unwrap_or_else(Conf::default_location);
        HelpState {
            text_area,
            scroll: 0,
            dirty: true,
            pattern: Pattern::None,
            tree_options,
            config_path,
            mode: con.initial_mode(),
        }
    }
}

impl PanelState for HelpState {

    fn get_type(&self) -> PanelStateType {
        PanelStateType::Help
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    fn get_mode(&self) -> Mode {
        self.mode
    }

    fn selected_path(&self) -> Option<&Path> {
        Some(&self.config_path)
    }

    fn tree_options(&self) -> TreeOptions {
        self.tree_options.clone()
    }

    fn with_new_options(
        &mut self,
        _screen: Screen,
        change_options: &dyn Fn(&mut TreeOptions) -> &'static str,
        _in_new_panel: bool, // TODO open a tree if true
        _con: &AppContext,
    ) -> CmdResult {
        change_options(&mut self.tree_options);
        CmdResult::Keep
    }

    fn selection(&self) -> Option<Selection<'_>> {
        Some(Selection {
            path: &self.config_path,
            stype: SelectionType::File,
            is_exe: false,
            line: 0,
        })
    }

    fn refresh(&mut self, _screen: Screen, _con: &AppContext) -> Command {
        self.dirty = true;
        Command::empty()
    }

    fn on_pattern(
        &mut self,
        pat: InputPattern,
        _app_state: &AppState,
        _con: &AppContext,
    ) -> Result<CmdResult, ProgramError> {
        self.pattern = pat.pattern;
        Ok(CmdResult::Keep)
    }

    fn display(
        &mut self,
        w: &mut W,
        disc: &DisplayContext,
    ) -> Result<(), ProgramError> {
        let con = &disc.con;
        let mut text_area = disc.state_area.clone();
        text_area.pad_for_max_width(120);
        if text_area != self.text_area {
            self.dirty = true;
            self.text_area = text_area;
        }
        if self.dirty {
            disc.panel_skin.styles.default.queue_bg(w)?;
            disc.screen.clear_area_to_right(w, &disc.state_area)?;
            self.dirty = false;
        }
        let mut expander = help_content::expander();
        expander.set("version", env!("CARGO_PKG_VERSION"));
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
                .set("key", &row.keys_desc);
            if row.verb.description.code {
                sub.set("description", "");
                sub.set("execution", &row.verb.description.content);
            } else {
                sub.set_md("description", &row.verb.description.content);
                sub.set("execution", "");
            }
        }
        let mode_help;
        if let Ok(default_mode) = con.search_modes.search_mode(None) {
            mode_help = super::search_mode_help(default_mode, con);
            expander
                .sub("default-search")
                .set_md("default-search-example", &mode_help.example);
        }
        let search_rows: Vec<SearchModeHelp> = SEARCH_MODES
            .iter()
            .map(|mode| super::search_mode_help(*mode, con))
            .collect();
        for row in &search_rows {
            expander
                .sub("search-mode-rows")
                .set("search-prefix", &row.prefix)
                .set("search-type", &row.description)
                .set_md("search-example", &row.example);
        }
        let nr_prefix = SearchMode::NameRegex.prefix(con);
        let ce_prefix = SearchMode::ContentExact.prefix(con);
        expander
            .set("nr-prefix", &nr_prefix)
            .set("ce-prefix", &ce_prefix);
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
            expander
                .sub("features")
                .set("feature-name", feature.0)
                .set("feature-description", feature.1);
        }
        let text = expander.expand();
        let fmt_text = FmtText::from_text(
            &disc.panel_skin.help_skin,
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
        app_state: &mut AppState,
        cc: &CmdContext,
    ) -> Result<CmdResult, ProgramError> {
        use Internal::*;
        Ok(match internal_exec.internal {
            Internal::back => {
                if self.pattern.is_some() {
                    self.pattern = Pattern::None;
                    CmdResult::Keep
                } else {
                    CmdResult::PopState
                }
            }
            help => CmdResult::Keep,
            line_down | line_down_no_cycle => {
                self.scroll += get_arg(input_invocation, internal_exec, 1);
                CmdResult::Keep
            }
            line_up | line_up_no_cycle => {
                let dy = get_arg(input_invocation, internal_exec, 1);
                self.scroll = if self.scroll > dy {
                    self.scroll - dy
                } else {
                    0
                };
                CmdResult::Keep
            }
            open_stay => match opener::open(Conf::default_location()) {
                Ok(exit_status) => {
                    info!("open returned with exit_status {:?}", exit_status);
                    CmdResult::Keep
                }
                Err(e) => CmdResult::DisplayError(format!("{e:?}")),
            },
            // FIXME check we can't use the generic one
            open_leave => {
                CmdResult::from(Launchable::opener(
                    Conf::default_location()
                ))
            }
            page_down => {
                self.scroll += self.text_area.height as usize;
                CmdResult::Keep
            }
            page_up => {
                let height = self.text_area.height as usize;
                self.scroll = if self.scroll > height {
                    self.scroll - self.text_area.height as usize
                } else {
                    0
                };
                CmdResult::Keep
            }
            _ => self.on_internal_generic(
                w,
                internal_exec,
                input_invocation,
                trigger_type,
                app_state,
                cc,
            )?,
        })
    }
}

