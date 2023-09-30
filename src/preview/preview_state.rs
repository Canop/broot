use {
    super::*,
    crate::{
        app::*,
        command::{Command, ScrollCommand, TriggerType},
        display::{Screen, W},
        errors::ProgramError,
        flag::Flag,
        pattern::InputPattern,
        task_sync::Dam,
        tree::TreeOptions,
        verb::*,
    },
    crokey::crossterm::{
        cursor,
        QueueableCommand,
    },
    std::path::{Path, PathBuf},
    termimad::{Area, CropWriter, SPACE_FILLING},
};

/// an application state dedicated to previewing files.
/// It's usually the only state in its panel and is kept when
/// the selection changes (other panels indirectly call
/// set_selected_path).
pub struct PreviewState {
    pub preview_area: Area,
    dirty: bool,   // true when background must be cleared
    path: PathBuf, // path to the previewed file
    preview: Preview,
    pending_pattern: InputPattern, // a pattern (or not) which has not yet be applied
    filtered_preview: Option<Preview>,
    removed_pattern: InputPattern,
    preferred_mode: Option<PreviewMode>,
    tree_options: TreeOptions,
    mode: Mode,
}

impl PreviewState {
    pub fn new(
        path: PathBuf,
        pending_pattern: InputPattern,
        preferred_mode: Option<PreviewMode>,
        tree_options: TreeOptions,
        con: &AppContext,
    ) -> PreviewState {
        let preview_area = Area::uninitialized(); // will be fixed at drawing time
        let preview = Preview::new(&path, preferred_mode, con);
        PreviewState {
            preview_area,
            dirty: true,
            path,
            preview,
            pending_pattern,
            filtered_preview: None,
            removed_pattern: InputPattern::none(),
            preferred_mode,
            tree_options,
            mode: initial_mode(con),
        }
    }
    fn vis_preview(&self) -> &Preview {
        self.filtered_preview.as_ref().unwrap_or(&self.preview)
    }
    fn mut_preview(&mut self) -> &mut Preview {
        self.filtered_preview.as_mut().unwrap_or(&mut self.preview)
    }
    fn set_mode(
        &mut self,
        mode: PreviewMode,
        con: &AppContext,
    ) -> Result<CmdResult, ProgramError> {
        if self.preview.get_mode() == Some(mode) {
            return Ok(CmdResult::Keep);
        }
        Ok(match Preview::with_mode(&self.path, mode, con) {
            Ok(preview) => {
                self.preview = preview;
                self.preferred_mode = Some(mode);
                CmdResult::Keep
            }
            Err(e) => {
                CmdResult::DisplayError(
                    format!("Can't display as {mode:?} : {e:?}")
                )
            }
        })
    }

    fn no_opt_selection(&self) -> Selection<'_> {
        Selection {
            path: &self.path,
            stype: SelectionType::File,
            is_exe: false, // not always true. It means :open_leave won't execute it
            line: self.vis_preview().get_selected_line_number().unwrap_or(0),
        }
    }

}

impl PanelState for PreviewState {

    fn get_type(&self) -> PanelStateType {
        PanelStateType::Preview
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    fn get_mode(&self) -> Mode {
        self.mode
    }

    fn get_pending_task(&self) -> Option<&'static str> {
        if self.pending_pattern.is_some() {
            Some("searching")
        } else {
            None
        }
    }

    fn on_pattern(
        &mut self,
        pat: InputPattern,
        _app_state: &AppState,
        _con: &AppContext,
    ) -> Result<CmdResult, ProgramError> {
        if pat.is_none() {
            if let Some(filtered_preview) = self.filtered_preview.take() {
                let old_selection = filtered_preview.get_selected_line_number();
                if let Some(number) = old_selection {
                    self.preview.try_select_line_number(number);
                }
                self.removed_pattern = filtered_preview.pattern();
            }
        } else if !self.preview.is_filterable() {
            return Ok(CmdResult::error("this preview can't be searched"));
        }
        self.pending_pattern = pat;
        Ok(CmdResult::Keep)
    }

    /// do the preview filtering if required and not yet done
    fn do_pending_task(
        &mut self,
        _app_state: &mut AppState,
        _screen: Screen,
        con: &AppContext,
        dam: &mut Dam,
    ) -> Result<(), ProgramError> {
        if self.pending_pattern.is_some() {
            let old_selection = self
                .filtered_preview
                .as_ref()
                .and_then(|p| p.get_selected_line_number())
                .or_else(|| self.preview.get_selected_line_number());
            let pattern = self.pending_pattern.take();
            self.filtered_preview = time!(
                Info,
                "preview filtering",
                self.preview.filtered(&self.path, pattern, dam, con),
            ); // can be None if a cancellation was required
            if let Some(ref mut filtered_preview) = self.filtered_preview {
                if let Some(number) = old_selection {
                    filtered_preview.try_select_line_number(number);
                }
            }
        }
        Ok(())
    }

    fn selected_path(&self) -> Option<&Path> {
        Some(&self.path)
    }

    fn set_selected_path(&mut self, path: PathBuf, con: &AppContext) {
        let selected_line_number = if self.path == path {
            self.preview.get_selected_line_number()
        } else {
            None
        };
        if let Some(fp) = &self.filtered_preview {
            self.pending_pattern = fp.pattern();
        };
        self.preview = Preview::new(&path, self.preferred_mode, con);
        if let Some(number) = selected_line_number {
            self.preview.try_select_line_number(number);
        }
        self.path = path;
    }

    fn selection(&self) -> Option<Selection<'_>> {
        Some(self.no_opt_selection())
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

    fn refresh(&mut self, _screen: Screen, con: &AppContext) -> Command {
        self.dirty = true;
        self.set_selected_path(self.path.clone(), con);
        Command::empty()
    }

    fn on_click(
        &mut self,
        _x: u16,
        y: u16,
        _screen: Screen,
        _con: &AppContext,
    ) -> Result<CmdResult, ProgramError> {
        if y >= self.preview_area.top && y < self.preview_area.top + self.preview_area.height {
            let y = y - self.preview_area.top;
            self.mut_preview().try_select_y(y);
        }
        Ok(CmdResult::Keep)
    }

    fn display(
        &mut self,
        w: &mut W,
        disc: &DisplayContext,
    ) -> Result<(), ProgramError> {
        let state_area = &disc.state_area;
        if state_area.height < 3 {
            warn!("area too small for preview");
            return Ok(());
        }
        let mut preview_area = state_area.clone();
        preview_area.height -= 1;
        preview_area.top += 1;
        if preview_area != self.preview_area {
            self.dirty = true;
            self.preview_area = preview_area;
        }
        if self.dirty {
            disc.panel_skin.styles.default.queue_bg(w)?;
            disc.screen.clear_area_to_right(w, state_area)?;
            self.dirty = false;
        }
        let styles = &disc.panel_skin.styles;
        w.queue(cursor::MoveTo(state_area.left, 0))?;
        let mut cw = CropWriter::new(w, state_area.width as usize);
        let file_name = self
            .path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "???".to_string());
        cw.queue_str(&styles.preview_title, &file_name)?;
        let info_area = Area::new(
            state_area.left + state_area.width - cw.allowed as u16,
            state_area.top,
            cw.allowed as u16,
            1,
        );
        cw.fill(&styles.preview_title, &SPACE_FILLING)?;
        let preview = self.filtered_preview.as_mut().unwrap_or(&mut self.preview);
        preview.display_info(w, disc.screen, disc.panel_skin, &info_area)?;
        if let Err(err) = preview.display(w, disc, &self.preview_area) {
            warn!("error while displaying file: {:?}", &err);
            if preview.get_mode().is_some() {
                // means it's not an error already
                if let ProgramError::Io { source } = err {
                    // we mutate the preview to Preview::IOError
                    self.preview = Preview::IoError(source);
                    return self.display(w, disc);
                }
            }
            return Err(err);
        }
        Ok(())
    }

    fn no_verb_status(
        &self,
        has_previous_state: bool,
        con: &AppContext,
        width: usize, // available width
    ) -> Status {
        let mut ssb = con.standard_status.builder(
            PanelStateType::Preview,
            self.no_opt_selection(),
            width,
        );
        ssb.has_previous_state = has_previous_state;
        ssb.is_filtered = self.filtered_preview.is_some();
        ssb.has_removed_pattern = self.removed_pattern.is_some();
        ssb.status()
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
        let con = &cc.app.con;
        match internal_exec.internal {
            Internal::back => {
                if self.filtered_preview.is_some() {
                    self.on_pattern(InputPattern::none(), app_state, con)
                } else {
                    Ok(CmdResult::PopState)
                }
            }
            Internal::copy_line => {
                #[cfg(not(feature = "clipboard"))]
                {
                    Ok(CmdResult::error("Clipboard feature not enabled at compilation"))
                }
                #[cfg(feature = "clipboard")]
                {
                    Ok(match self.mut_preview().get_selected_line() {
                        Some(line) => {
                            match terminal_clipboard::set_string(line) {
                                Ok(()) => CmdResult::Keep,
                                Err(_) => CmdResult::error("Clipboard error while copying path"),
                            }
                        }
                        None => CmdResult::error("No selected line in preview"),
                    })
                }
            }
            Internal::line_down => {
                let count = get_arg(input_invocation, internal_exec, 1);
                self.mut_preview().move_selection(count, true);
                Ok(CmdResult::Keep)
            }
            Internal::line_up => {
                let count = get_arg(input_invocation, internal_exec, 1);
                self.mut_preview().move_selection(-count, true);
                Ok(CmdResult::Keep)
            }
            Internal::line_down_no_cycle => {
                let count = get_arg(input_invocation, internal_exec, 1);
                self.mut_preview().move_selection(count, false);
                Ok(CmdResult::Keep)
            }
            Internal::line_up_no_cycle => {
                let count = get_arg(input_invocation, internal_exec, 1);
                self.mut_preview().move_selection(-count, false);
                Ok(CmdResult::Keep)
            }
            Internal::page_down => {
                self.mut_preview().try_scroll(ScrollCommand::Pages(1));
                Ok(CmdResult::Keep)
            }
            Internal::page_up => {
                self.mut_preview().try_scroll(ScrollCommand::Pages(-1));
                Ok(CmdResult::Keep)
            }
            //Internal::restore_pattern => {
            //    debug!("restore_pattern");
            //    self.pending_pattern = self.removed_pattern.take();
            //    Ok(CmdResult::Keep)
            //}
            Internal::panel_left if self.removed_pattern.is_some() => {
                self.pending_pattern = self.removed_pattern.take();
                Ok(CmdResult::Keep)
            }
            Internal::panel_left_no_open if self.removed_pattern.is_some() => {
                self.pending_pattern = self.removed_pattern.take();
                Ok(CmdResult::Keep)
            }
            Internal::panel_right if self.filtered_preview.is_some() => {
                self.on_pattern(InputPattern::none(), app_state, con)
            }
            Internal::panel_right_no_open if self.filtered_preview.is_some() => {
                self.on_pattern(InputPattern::none(), app_state, con)
            }
            Internal::select_first => {
                self.mut_preview().select_first();
                Ok(CmdResult::Keep)
            }
            Internal::select_last => {
                self.mut_preview().select_last();
                Ok(CmdResult::Keep)
            }
            Internal::preview_image => self.set_mode(PreviewMode::Image, con),
            Internal::preview_text => self.set_mode(PreviewMode::Text, con),
            Internal::preview_binary => self.set_mode(PreviewMode::Hex, con),
            _ => self.on_internal_generic(
                w,
                internal_exec,
                input_invocation,
                trigger_type,
                app_state,
                cc,
            ),
        }
    }

    fn get_flags(&self) -> Vec<Flag> {
        vec![]
    }

    fn get_starting_input(&self) -> String {
        if let Some(preview) = &self.filtered_preview {
            preview.pattern().raw
        } else {
            self.pending_pattern.raw.clone()
        }
    }

}
