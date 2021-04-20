use {
    super::*,
    crate::{
        app::*,
        command::{Command, TriggerType},
        display::{CropWriter, MatchedString, Screen, SPACE_FILLING, W},
        errors::ProgramError,
        pattern::*,
        skin::*,
        tree::*,
        verb::*,
    },
    crossterm::{
        cursor,
        QueueableCommand,
    },
    std::path::{Path},
    termimad::Area,
    unicode_width::{UnicodeWidthChar, UnicodeWidthStr},
};

static TITLE: &str = "Staging Area"; // no wide char allowed here
static ELLIPSIS: char = '…';

pub struct StageState {

    filtered_stage: FilteredStage,

    /// those options are mainly kept for transmission to child state
    /// (if they become possible)
    tree_options: TreeOptions,

    /// the 'modal' mode
    mode: Mode,
}

impl StageState {

    pub fn new(
        app_state: &AppState,
        tree_options: TreeOptions,
        con: &AppContext,
    ) -> StageState {
        let filtered_stage = FilteredStage::filtered(
            &app_state.stage,
            tree_options.pattern.clone(),
        );
        Self {
            filtered_stage,
            tree_options,
            mode: initial_mode(con),
        }
    }

    fn write_title_line(
        &self,
        stage: &Stage,
        cw: &mut CropWriter<'_, W>,
        styles: &StyleMap,
    ) -> Result<(), ProgramError> {
        let total_count = format!("{}", stage.len());
        let mut count_len = total_count.len();
        if self.filtered_stage.pattern().is_some() {
            count_len += total_count.len() + 1; // 1 for '/'
        }
        if cw.allowed < count_len {
            return Ok(());
        }
        if TITLE.len() + 1 + count_len <= cw.allowed {
            cw.queue_str(
                &styles.staging_area_title,
                TITLE,
            )?;
        }
        cw.repeat(&styles.staging_area_title, &SPACE_FILLING, cw.allowed - count_len)?;
        if self.filtered_stage.pattern().is_some() {
            cw.queue_g_string(
                &styles.char_match,
                format!("{}", self.filtered_stage.len()),
            )?;
            cw.queue_char(
                &styles.staging_area_title,
                '/',
            )?;
        }
        cw.queue_g_string(
            &styles.staging_area_title,
            total_count,
        )?;
        cw.fill(&styles.staging_area_title, &SPACE_FILLING)?;
        Ok(())
    }
}

impl PanelState for StageState {

    fn get_type(&self) -> PanelStateType {
        PanelStateType::Stage
    }

    fn get_pending_task(&self) -> Option<&'static str> {
        None
    }

    fn selected_path(&self) -> Option<&Path> {
        None
    }

    fn selection(&self) -> Option<Selection<'_>> {
        None
    }

    fn sel_info<'c>(&'c self, app_state: &'c AppState) -> SelInfo<'c> {
        match app_state.stage.len() {
            0 => SelInfo::None,
            1 => SelInfo::One(Selection {
                path: &app_state.stage.paths()[0],
                stype: SelectionType::File,
                is_exe: false,
                line: 0,
            }),
            _ => SelInfo::More(&app_state.stage),
        }
    }

    fn has_at_least_one_selection(&self, app_state: &AppState) -> bool {
        !app_state.stage.is_empty()
    }

    fn tree_options(&self) -> TreeOptions {
        self.tree_options.clone()
    }

    /// option changing is unlikely to be done on this state, but
    /// we'll still do it in case a future scenario makes it possible
    /// to open a different state from this state
    fn with_new_options(
        &mut self,
        _screen: Screen,
        change_options: &dyn Fn(&mut TreeOptions),
        in_new_panel: bool,
        con: &AppContext,
    ) -> CmdResult {
        if in_new_panel {
            CmdResult::error("stage can't be displayed in two panels")
        } else {
            let mut new_options= self.tree_options();
            change_options(&mut new_options);
            CmdResult::NewState(Box::new(StageState {
                filtered_stage: self.filtered_stage.clone(),
                mode: initial_mode(con),
                tree_options: new_options,
            }))
        }
    }

    fn on_pattern(
        &mut self,
        pat: InputPattern,
        app_state: &AppState,
        _con: &AppContext,
    ) -> Result<CmdResult, ProgramError> {
        self.filtered_stage = FilteredStage::filtered(&app_state.stage, pat);
        Ok(CmdResult::Keep)
    }

    fn display(
        &mut self,
        w: &mut W,
        disc: &DisplayContext,
    ) -> Result<(), ProgramError> {
        let stage = &disc.app_state.stage;
        self.filtered_stage.update(stage);
        let area = &disc.state_area;
        let styles = &disc.panel_skin.styles;
        let width = area.width as usize;
        w.queue(cursor::MoveTo(area.left, 0))?;
        let mut cw = CropWriter::new(w, width);
        self.write_title_line(stage, &mut cw, styles)?;
        let list_area = Area::new(area.left, area.top + 1, area.width, area.height - 1);
        let list_height = list_area.height as usize;
        let pattern = &self.filtered_stage.pattern().pattern;
        let pattern_object = pattern.object();
        for idx in 0..list_height {
            let y = list_area.top + idx as u16;
            let stage_idx = idx; // + scroll
            w.queue(cursor::MoveTo(area.left, y))?;
            let mut cw = CropWriter::new(w, width);
            let cw = &mut cw;
            if let Some(path) = self.filtered_stage.path(stage, stage_idx) {
                let style = if path.is_dir() {
                    &styles.directory
                } else {
                    &styles.file
                };
                if pattern_object.subpath {
                    let label = path.to_string_lossy();
                    // we must display the matching on the whole path
                    // (subpath is the path for the staging area)
                    let name_match = pattern.search_string(&label);
                    let matched_string = MatchedString::new(
                        name_match,
                        &label,
                        &style,
                        &styles.char_match,
                    );
                    matched_string.queue_on(cw)?;
                } else if let Some(file_name) = path.file_name() {
                    let label = file_name.to_string_lossy();
                    let label_cols = label.width();
                    if label_cols + 2 < cw.allowed {
                        if let Some(parent_path) = path.parent() {
                            let cols_max = cw.allowed - label_cols - 3;
                            let parent_path = parent_path.to_string_lossy();
                            let parent_cols = parent_path.width();
                            if parent_cols <= cols_max {
                                cw.queue_str(
                                    &styles.parent,
                                    &parent_path,
                                )?;
                            } else {
                                // TODO move to (crop_writer ? termimad ?)
                                // we'll compute the size of the tail fitting
                                // the width minus one (for the ellipsis)
                                let mut bytes_count = 0;
                                let mut cols_count = 0;
                                for c in parent_path.chars().rev() {
                                    let char_width = UnicodeWidthChar::width(c).unwrap_or(0);
                                    let next_str_width = cols_count + char_width;
                                    if next_str_width > cols_max {
                                        break;
                                    }
                                    cols_count = next_str_width;
                                    bytes_count += c.len_utf8();
                                }
                                cw.queue_char(
                                    &styles.parent,
                                    ELLIPSIS,
                                )?;
                                cw.queue_str(
                                    &styles.parent,
                                    &parent_path[parent_path.len()-bytes_count..],
                                )?;
                            }
                            cw.queue_char(
                                &styles.parent,
                                '/',
                            )?;
                        }
                    }
                    let name_match = pattern.search_string(&label);
                    let matched_string = MatchedString::new(
                        name_match,
                        &label,
                        &style,
                        &styles.char_match,
                    );
                    matched_string.queue_on(cw)?;
                } else {
                    // this should not happen
                    warn!("how did we fall on a path without filename?");
                }
            }
            cw.fill(&styles.default, &SPACE_FILLING)?;
        }
        Ok(())
    }

    fn refresh(&mut self, _screen: Screen, _con: &AppContext) -> Command {
        Command::empty()
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    fn get_mode(&self) -> Mode {
        self.mode
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
        Ok(match internal_exec.internal {
            Internal::back if self.filtered_stage.pattern().is_some() => {
                self.filtered_stage = FilteredStage::unfiltered(&app_state.stage);
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

    fn execute_sequence(
        &mut self,
        _w: &mut W,
        _verb: &Verb,
        _seq_ex: &SequenceExecution,
        _invocation: Option<&VerbInvocation>,
        _app_state: &mut AppState,
        _cc: &CmdContext,
    ) -> Result<CmdResult, ProgramError> {
        Ok(CmdResult::error("sequence execution not yet implemented on staging area"))
    }
}

