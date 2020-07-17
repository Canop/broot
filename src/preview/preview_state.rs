use {
    super::*,
    crate::{
        app::*,
        command::{Command, ScrollCommand, TriggerType},
        display::{CropWriter, LONG_SPACE, Screen, W},
        errors::ProgramError,
        flag::Flag,
        pattern::{InputPattern, Pattern},
        selection_type::SelectionType,
        skin::PanelSkin,
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
    pattern: InputPattern, // kept but not applied when the preview isn't filterable
    filtered_preview: Option<Preview>,
}

impl PreviewState {
    pub fn new(path: PathBuf, _con: &AppContext) -> PreviewState {
        let preview_area = Area::uninitialized(); // will be fixed at drawing time
        let preview = Preview::new(&path, Pattern::None);
        let file_name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "???".to_string());
        PreviewState {
            preview_area,
            dirty: true,
            file_name,
            path,
            preview,
            pattern: InputPattern::none(),
            filtered_preview: None,
        }
    }
    fn mut_preview(&mut self) -> &mut Preview {
        self.filtered_preview.as_mut().unwrap_or(&mut self.preview)
    }
}

impl AppState for PreviewState {

    fn selected_path(&self) -> &Path {
        &self.path
    }

    fn set_selected_path(&mut self, path: PathBuf) {
        self.preview = Preview::new(&path, Pattern::None);
        self.file_name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "???".to_string());
        if self.pattern.is_some() && self.preview.is_filterable() {
            self.filtered_preview = Some(Preview::new(&path, self.pattern.pattern.clone()));
        } else {
            self.filtered_preview = None;
        }
        self.path = path;
    }

    fn selection_type(&self) -> SelectionType {
        SelectionType::File
    }

    fn refresh(&mut self, _screen: &Screen, _con: &AppContext) -> Command {
        self.dirty = true;
        self.set_selected_path(self.path.clone());
        Command::empty()
    }

    fn on_pattern(
        &mut self,
        pat: InputPattern,
        _con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError> {
        debug!("preview pattern: {:?}", &pat);
        self.pattern = pat.clone();
        if pat.is_some() {
            if !self.preview.is_filterable() {
                return Ok(AppStateCmdResult::DisplayError(
                    "this preview can't be searched".to_string()
                ));
            }
            self.filtered_preview = Some(Preview::new(&self.path, pat.pattern));
        } else {
            self.filtered_preview = None;
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
        cw.fill(&styles.default, LONG_SPACE)?;
        self.filtered_preview.as_mut().unwrap_or(&mut self.preview)
            .display(w, screen, panel_skin, &self.preview_area)
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
                } else {
                    Ok(AppStateCmdResult::PopState)
                }
            }
            Internal::line_down => {
                self.mut_preview().try_scroll(ScrollCommand::Lines(1));
                Ok(AppStateCmdResult::Keep)
            }
            Internal::line_up => {
                self.mut_preview().try_scroll(ScrollCommand::Lines(-1));
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

}
