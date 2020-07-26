use {
    super::*,
    crate::{
        app::*,
        command::{Command, ScrollCommand, TriggerType},
        display::{CropWriter, LONG_SPACE, Screen, W},
        errors::ProgramError,
        flag::Flag,
        pattern::InputPattern,
        selection_type::SelectionType,
        skin::PanelSkin,
        task_sync::Dam,
        verb::*,
    },
    crossterm::{
        cursor,
        QueueableCommand,
    },
    std::path::{Path, PathBuf},
    termimad::{Area},
};

/// an application state dedicated to previewing files
pub struct PreviewState {
    pub preview_area: Area,
    dirty: bool, // background must be cleared
    file_name: String,
    path: PathBuf, // path to the previewed file
    preview: Preview,
    //pattern: InputPattern, // kept but not applied when the preview isn't filterable
    pending_pattern: InputPattern, // a pattern (or not) which has not yet be applied
    filtered_preview: Option<Preview>,
}

impl PreviewState {
    pub fn new(
        path: PathBuf,
        pending_pattern: InputPattern,
        con: &AppContext,
    ) -> PreviewState {
        let preview_area = Area::uninitialized(); // will be fixed at drawing time
        let preview = Preview::unfiltered(&path, con);
        let file_name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "???".to_string());
        PreviewState {
            preview_area,
            dirty: true,
            file_name,
            path,
            preview,
            pending_pattern,
            filtered_preview: None,
        }
    }
    fn mut_preview(&mut self) -> &mut Preview {
        self.filtered_preview.as_mut().unwrap_or(&mut self.preview)
    }
}

impl AppState for PreviewState {

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
        _con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError> {
        if pat.is_none() {
            if let Some(filtered_preview) = self.filtered_preview.take() {
                let old_selection = filtered_preview.get_selected_line_number();
                if let Some(number) = old_selection {
                    self.preview.try_select_line_number(number);
                }
            }
        } else {
            if !self.preview.is_filterable() {
                return Ok(AppStateCmdResult::DisplayError(
                    "this preview can't be searched".to_string()
                ));
            }
        }
        self.pending_pattern = pat.clone();
        Ok(AppStateCmdResult::Keep)
    }

    fn do_pending_task(
        &mut self,
        _screen: &mut Screen,
        con: &AppContext,
        dam: &mut Dam,
    ) {
        if self.pending_pattern.is_some() {
            let old_selection = self
                .filtered_preview.as_ref().and_then(|p| p.get_selected_line_number())
                .or(self.preview.get_selected_line_number());
            let pattern = self.pending_pattern.take();
            self.filtered_preview = time!(
                Info,
                "preview filtering",
                Preview::filtered(&self.path, pattern, dam, con),
            ); // can be None if a cancellation was required
            if let Some(ref mut filtered_preview) = self.filtered_preview {
                if let Some(number) = old_selection {
                    filtered_preview.try_select_line_number(number);
                }
            }
        }
    }

    fn selected_path(&self) -> &Path {
        &self.path
    }

    fn set_selected_path(&mut self, path: PathBuf, con: &AppContext) {
        if let Some(fp) = &self.filtered_preview {
            self.pending_pattern = fp.pattern();
        };
        self.preview = Preview::unfiltered(&path, con);
        self.file_name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "???".to_string());
        self.path = path;
    }

    fn selection_type(&self) -> SelectionType {
        SelectionType::File
    }

    fn refresh(&mut self, _screen: &Screen, con: &AppContext) -> Command {
        self.dirty = true;
        self.set_selected_path(self.path.clone(), con);
        Command::empty()
    }

    fn on_click(
        &mut self,
        _x: u16,
        y: u16,
        _screen: &mut Screen,
        _con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError> {
        if y >= self.preview_area.top  && y < self.preview_area.top + self.preview_area.height {
            let y = y - self.preview_area.top;
            self.mut_preview().try_select_y(y);
        }
        Ok(AppStateCmdResult::Keep)
    }

    fn display(
        &mut self,
        w: &mut W,
        screen: &Screen,
        state_area: Area,
        panel_skin: &PanelSkin,
        _con: &AppContext,
    ) -> Result<(), ProgramError> {
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
            panel_skin.styles.default.queue_bg(w)?;
            screen.clear_area_to_right(w, &state_area)?;
            self.dirty = false;
        }
        let styles = &panel_skin.styles;
        w.queue(cursor::MoveTo(state_area.left, 0))?;
        let mut cw = CropWriter::new(w, state_area.width as usize);
        cw.queue_str(&styles.default, &self.file_name)?;
        let info_area = Area::new(
            state_area.left + state_area.width - cw.allowed as u16,
            state_area.top,
            cw.allowed as u16,
            1,
        );
        cw.fill(&styles.default, LONG_SPACE)?;
        let preview = self.filtered_preview.as_mut().unwrap_or(&mut self.preview);
        preview.display_info(w, screen, panel_skin, &info_area)?;
        preview.display(w, screen, panel_skin, &self.preview_area)
    }

    fn get_status(
        &self,
        cmd: &Command,
        other_path: &Option<PathBuf>,
        con: &AppContext,
    ) -> Status {
        match cmd {
            Command::VerbEdit(invocation) => {
                if invocation.name.is_empty() {
                    Status::from_message(
                        "Type a verb then *enter* to execute it (*?* for the list of verbs)",
                    )
                } else {
                    match con.verb_store.search(&invocation.name) {
                        PrefixSearchResult::NoMatch => Status::from_error("No matching verb"),
                        PrefixSearchResult::Match(_, verb) => {
                            verb.get_status(&self.path, other_path, invocation)
                        }
                        PrefixSearchResult::Matches(completions) => {
                            Status::from_message(format!(
                                "Possible completions: {}",
                                completions
                                    .iter()
                                    .map(|c| format!("*{}*", c))
                                    .collect::<Vec<String>>()
                                    .join(", "),
                            ))
                        }
                    }
                }
            }
            _ => Status::from_message(
                "Hit *esc* to get back to the tree, or a space to start a verb",
            ),
        }
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
        match internal_exec.internal {
            Internal::back => {
                if self.filtered_preview.is_some() {
                    self.on_pattern(InputPattern::none(), &cc.con)
                } else if self.mut_preview().get_selected_line_number().is_some() {
                    self.mut_preview().unselect();
                    Ok(AppStateCmdResult::Keep)
                } else {
                    Ok(AppStateCmdResult::PopState)
                }
            }
            Internal::line_down => {
                self.mut_preview().select_next_line();
                Ok(AppStateCmdResult::Keep)
            }
            Internal::line_up => {
                self.mut_preview().select_previous_line();
                Ok(AppStateCmdResult::Keep)
            }
            Internal::page_down => {
                self.mut_preview().try_scroll(ScrollCommand::Pages(1));
                Ok(AppStateCmdResult::Keep)
            }
            Internal::page_up => {
                self.mut_preview().try_scroll(ScrollCommand::Pages(-1));
                Ok(AppStateCmdResult::Keep)
            }
            Internal::select_first => {
                self.mut_preview().select_first();
                Ok(AppStateCmdResult::Keep)
            }
            Internal::select_last => {
                self.mut_preview().select_last();
                Ok(AppStateCmdResult::Keep)
            }
            _ => self.on_internal_generic(
                w,
                internal_exec,
                input_invocation,
                trigger_type,
                cc,
                screen,
            ),
        }
    }

    // TODO put the hex/txt view here
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
