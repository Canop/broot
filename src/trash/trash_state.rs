use {
    crate::{
        app::*,
        command::*,
        display::*,
        errors::ProgramError,
        pattern::*,
        tree::TreeOptions,
        verb::*,
    },
    crokey::crossterm::{
        cursor,
        style::Color,
        QueueableCommand,
    },
    std::{
        path::Path,
    },
    termimad::{
        minimad::Alignment,
        *,
    },
    trash::{
        self as trash_crate,
        TrashItem,
    },
    unicode_width::UnicodeWidthStr,
};

struct FilteredContent {
    pattern: Pattern,
    items: Vec<TrashItem>,
    selection_idx: Option<usize>,
}

/// an application state showing the content of the trash
pub struct TrashState {
    items: Vec<TrashItem>,
    selection_idx: Option<usize>,
    scroll: usize,
    page_height: usize,
    tree_options: TreeOptions,
    filtered: Option<FilteredContent>,
    mode: Mode,
}

impl TrashState {
    /// create a state listing the content of the system's trash
    pub fn new(
        tree_options: TreeOptions,
        con: &AppContext,
    ) -> Result<TrashState, ProgramError> {
        let items = trash::os_limited::list()
            .map_err(|e| ProgramError::Trash { message: e.to_string() })?;
        let selection_idx = None;
        Ok(TrashState {
            items,
            selection_idx,
            scroll: 0,
            page_height: 0,
            tree_options,
            filtered: None,
            mode: con.initial_mode(),
        })
    }
    pub fn count(&self) -> usize {
        self.filtered
            .as_ref()
            .map(|f| f.items.len())
            .unwrap_or_else(|| self.items.len().into())
    }
    pub fn try_scroll(
        &mut self,
        cmd: ScrollCommand,
    ) -> bool {
        let old_scroll = self.scroll;
        self.scroll = cmd.apply(self.scroll, self.count(), self.page_height);
        // move selection to an item in view
        if let Some(f) = self.filtered.as_mut() {
            if let Some(idx) = f.selection_idx {
                if idx < self.scroll {
                    f.selection_idx = Some(self.scroll);
                } else if idx >= self.scroll + self.page_height {
                    f.selection_idx = Some(self.scroll + self.page_height - 1);
                }
            }
        } else {
            if let Some(idx) = self.selection_idx {
                if idx < self.scroll {
                    self.selection_idx = Some(self.scroll);
                } else if idx >= self.scroll + self.page_height {
                    self.selection_idx = Some(self.scroll + self.page_height - 1);
                }
            }
        }
        self.scroll != old_scroll
    }
    /// If there's a selection, adjust the scroll to make it visible
    pub fn show_selection(&mut self) {
        let selection_idx = if let Some(f) = self.filtered.as_ref() {
            f.selection_idx
        } else {
            self.selection_idx
        };
        if let Some(idx) = selection_idx {
            if idx < self.scroll {
                self.scroll = idx;
            } else if idx >= self.scroll + self.page_height {
                self.scroll = idx - self.page_height + 1;
            }
        }
    }

    /// change the selection
    fn move_line(
        &mut self,
        internal_exec: &InternalExecution,
        input_invocation: Option<&VerbInvocation>,
        dir: i32, // -1 for up, 1 for down
        cycle: bool,
    ) -> CmdResult {
        let count = get_arg(input_invocation, internal_exec, 1);
        let dec = dir * count;
        let selection_idx;
        if let Some(f) = self.filtered.as_mut() {
            selection_idx = if let Some(idx) = f.selection_idx {
                Some(move_sel(idx, f.items.len(), dec, cycle))
            } else if !f.items.is_empty() {
                Some(if dec > 0 { 0 } else { f.items.len() - 1 })
            } else {
                None
            };
            f.selection_idx = selection_idx;
        } else {
            selection_idx = if let Some(idx) = self.selection_idx {
                Some(move_sel(idx, self.items.len(), dec, cycle))
            } else if !self.items.is_empty() {
                Some(if dec > 0 { 0 } else { self.items.len() - 1 })
            } else {
                None
            };
            self.selection_idx = selection_idx;
        }
        if let Some(selection_idx) = selection_idx {
            if selection_idx < self.scroll {
                self.scroll = selection_idx;
            } else if selection_idx >= self.scroll + self.page_height {
                self.scroll = selection_idx + 1 - self.page_height;
            }
        }
        CmdResult::Keep
    }

    fn selected_item(&self) -> Option<&TrashItem> {
        if let Some(f) = self.filtered.as_ref() {
            f.selection_idx.map(|idx| &f.items[idx])
        } else {
            self.selection_idx.map(|idx| &self.items[idx])
        }
    }

    fn take_selected_item(&mut self) -> Option<TrashItem> {
        if let Some(f) = self.filtered.as_mut() {
            if let Some(idx) = f.selection_idx {
                let item = f.items.remove(idx);
                if f.items.is_empty() {
                    f.selection_idx = None;
                } else if idx == f.items.len() {
                    f.selection_idx = Some(idx - 1);
                }
                Some(item)
            } else {
                None
            }
        } else {
            if let Some(idx) = self.selection_idx {
                let item = self.items.remove(idx);
                if self.items.is_empty() {
                    self.selection_idx = None;
                } else if idx == self.items.len() {
                    self.selection_idx = Some(idx - 1);
                }
                Some(item)
            } else {
                None
            }
        }
    }
}

impl PanelState for TrashState {
    fn get_type(&self) -> PanelStateType {
        PanelStateType::Trash
    }

    fn set_mode(
        &mut self,
        mode: Mode,
    ) {
        self.mode = mode;
    }

    fn get_mode(&self) -> Mode {
        self.mode
    }

    /// We don't want to expose path to verbs because you can't
    /// normally access files in the trash
    fn selected_path(&self) -> Option<&Path> {
        None
    }

    fn tree_options(&self) -> TreeOptions {
        self.tree_options.clone()
    }

    fn with_new_options(
        &mut self,
        _screen: Screen,
        change_options: &dyn Fn(&mut TreeOptions) -> &'static str,
        _in_new_panel: bool, // TODO open tree if true
        _con: &AppContext,
    ) -> CmdResult {
        change_options(&mut self.tree_options);
        CmdResult::Keep
    }

    /// We don't want to expose path to verbs because you can't
    /// normally access files in the trash
    fn selection(&self) -> Option<Selection<'_>> {
        None
    }

    fn refresh(
        &mut self,
        _screen: Screen,
        _con: &AppContext,
    ) -> Command {
        // minimal implementation. It would be better to keep filtering, and
        // also selection & scroll whenever possible
        if let Ok(items) = trash::os_limited::list() {
            self.items = items;
            self.selection_idx = None;
            self.scroll = 0;
        }
        Command::empty()
    }

    fn on_pattern(
        &mut self,
        pattern: InputPattern,
        _app_state: &AppState,
        _con: &AppContext,
    ) -> Result<CmdResult, ProgramError> {
        if pattern.is_none() {
            if let Some(f) = self.filtered.take() {
                if let Some(idx) = f.selection_idx {
                    self.selection_idx = self.items
                        .iter()
                        .position(|m| m.id == f.items[idx].id);
                }
            }
        } else {
            let pattern = pattern.pattern;
            let mut best_score = 0;
            let mut selection_idx = None;
            let mut items = Vec::new();
            for item in &self.items {
                let score = pattern.score_of_string(&item.name).unwrap_or(0)
                    + pattern
                        .score_of_string(&item.original_parent.to_string_lossy())
                        .unwrap_or(0);
                if score > 0 {
                    items.push(item.clone());
                    if score > best_score {
                        best_score = score;
                        selection_idx = Some(items.len() - 1);
                    }
                }
            }
            self.filtered = Some(FilteredContent {
                pattern,
                items,
                selection_idx,
            });
        }
        self.show_selection();
        Ok(CmdResult::Keep)
    }

    fn display(
        &mut self,
        w: &mut W,
        disc: &DisplayContext,
    ) -> Result<(), ProgramError> {
        let title_parent = "Original parent";
        let title_name = "Deleted file name";
        let area = &disc.state_area;
        let con = &disc.con;
        self.page_height = area.height as usize - 2;
        let (items, selection_idx) = if let Some(filtered) = &self.filtered {
            (filtered.items.as_slice(), filtered.selection_idx)
        } else {
            (self.items.as_slice(), self.selection_idx)
        };
        let scrollbar = area.scrollbar(self.scroll, items.len());
        //- style preparation
        let styles = &disc.panel_skin.styles;
        let selection_bg = styles
            .selected_line
            .get_bg()
            .unwrap_or(Color::AnsiValue(240));
        let match_style = &styles.char_match;
        let mut selected_match_style = styles.char_match.clone();
        selected_match_style.set_bg(selection_bg);
        let border_style = &styles.help_table_border;
        let mut selected_border_style = styles.help_table_border.clone();
        selected_border_style.set_bg(selection_bg);
        //- width computations
        let width = area.width as usize;
        let optimal_parent_width = items
            .iter()
            .map(|m| m.original_parent.to_string_lossy().width())
            .max()
            .unwrap_or(0)
            .max(title_parent.len());
        let optimal_name_width = items
            .iter()
            .map(|m| m.name.width())
            .max()
            .unwrap_or(0)
            .max(title_name.len());
        let available_width = if con.show_selection_mark {
            width - 1
        } else {
            width
        };
        let mut w_parent = optimal_parent_width;
        let mut w_name = optimal_name_width;
        if w_name + w_parent > available_width {
            w_name = (width * 2 / 3).min(optimal_name_width);
            w_parent = width - w_name;
        }
        info!("optimal_parent_width: {}, optimal_name_width: {}", optimal_parent_width, optimal_name_width);
        info!("available_width: {}, w_parent: {}, w_name: {}", available_width, w_parent, w_name);
        //- titles
        w.queue(cursor::MoveTo(area.left, area.top))?;
        let mut cw = CropWriter::new(w, width);
        if con.show_selection_mark {
            cw.queue_char(&styles.default, ' ')?;
        }
        let title = if title_parent.len() > w_parent {
            &title_parent[..w_parent]
        } else {
            title_parent
        };
        cw.queue_g_string(&styles.default, format!("{:^w_parent$}", title))?;
        cw.queue_char(border_style, '│')?;
        let title = if title_name.len() > w_name {
            &title_name[..w_name]
        } else {
            title_name
        };
        cw.queue_g_string(&styles.default, format!("{:^w_name$}", title))?;
        cw.fill(border_style, &SPACE_FILLING)?;
        //- horizontal line
        w.queue(cursor::MoveTo(area.left, 1 + area.top))?;
        let mut cw = CropWriter::new(w, width);
        if con.show_selection_mark {
            cw.queue_char(&styles.default, ' ')?;
        }
        cw.queue_g_string(border_style, format!("{:─>width$}", '┼', width = w_parent + 1))?;
        cw.fill(border_style, &BRANCH_FILLING)?;
        //- content
        let mut idx = self.scroll;
        for y in 2..area.height {
            w.queue(cursor::MoveTo(area.left, y + area.top))?;
            let selected = selection_idx == Some(idx);
            let mut cw = CropWriter::new(w, width - 1); // -1 for scrollbar
            let txt_style = if selected {
                &styles.selected_line
            } else {
                &styles.default
            };
            if let Some(item) = items.get(idx) {
                let match_style = if selected {
                    &selected_match_style
                } else {
                    match_style
                };
                let border_style = if selected {
                    &selected_border_style
                } else {
                    border_style
                };
                if con.show_selection_mark {
                    cw.queue_char(txt_style, if selected { '▶' } else { ' ' })?;
                }
                // parent
                let s = item.original_parent.to_string_lossy();
                let mut matched_string = MatchedString::new(
                    self.filtered
                        .as_ref()
                        .and_then(|f| f.pattern.search_string(&s)),
                    &s,
                    txt_style,
                    match_style,
                );
                if s.width() > w_parent {
                    //info!("CUT w_parent: {}, s.width(): {}", w_parent, s.width());
                    cw.queue_char(txt_style, '…')?;
                    matched_string.cut_left_to_fit(w_parent - 1);
                    //info!(" cut string width: {}", matched_string.string.width());
                    matched_string.queue_on(&mut cw)?;
                } else {
                    matched_string.fill(w_parent, Alignment::Left);
                    matched_string.queue_on(&mut cw)?;
                }
                cw.queue_char(border_style, '│')?;
                // name
                let s = &item.name;
                let mut matched_string = MatchedString::new(
                    self.filtered
                        .as_ref()
                        .and_then(|f| f.pattern.search_string(s)),
                    s,
                    txt_style,
                    match_style,
                );
                matched_string.fill(w_name, Alignment::Left);
                matched_string.queue_on(&mut cw)?;
                idx += 1;
            } else {
                if con.show_selection_mark {
                    cw.queue_char(&styles.default, ' ')?;
                }
                cw.queue_g_string(border_style, format!("{: >width$}", '│', width = w_parent + 1))?;
            }
            cw.fill(txt_style, &SPACE_FILLING)?;
            let scrollbar_style = if ScrollCommand::is_thumb(y, scrollbar) {
                &styles.scrollbar_thumb
            } else {
                &styles.scrollbar_track
            };
            scrollbar_style.queue_str(w, "▐")?;
        }
        Ok(())
    }

    fn on_internal(
        &mut self,
        w: &mut W,
        invocation_parser: Option<&InvocationParser>,
        internal_exec: &InternalExecution,
        input_invocation: Option<&VerbInvocation>,
        trigger_type: TriggerType,
        app_state: &mut AppState,
        cc: &CmdContext,
    ) -> Result<CmdResult, ProgramError> {
        use Internal::*;
        Ok(match internal_exec.internal {
            Internal::restore_trashed_file => {
                if let Some(item) = self.selected_item() {
                    match trash_crate::os_limited::restore_all([item.clone()]) {
                        Ok(_) => {
                            let path = item.original_path();
                            self.take_selected_item();
                            CmdResult::Message(format!(
                                "File *{}* restored",
                                path.to_string_lossy(),
                            ))
                        }
                        Err(trash_crate::Error::RestoreCollision { path, .. }) => {
                            CmdResult::DisplayError(format!(
                                "collision: *{}* already exists",
                                path.to_string_lossy(),
                            ))
                        }
                        Err(e) => {
                            CmdResult::DisplayError(format!(
                                "restore failed: {}",
                                e.to_string(),
                            ))
                        }
                    }
                } else {
                    CmdResult::DisplayError(
                        "an item must be selected".to_string(),
                    )
                }
            }
            Internal::delete_trashed_file => {
                if let Some(item) = self.selected_item() {
                    match trash_crate::os_limited::purge_all([item.clone()]) {
                        Ok(_) => {
                            let path = item.original_path();
                            self.take_selected_item();
                            CmdResult::Message(format!(
                                "File *{}* restored",
                                path.to_string_lossy(),
                            ))
                        }
                        Err(e) => {
                            CmdResult::DisplayError(format!(
                                "deletion failed: {}",
                                e.to_string(),
                            ))
                        }
                    }
                } else {
                    CmdResult::DisplayError(
                        "an item must be selected".to_string(),
                    )
                }
            }
            Internal::back => {
                if let Some(f) = self.filtered.take() {
                    if let Some(idx) = f.selection_idx {
                        self.selection_idx = self.items
                            .iter()
                            .position(|m| m.id == f.items[idx].id);
                    }
                    self.show_selection();
                    CmdResult::Keep
                } else {
                    CmdResult::PopState
                }
            }
            Internal::line_down => self.move_line(internal_exec, input_invocation, 1, true),
            Internal::line_up => self.move_line(internal_exec, input_invocation, -1, true),
            Internal::line_down_no_cycle => {
                self.move_line(internal_exec, input_invocation, 1, false)
            }
            Internal::line_up_no_cycle => {
                self.move_line(internal_exec, input_invocation, -1, false)
            }
            Internal::open_stay => {
                // it would probably be a good idea to bind enter to restore_trash_file ?
                CmdResult::DisplayError("can't open a file from the trash".to_string())
            }
            Internal::panel_left_no_open => CmdResult::HandleInApp(Internal::panel_left_no_open),
            Internal::panel_right_no_open => CmdResult::HandleInApp(Internal::panel_right_no_open),
            Internal::page_down => {
                if !self.try_scroll(ScrollCommand::Pages(1)) {
                    self.selection_idx = Some(self.count() - 1);
                }
                CmdResult::Keep
            }
            Internal::page_up => {
                if !self.try_scroll(ScrollCommand::Pages(-1)) {
                    self.selection_idx = Some(0);
                }
                CmdResult::Keep
            }
            open_leave => CmdResult::PopStateAndReapply,
            _ => self.on_internal_generic(
                w,
                invocation_parser,
                internal_exec,
                input_invocation,
                trigger_type,
                app_state,
                cc,
            )?,
        })
    }

    fn on_click(
        &mut self,
        _x: u16,
        y: u16,
        _screen: Screen,
        _con: &AppContext,
    ) -> Result<CmdResult, ProgramError> {
        if y >= 2 {
            let y = y as usize - 2 + self.scroll;
            if y < self.items.len().into() {
                self.selection_idx = Some(y);
            }
        }
        Ok(CmdResult::Keep)
    }
}
