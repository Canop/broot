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
    crokey::crossterm::{QueueableCommand, cursor},
    std::path::{Path, PathBuf},
    termimad::{Area, CropWriter, SPACE_FILLING},
    unicode_width::{UnicodeWidthChar, UnicodeWidthStr},
};

static TITLE: &str = "Favorites";
static COUNT_LABEL: &str = " count: ";
static ELLIPSIS: char = '…';

pub struct FavoriteState {
    filtered_favorites: FilteredFavorites,
    scroll: usize,
    tree_options: TreeOptions,
    mode: Mode,
    page_height: usize,
    /// cached copy of the currently selected path, updated during display
    selected_path: Option<PathBuf>,
}

impl FavoriteState {
    pub fn new(
        app_state: &AppState,
        tree_options: TreeOptions,
        con: &AppContext,
    ) -> FavoriteState {
        let filtered_favorites =
            FilteredFavorites::filtered(&app_state.favorites, tree_options.pattern.clone());
        Self {
            filtered_favorites,
            scroll: 0,
            tree_options,
            mode: con.initial_mode(),
            page_height: 0,
            selected_path: None,
        }
    }

    pub fn try_scroll(&mut self, cmd: ScrollCommand) -> bool {
        let old_scroll = self.scroll;
        self.scroll = cmd.apply(self.scroll, self.filtered_favorites.len(), self.page_height);
        self.scroll != old_scroll
    }

    pub fn fix_scroll(&mut self) {
        let len = self.filtered_favorites.len();
        if self.scroll + self.page_height > len {
            self.scroll = len.saturating_sub(self.page_height);
        }
    }

    fn write_title_line(
        &self,
        favorites: &Favorites,
        cw: &mut CropWriter<'_, W>,
        styles: &StyleMap,
    ) -> Result<(), ProgramError> {
        let total_count = format!("{}", favorites.len());
        let mut count_len = total_count.len();
        if self.filtered_favorites.pattern().is_some() {
            count_len += total_count.len() + 1;
        }
        if cw.allowed < count_len {
            return Ok(());
        }
        if TITLE.len() + 1 + count_len <= cw.allowed {
            cw.queue_str(&styles.staging_area_title, TITLE)?;
        }
        let mut show_count_label = false;
        let rem = cw.allowed - count_len;
        if COUNT_LABEL.len() < rem {
            show_count_label = true;
        }
        let fill = if show_count_label {
            rem - COUNT_LABEL.len()
        } else {
            rem
        };
        cw.repeat(&styles.staging_area_title, &SPACE_FILLING, fill)?;
        if show_count_label {
            cw.queue_g_string(&styles.staging_area_title, COUNT_LABEL.to_string())?;
        }
        if self.filtered_favorites.pattern().is_some() {
            cw.queue_g_string(
                &styles.char_match,
                format!("{}", self.filtered_favorites.len()),
            )?;
            cw.queue_char(&styles.staging_area_title, '/')?;
        }
        cw.queue_g_string(&styles.staging_area_title, total_count)?;
        cw.fill(&styles.staging_area_title, &SPACE_FILLING)?;
        Ok(())
    }

    fn move_selection(&mut self, dy: i32, cycle: bool) -> CmdResult {
        self.filtered_favorites.move_selection(dy, cycle);
        if let Some(sel) = self.filtered_favorites.selection() {
            if sel < self.scroll + 5 {
                self.scroll = (sel as i32 - 5).max(0) as usize;
            } else if self.page_height > 0 && sel > self.scroll + self.page_height - 5 {
                self.scroll = (sel + 5 - self.page_height)
                    .min(self.filtered_favorites.len().saturating_sub(self.page_height));
            }
        }
        CmdResult::Keep
    }

    /// Update the cached `selected_path` from the current selection.
    /// Must be called after any operation that changes the selection.
    fn update_selected_path(&mut self, favorites: &Favorites) {
        self.selected_path = self
            .filtered_favorites
            .selected_path(favorites)
            .map(|p| p.to_path_buf());
    }
}

impl PanelState for FavoriteState {
    fn get_type(&self) -> PanelStateType {
        PanelStateType::Favorite
    }

    fn selected_path(&self) -> Option<&Path> {
        self.selected_path.as_deref()
    }

    fn selection(&self) -> Option<Selection<'_>> {
        self.selected_path().map(|path| Selection {
            path,
            line: 0,
            stype: SelectionType::from(path),
            is_exe: false,
        })
    }

    fn clear_pending(&mut self) {}

    fn do_pending_task(
        &mut self,
        _app_state: &mut AppState,
        _screen: Screen,
        _con: &AppContext,
        _dam: &mut Dam,
    ) -> Result<(), ProgramError> {
        Ok(())
    }

    fn get_pending_task(&self) -> Option<&'static str> {
        None
    }

    fn sel_info<'c>(&'c self, _app_state: &'c AppState) -> SelInfo<'c> {
        match &self.selected_path {
            Some(path) => SelInfo::One(Selection {
                path,
                line: 0,
                stype: SelectionType::from(path.as_path()),
                is_exe: false,
            }),
            None => SelInfo::None,
        }
    }

    fn has_at_least_one_selection(&self, _app_state: &AppState) -> bool {
        self.selected_path.is_some()
    }

    fn tree_options(&self) -> TreeOptions {
        self.tree_options.clone()
    }

    fn with_new_options(
        &mut self,
        _screen: Screen,
        change_options: &dyn Fn(&mut TreeOptions) -> &'static str,
        in_new_panel: bool,
        con: &AppContext,
    ) -> CmdResult {
        if in_new_panel {
            CmdResult::error("favorites can't be displayed in two panels")
        } else {
            let mut new_options = self.tree_options();
            let message = change_options(&mut new_options);
            let state = Box::new(FavoriteState {
                filtered_favorites: self.filtered_favorites.clone(),
                scroll: self.scroll,
                mode: con.initial_mode(),
                tree_options: new_options,
                page_height: self.page_height,
                selected_path: self.selected_path.clone(),
            });
            CmdResult::NewState {
                state,
                message: Some(message),
            }
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
            self.filtered_favorites
                .try_select_idx(y as usize - 1 + self.scroll);
        }
        Ok(CmdResult::Keep)
    }

    fn on_pattern(
        &mut self,
        pat: InputPattern,
        app_state: &AppState,
        _con: &AppContext,
    ) -> Result<CmdResult, ProgramError> {
        self.filtered_favorites
            .set_pattern(&app_state.favorites, pat);
        self.fix_scroll();
        self.update_selected_path(&app_state.favorites);
        Ok(CmdResult::Keep)
    }

    fn display(
        &mut self,
        w: &mut W,
        disc: &DisplayContext,
    ) -> Result<(), ProgramError> {
        let favorites = &disc.app_state.favorites;
        if self.filtered_favorites.update(favorites) {
            self.fix_scroll();
        }
        self.selected_path = self
            .filtered_favorites
            .selected_path(favorites)
            .map(|p| p.to_path_buf());
        let area = &disc.state_area;
        let styles = &disc.panel_skin.styles;
        let width = area.width as usize;
        w.queue(cursor::MoveTo(area.left, 0))?;
        let mut cw = CropWriter::new(w, width);
        self.write_title_line(favorites, &mut cw, styles)?;
        let list_area = Area::new(area.left, area.top + 1, area.width, area.height - 1);
        self.page_height = list_area.height as usize;
        let pattern = &self.filtered_favorites.pattern().pattern;
        let pattern_object = pattern.object();
        let scrollbar = list_area.scrollbar(self.scroll, self.filtered_favorites.len());
        for idx in 0..self.page_height {
            let y = list_area.top + idx as u16;
            let fav_idx = idx + self.scroll;
            w.queue(cursor::MoveTo(area.left, y))?;
            let mut cw = CropWriter::new(w, width - 1);
            let cw = &mut cw;
            if let Some((path, selected)) =
                self.filtered_favorites.path_sel(favorites, fav_idx)
            {
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
                if disc.con.show_selection_mark && self.filtered_favorites.has_selection() {
                    cw.queue_char(style, if selected { '▶' } else { ' ' })?;
                }
                if pattern_object.subpath {
                    let label = path.to_string_lossy();
                    let name_match = pattern.search_string(&label);
                    let matched_string =
                        MatchedString::new(name_match, &label, style, style_match);
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
                                cw.queue_str(parent_style, &parent_path)?;
                            } else {
                                let mut bytes_count = 0;
                                let mut cols_count = 0;
                                for c in parent_path.chars().rev() {
                                    let char_width =
                                        UnicodeWidthChar::width(c).unwrap_or(0);
                                    let next_str_width = cols_count + char_width;
                                    if next_str_width > cols_max {
                                        break;
                                    }
                                    cols_count = next_str_width;
                                    bytes_count += c.len_utf8();
                                }
                                cw.queue_char(parent_style, ELLIPSIS)?;
                                cw.queue_str(
                                    parent_style,
                                    &parent_path[parent_path.len() - bytes_count..],
                                )?;
                            }
                            cw.queue_char(parent_style, '/')?;
                        }
                    }
                    let name_match = pattern.search_string(&label);
                    let matched_string =
                        MatchedString::new(name_match, &label, style, style_match);
                    matched_string.queue_on(cw)?;
                } else {
                    warn!("path without filename in favorites");
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
        invocation_parser: Option<&InvocationParser>,
        internal_exec: &InternalExecution,
        input_invocation: Option<&VerbInvocation>,
        trigger_type: TriggerType,
        app_state: &mut AppState,
        cc: &CmdContext,
    ) -> Result<CmdResult, ProgramError> {
        let con = &cc.app.con;
        let screen = cc.app.screen;
        let result = match internal_exec.internal {
            Internal::back if self.filtered_favorites.pattern().is_some() => {
                self.filtered_favorites =
                    FilteredFavorites::unfiltered(&app_state.favorites);
                CmdResult::Keep
            }
            Internal::back if self.filtered_favorites.has_selection() => {
                self.filtered_favorites.unselect();
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
            Internal::panel_right => {
                let areas = &cc.panel.areas;
                if areas.is_last() && areas.nb_pos < 3 {
                    if let Some(path) = &self.selected_path {
                        let purpose = if path.is_file() && cc.app.preview_panel.is_none() {
                            PanelPurpose::Preview
                        } else {
                            PanelPurpose::None
                        };
                        internal_focus::new_panel_on_path(
                            path.clone(),
                            screen,
                            self.tree_options.clone(),
                            purpose,
                            con,
                            HDir::Right,
                        )
                    } else {
                        CmdResult::HandleInApp(Internal::panel_right_no_open)
                    }
                } else {
                    CmdResult::HandleInApp(Internal::panel_right_no_open)
                }
            }
            Internal::toggle_favorite | Internal::unfavorite => {
                if self.filtered_favorites.unfavorite_selection(&mut app_state.favorites) {
                    CmdResult::Keep
                } else {
                    CmdResult::error("you must select a path to unfavorite")
                }
            }
            Internal::favorite => {
                CmdResult::error("nothing to favorite here")
            }
            _ => self.on_internal_generic(
                w,
                invocation_parser,
                internal_exec,
                input_invocation,
                trigger_type,
                app_state,
                cc,
            )?,
        };
        self.update_selected_path(&app_state.favorites);
        Ok(result)
    }
}
