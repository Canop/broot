use {
    super::*,
    crate::{
        app::*,
        browser::BrowserState,
        command::{Command, ScrollCommand, TriggerType},
        display::{CropWriter, BRANCH_FILLING, SPACE_FILLING, Screen, W},
        errors::ProgramError,
        pattern::*,
        skin::PanelSkin,
        task_sync::Dam,
        tree::TreeOptions,
        verb::*,
    },
    crossterm::{
        cursor,
        style::{Color, Print, SetBackgroundColor, SetForegroundColor},
        QueueableCommand,
    },
    lfs_core::Mount,
    std::{
        convert::TryInto,
        fs,
        os::unix::fs::MetadataExt,
        path::Path,
    },
    strict::NonEmptyVec,
    termimad::{Area, ProgressBar},
};

/// an application state showing the currently mounted filesystems
pub struct FilesystemState {
    mounts: NonEmptyVec<Mount>,
    selection_idx: usize,
    scroll: usize,
    page_height: usize,
    tree_options: TreeOptions,
}

impl FilesystemState {
    /// create a state listing the filesystem, trying to select
    /// the one containing the path given in argument.
    /// Not finding any filesystem is considered an error and prevents
    /// the opening of this state.
    pub fn new(
        path: &Path,
        tree_options: TreeOptions,
        _con: &AppContext,
    ) -> Result<FilesystemState, ProgramError> {
        let mut mount_list = MOUNTS.lock().unwrap();
        let show_only_disks = false;
        let mounts = mount_list
            .load()?
            .iter()
            .filter(|mount|
                if show_only_disks {
                    mount.disk.is_some()
                } else {
                    mount.stats.is_some()
                }
            )
            .cloned()
            .collect::<Vec<Mount>>();
        let mounts: NonEmptyVec<Mount> = match mounts.try_into() {
            Ok(nev) => nev,
            _ => {
                return Err(ProgramError::Lfs{details: "no disk in lfs-core list".to_string()});
            }
        };
        let device_id = fs::metadata(path)?.dev().into();
        let selection_idx = mounts.iter()
            .position(|m| m.info.dev == device_id)
            .unwrap_or(0);
        Ok(FilesystemState {
            mounts,
            selection_idx,
            scroll: 0,
            page_height: 0,
            tree_options,
        })
    }
    pub fn count(&self) -> usize {
        self.mounts.len().into()
    }
    pub fn try_scroll(
        &mut self,
        cmd: ScrollCommand,
    ) -> bool {
        let old_scroll = self.scroll;
        self.scroll = cmd.apply(self.scroll, self.count(), self.page_height);
        self.scroll != old_scroll
    }
}

impl AppState for FilesystemState {

    fn selected_path(&self) -> &Path {
        &self.mounts[self.selection_idx].info.mount_point
    }

    fn tree_options(&self) -> TreeOptions {
        self.tree_options.clone()
    }

    fn with_new_options(
        &mut self,
        _screen: Screen,
        change_options: &dyn Fn(&mut TreeOptions),
        _in_new_panel: bool, // TODO open tree if true
        _con: &AppContext,
    ) -> AppStateCmdResult {
        change_options(&mut self.tree_options);
        AppStateCmdResult::Keep
    }

    fn selection(&self) -> Selection<'_> {
        Selection {
            path: self.selected_path(),
            stype: SelectionType::Directory,
            is_exe: false,
            line: 0,
        }
    }

    fn refresh(&mut self, _screen: Screen, _con: &AppContext) -> Command {
        Command::empty()
    }

    fn on_pattern(
        &mut self,
        _pat: InputPattern,
        _con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError> {
        //self.pattern = pat.pattern;
        Ok(AppStateCmdResult::Keep)
    }

    fn display(
        &mut self,
        w: &mut W,
        _screen: Screen,
        area: Area,
        panel_skin: &PanelSkin,
        con: &AppContext,
    ) -> Result<(), ProgramError> {
        self.page_height = area.height as usize;
        let scrollbar = area.scrollbar(self.scroll as i32, self.count() as i32);
        //- style preparation
        let styles = &panel_skin.styles;
        let normal_bg = styles.default.get_bg()
            .or_else(|| styles.preview.get_bg())
            .unwrap_or(Color::AnsiValue(238));
        let selection_bg = styles.selected_line.get_bg()
            .unwrap_or(Color::AnsiValue(240));
        let text = |cw: &mut CropWriter<W>, s: String| {
            cw.queue_fg(&styles.default)?;
            cw.queue_unstyled_g_string(s)
        };
        let border = |cw: &mut CropWriter<W>| {
            cw.queue_fg(&styles.help_table_border)?;
            cw.queue_unstyled_char('│')
        };
        let scrollbar_fg = styles.scrollbar_thumb.get_fg()
            .or_else(|| styles.preview.get_fg())
            .unwrap_or_else(|| Color::White);
        //- width computations and selection of columns to display
        let width = area.width as usize;
        let w_fs = self.mounts.iter()
            .map(|m| m.info.fs.chars().count())
            .max().unwrap() // unwrap is safe because mounts is a nonEmptyVec
            .max("filesystem".len());
        let mut wc_fs = w_fs; // width of the column (may include selection mark)
        if con.show_selection_mark {
            wc_fs += 1;
        }
        let w_dsk = 3;
        let w_type = self.mounts.iter()
            .map(|m| m.info.fs_type.chars().count())
            .max().unwrap()
            .max("type".len());
        let w_size = 4;
        let w_use = 4;
        let mut w_use_bar = 1; // min size, may grow if space available
        let w_use_share = 4;
        let mut wc_use = w_use; // sum of all the parts of the usage column
        let w_free = 4;
        let w_mount_point = self.mounts.iter()
            .map(|m| m.info.mount_point.to_string_lossy().chars().count())
            .max().unwrap()
            .max("mount point".len());
        let w_mandatory = wc_fs + 1 + w_size + 1 + w_free + 1 + w_mount_point;
        let mut e_dsk = false;
        let mut e_type = false;
        let mut e_use_bar = false;
        let mut e_use_share = false;
        let mut e_use = false;
        if w_mandatory + 1 < width {
            let mut rem = width - w_mandatory - 1;
            if rem > w_use {
                rem -= w_use + 1;
                e_use = true;
            }
            if e_use && rem > w_use_share {
                rem -= w_use_share; // no separation with use
                e_use_share = true;
                wc_use += w_use_share;
            }
            if rem > w_dsk {
                rem -= w_dsk + 1;
                e_dsk = true;
            }
            if e_use && rem > w_use_bar {
                rem -= w_use_bar + 1;
                e_use_bar = true;
                wc_use += w_use_bar + 1;
            }
            if rem > w_type {
                rem -= w_type + 1;
                e_type = true;
            }
            if e_use_bar && rem > 0 {
                let incr = rem.min(9);
                w_use_bar += incr;
                wc_use += incr;
            }
        }
        //- titles
        w.queue(cursor::MoveTo(area.left, area.top))?;
        w.queue(SetBackgroundColor(normal_bg))?;
        let mut cw = CropWriter::new(w, width);
        let cw = &mut cw;
        text(cw, format!("{:width$}", "filesystem", width = wc_fs))?;
        border(cw)?;
        if e_dsk {
            text(cw, "dsk".to_string())?;
            border(cw)?;
        }
        if e_type {
            text(cw, format!("{:^width$}", "type", width = w_type))?;
            border(cw)?;
        }
        text(cw, "size".to_string())?;
        border(cw)?;
        if e_use {
            text(cw, format!("{:^width$}", if wc_use > 4 { "usage" } else { "use" }, width = wc_use))?;
            border(cw)?;
        }
        text(cw, "free".to_string())?;
        border(cw)?;
        text(cw, "mount point".to_string())?;
        cw.fill(&styles.help_table_border, &SPACE_FILLING)?;
        //- horizontal line
        w.queue(cursor::MoveTo(area.left, 1 + area.top))?;
        w.queue(SetBackgroundColor(normal_bg))?;
        let mut cw = CropWriter::new(w, width);
        let cw = &mut cw;
        cw.queue_fg(&styles.help_table_border)?;
        cw.queue_unstyled_g_string(format!("{:─>width$}", '┼', width = wc_fs+1))?;
        if e_dsk {
            cw.queue_unstyled_g_string(format!("{:─>width$}", '┼', width = w_dsk+1))?;
        }
        if e_type {
            cw.queue_unstyled_g_string(format!("{:─>width$}", '┼', width = w_type+1))?;
        }
        cw.queue_unstyled_g_string(format!("{:─>width$}", '┼', width = w_size+1))?;
        if e_use {
            cw.queue_unstyled_g_string(format!("{:─>width$}", '┼', width = wc_use+1))?;
        }
        cw.queue_unstyled_g_string(format!("{:─>width$}", '┼', width = w_free+1))?;
        cw.fill(&styles.help_table_border, &BRANCH_FILLING)?;
        //- content
        let mut idx = self.scroll as usize;
        for y in 2..area.height {
            w.queue(cursor::MoveTo(area.left, y + area.top))?;
            let selected = self.selection_idx == idx;
            let bg = if selected {
                selection_bg
            } else {
                normal_bg
            };
            let mut cw = CropWriter::new(w, width - 1);
            let cw = &mut cw;
            cw.w.queue(SetBackgroundColor(bg))?;
            if let Some(mount) = self.mounts.get(idx) {
                if con.show_selection_mark {
                    cw.queue_unstyled_char(if selected { '▶' } else { ' ' })?;
                }
                // fs
                text(cw, format!("{:width$}", &mount.info.fs, width = w_fs))?;
                border(cw)?;
                // dsk
                if e_dsk {
                    text(cw, mount.disk.as_ref().map_or_else(
                        || "   ".to_string(),
                        |d| format!("{:>3}", d.disk_type()),
                    ))?;
                    border(cw)?;
                }
                // type
                if e_type {
                    text(cw, format!("{:^width$}", &mount.info.fs_type, width = w_type))?;
                    border(cw)?;
                }
                // size, used, free
                if let Some(stats) = mount.stats.as_ref().filter(|s|s.size()>0) {
                    // size
                    text(cw, format!("{:>4}", file_size::fit_4(mount.size())))?;
                    border(cw)?;
                    // used
                    if e_use {
                        text(cw, format!("{:>4}", file_size::fit_4(stats.used())))?;
                        let share_color = super::share_color(stats.use_share());
                        if e_use_bar {
                            let pb = ProgressBar::new(stats.use_share() as f32, w_use_bar);
                            cw.queue_unstyled_char(' ')?;
                            cw.w.queue(SetBackgroundColor(share_color))?;
                            text(cw, format!("{:<width$}", pb, width=w_use_bar))?;
                            cw.w.queue(SetBackgroundColor(bg))?;
                        }
                        if e_use_share {
                            cw.w.queue(SetForegroundColor(share_color))?;
                            cw.queue_unstyled_g_string(format!("{:>3.0}%", 100.0*stats.use_share()))?;
                        }
                        border(cw)?;
                    }
                    // free
                    text(cw, format!("{:>4}", file_size::fit_4(stats.available())))?;
                    border(cw)?;
                } else {
                    // size
                    cw.repeat_unstyled(&SPACE_FILLING, w_size)?;
                    border(cw)?;
                    // used
                    if e_use {
                        cw.repeat_unstyled(&SPACE_FILLING, wc_use)?;
                        border(cw)?;
                    }
                    // free
                    cw.repeat_unstyled(&SPACE_FILLING, w_free)?;
                    border(cw)?;
                }
                // mount point
                text(cw, mount.info.mount_point.to_string_lossy().to_string())?;
                idx += 1;
            }
            cw.fill_unstyled(&SPACE_FILLING)?;
            w.queue(SetBackgroundColor(bg))?;
            if is_thumb(y, scrollbar) {
                w.queue(SetForegroundColor(scrollbar_fg))?;
                w.queue(Print('▐'))?;
            } else {
                w.queue(Print(' '))?;
            }
        }
        Ok(())
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
            Internal::line_down => {
                if self.selection_idx + 1 < self.count() {
                    self.selection_idx += 1;
                }
                AppStateCmdResult::Keep
            }
            Internal::line_up => {
                if self.selection_idx > 0 {
                    self.selection_idx -= 1;
                }
                AppStateCmdResult::Keep
            }
            Internal::open_stay => {
                let in_new_panel = input_invocation
                    .map(|inv| inv.bang)
                    .unwrap_or(internal_exec.bang);
                let dam = Dam::unlimited();
                let mut tree_options = self.tree_options();
                tree_options.show_root_fs = true;
                AppStateCmdResult::from_optional_state(
                    BrowserState::new(
                        self.selected_path().to_path_buf(),
                        tree_options,
                        screen,
                        &cc.con,
                        &dam,
                    ),
                    in_new_panel,
                )
            }
            Internal::page_down => {
                self.try_scroll(ScrollCommand::Pages(1));
                AppStateCmdResult::Keep
            }
            Internal::page_up => {
                self.try_scroll(ScrollCommand::Pages(-1));
                AppStateCmdResult::Keep
            }
            open_leave => AppStateCmdResult::PopStateAndReapply,
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

    fn on_click(
        &mut self,
        _x: u16,
        y: u16,
        _screen: Screen,
        _con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError> {
        if y >= 2 {
            let y = y as usize - 2 + self.scroll;
            if y < self.mounts.len().into() {
                self.selection_idx = y;
            }
        }
        Ok(AppStateCmdResult::Keep)
    }
}

fn is_thumb(y: u16, scrollbar: Option<(u16, u16)>) -> bool {
    if let Some((sctop, scbottom)) = scrollbar {
        if sctop <= y && y <= scbottom {
            return true;
        }
    }
    false
}
