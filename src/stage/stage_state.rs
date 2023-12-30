use {
    super::*,
    crate::{
        app::*,
        command::*,
        display::{MatchedString, Screen, W},
        errors::ProgramError,
        pattern::*,
        skin::*,
        task_sync::Dam,
        tree::*,
        verb::*,
    },
    crokey::crossterm::{
        cursor,
        QueueableCommand,
    },
    std::path::{Path},
    termimad::{Area, CropWriter, SPACE_FILLING},
    unicode_width::{UnicodeWidthChar, UnicodeWidthStr},
};

static TITLE: &str = "Staging Area"; // no wide char allowed here
static COUNT_LABEL: &str = " count: ";
static SIZE_LABEL: &str = " size: ";
static ELLIPSIS: char = '…';

pub struct StageState {

    filtered_stage: FilteredStage,

    scroll: usize,

    tree_options: TreeOptions,

    /// the 'modal' mode
    mode: Mode,

    page_height: usize,

    stage_sum: StageSum,

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
            scroll: 0,
            tree_options,
            mode: initial_mode(con),
            page_height: 0,
            stage_sum: StageSum::default(),
        }
    }

    fn need_sum_computation(&self) -> bool {
        self.tree_options.show_sizes && !self.stage_sum.is_up_to_date()
    }


    pub fn try_scroll(
        &mut self,
        cmd: ScrollCommand,
    ) -> bool {
        let old_scroll = self.scroll;
        self.scroll = cmd.apply(self.scroll, self.filtered_stage.len(), self.page_height);
        self.scroll != old_scroll
    }

    pub fn fix_scroll(&mut self) {
        let len = self.filtered_stage.len();
        if self.scroll + self.page_height > len {
            self.scroll = if len > self.page_height {
                len - self.page_height
            } else {
                0
            };
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
        let mut show_count_label = false;
        let mut rem = cw.allowed - count_len;
        if COUNT_LABEL.len() < rem {
            rem -= COUNT_LABEL.len();
            show_count_label = true;
            if self.tree_options.show_sizes {
                if let Some(sum) = self.stage_sum.computed() {
                    let size = file_size::fit_4(sum.to_size());
                    let size_len = SIZE_LABEL.len() + size.len();
                    if size_len < rem {
                        rem -= size_len;
                        // we display the size in the middle, so we cut rem in two
                        let left_rem  = rem / 2;
                        rem -= left_rem;
                        cw.repeat(&styles.staging_area_title, &SPACE_FILLING, left_rem)?;
                        cw.queue_g_string(
                            &styles.staging_area_title,
                            SIZE_LABEL.to_string(),
                        )?;
                        cw.queue_g_string(
                            &styles.staging_area_title,
                            size,
                        )?;
                    }
                }
            }
        }
        cw.repeat(&styles.staging_area_title, &SPACE_FILLING, rem)?;
        if show_count_label {
            cw.queue_g_string(
                &styles.staging_area_title,
                COUNT_LABEL.to_string(),
            )?;
        }
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

    fn move_selection(&mut self, dy: i32, cycle: bool) -> CmdResult {
        self.filtered_stage.move_selection(dy, cycle);
        if let Some(sel) = self.filtered_stage.selection() {
            if sel < self.scroll + 5 {
                self.scroll = (sel as i32 -5).max(0) as usize;
            } else if sel > self.scroll + self.page_height - 5 {
                self.scroll = (sel + 5 - self.page_height)
                    .min(self.filtered_stage.len() - self.page_height);
            }
        }
        CmdResult::Keep
    }

}

impl PanelState for StageState {

    fn get_type(&self) -> PanelStateType {
        PanelStateType::Stage
    }

    fn selected_path(&self) -> Option<&Path> {
        None
    }

    fn selection(&self) -> Option<Selection<'_>> {
        None
    }

    fn clear_pending(&mut self) {
        self.stage_sum.clear();
    }
    fn do_pending_task(
        &mut self,
        app_state: &mut AppState,
        _screen: Screen,
        con: &AppContext,
        dam: &mut Dam,
        // need the stage here
    ) -> Result<(), ProgramError> {
        if self.need_sum_computation() {
            self.stage_sum.compute(&app_state.stage, dam, con);
        }
        Ok(())
    }
    fn get_pending_task(&self) -> Option<&'static str> {
        if self.need_sum_computation() {
            Some("stage size summing")
        } else {
            None
        }
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
        change_options: &dyn Fn(&mut TreeOptions) -> &'static str,
        in_new_panel: bool,
        con: &AppContext,
    ) -> CmdResult {
        if in_new_panel {
            CmdResult::error("stage can't be displayed in two panels")
        } else {
            let mut new_options= self.tree_options();
            let message = change_options(&mut new_options);
            let state = Box::new(StageState {
                filtered_stage: self.filtered_stage.clone(),
                scroll: self.scroll,
                mode: initial_mode(con),
                tree_options: new_options,
                page_height: self.page_height,
                stage_sum: self.stage_sum,
            });
            CmdResult::NewState { state, message: Some(message) }
        }
    }

    fn on_click(
        &mut self,
        _x: u16,
        y: u16,
        _screen: Screen,
        _con: &AppContext,
    ) -> Result<CmdResult, ProgramError> {
        if y > 0 {
            // the list starts on the second row
            self.filtered_stage.try_select_idx(y as usize - 1 + self.scroll);
        }
        Ok(CmdResult::Keep)
    }

    fn on_pattern(
        &mut self,
        pat: InputPattern,
        app_state: &AppState,
        _con: &AppContext,
    ) -> Result<CmdResult, ProgramError> {
        self.filtered_stage.set_pattern(&app_state.stage, pat);
        self.fix_scroll();
        Ok(CmdResult::Keep)
    }

    fn display(
        &mut self,
        w: &mut W,
        disc: &DisplayContext,
    ) -> Result<(), ProgramError> {
        let stage = &disc.app_state.stage;
        self.stage_sum.see_stage(stage); // this may invalidate the sum
        if self.filtered_stage.update(stage) {
            self.fix_scroll();
        }
        let area = &disc.state_area;
        let styles = &disc.panel_skin.styles;
        let width = area.width as usize;
        w.queue(cursor::MoveTo(area.left, 0))?;
        let mut cw = CropWriter::new(w, width);
        self.write_title_line(stage, &mut cw, styles)?;
        let list_area = Area::new(area.left, area.top + 1, area.width, area.height - 1);
        self.page_height = list_area.height as usize;
        let pattern = &self.filtered_stage.pattern().pattern;
        let pattern_object = pattern.object();
        let scrollbar = list_area.scrollbar(self.scroll, self.filtered_stage.len());
        for idx in 0..self.page_height {
            let y = list_area.top + idx as u16;
            let stage_idx = idx + self.scroll;
            w.queue(cursor::MoveTo(area.left, y))?;
            let mut cw = CropWriter::new(w, width - 1);
            let cw = &mut cw;
            if let Some((path, selected)) = self.filtered_stage.path_sel(stage, stage_idx) {
                let mut style = if path.is_dir() {
                    &styles.directory
                } else {
                    &styles.file
                };
                let mut bg_style;
                if selected {
                    bg_style = style.clone();
                    if let Some(c) = styles.selected_line.get_bg() {
                        bg_style.set_bg(c);
                    }
                    style = &bg_style;
                }
                let mut bg_style_match;
                let mut style_match = &styles.char_match;
                if selected {
                    bg_style_match = style_match.clone();
                    if let Some(c) = styles.selected_line.get_bg() {
                        bg_style_match.set_bg(c);
                    }
                    style_match = &bg_style_match;
                }
                if disc.con.show_selection_mark && self.filtered_stage.has_selection() {
                    cw.queue_char(style, if selected { '▶' } else { ' ' })?;
                }
                if pattern_object.subpath {
                    let label = path.to_string_lossy();
                    // we must display the matching on the whole path
                    // (subpath is the path for the staging area)
                    let name_match = pattern.search_string(&label);
                    let matched_string = MatchedString::new(
                        name_match,
                        &label,
                        style,
                        style_match,
                    );
                    matched_string.queue_on(cw)?;
                } else if let Some(file_name) = path.file_name() {
                    let label = file_name.to_string_lossy();
                    let label_cols = label.width();
                    if label_cols + 2 < cw.allowed {
                        if let Some(parent_path) = path.parent() {
                            let mut parent_style = &styles.parent;
                            let mut bg_style;
                            if selected {
                                bg_style = parent_style.clone();
                                if let Some(c) = styles.selected_line.get_bg() {
                                    bg_style.set_bg(c);
                                }
                                parent_style = &bg_style;
                            }
                            let cols_max = cw.allowed - label_cols - 3;
                            let parent_path = parent_path.to_string_lossy();
                            let parent_cols = parent_path.width();
                            if parent_cols <= cols_max {
                                cw.queue_str(
                                    parent_style,
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
                                    parent_style,
                                    ELLIPSIS,
                                )?;
                                cw.queue_str(
                                    parent_style,
                                    &parent_path[parent_path.len()-bytes_count..],
                                )?;
                            }
                            cw.queue_char(
                                parent_style,
                                '/',
                            )?;
                        }
                    }
                    let name_match = pattern.search_string(&label);
                    let matched_string = MatchedString::new(
                        name_match,
                        &label,
                        style,
                        style_match,
                    );
                    matched_string.queue_on(cw)?;
                } else {
                    // this should not happen
                    warn!("how did we fall on a path without filename?");
                }
                cw.fill(style, &SPACE_FILLING)?;
            }
            cw.fill(&styles.default, &SPACE_FILLING)?;
            let scrollbar_style = if ScrollCommand::is_thumb(y, scrollbar) {
                &styles.scrollbar_thumb
            } else {
                &styles.scrollbar_track
            };
            scrollbar_style.queue_str(w, "▐")?;
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
            Internal::back if self.filtered_stage.has_selection() => {
                self.filtered_stage.unselect();
                CmdResult::Keep
            }
            Internal::line_down => {
                let count = get_arg(input_invocation, internal_exec, 1);
                self.move_selection(count, true)
            }
            Internal::line_up => {
                let count = get_arg(input_invocation, internal_exec, 1);
                self.move_selection(-count, true)
            }
            Internal::line_down_no_cycle => {
                let count = get_arg(input_invocation, internal_exec, 1);
                self.move_selection(count, false)
            }
            Internal::line_up_no_cycle => {
                let count = get_arg(input_invocation, internal_exec, 1);
                self.move_selection(-count, false)
            }
            Internal::page_down => {
                self.try_scroll(ScrollCommand::Pages(1));
                CmdResult::Keep
            }
            Internal::page_up => {
                self.try_scroll(ScrollCommand::Pages(-1));
                CmdResult::Keep
            }
            Internal::stage => {
                // shall we restage what we just unstaged ?
                CmdResult::error("nothing to stage here")
            }
            Internal::unstage | Internal::toggle_stage => {
                if self.filtered_stage.unstage_selection(&mut app_state.stage) {
                    CmdResult::Keep
                } else {
                    CmdResult::error("you must select a path to unstage")
                }
            }
            Internal::trash => {
                info!("trash {} staged files", app_state.stage.len());
                match trash::delete_all(app_state.stage.paths()) {
                    Ok(()) => CmdResult::RefreshState { clear_cache: true },
                    Err(e) => {
                        warn!("trash error: {:?}", &e);
                        CmdResult::DisplayError(format!("trash error: {:?}", &e))
                    }
                }
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

