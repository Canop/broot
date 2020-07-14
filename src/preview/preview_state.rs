use {
    super::*,
    crate::{
        app::*,
        browser::BrowserState,
        command::{Command, ScrollCommand, TriggerType},
        conf::{self, Conf},
        display::{CropWriter, LONG_SPACE, Screen, W},
        errors::ProgramError,
        flag::Flag,
        launchable::Launchable,
        print,
        selection_type::SelectionType,
        skin::PanelSkin,
        task_sync::Dam,
        tree::TreeOptions,
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
}

impl PreviewState {
    pub fn new(path: PathBuf, _con: &AppContext) -> PreviewState {
        let preview_area = Area::uninitialized(); // will be fixed at drawing time
        let preview = Preview::from_path(&path);
        let file_name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "???".to_string());
        PreviewState {
            preview_area,
            dirty: true,
            file_name,
            path,
            preview,
        }
    }
}

impl AppState for PreviewState {

    fn selected_path(&self) -> &Path {
        &self.path
    }

    fn set_selected_path(&mut self, path: PathBuf) {
        // this is only called when the path really changed
        self.preview = Preview::from_path(&path);
        self.file_name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "???".to_string());
        self.path = path;
    }

    fn selection_type(&self) -> SelectionType {
        SelectionType::File
    }

    fn refresh(&mut self, _screen: &Screen, _con: &AppContext) -> Command {
        self.dirty = true;
        Command::empty()
    }

    fn display(
        &mut self,
        w: &mut W,
        screen: &Screen,
        state_area: Area,
        panel_skin: &PanelSkin,
        _con: &AppContext,
    ) -> Result<(), ProgramError> {
        if state_area.height < 8 {
            warn!("area too small for preview");
            return Ok(());
        }
        if self.dirty {
            panel_skin.styles.default.queue_bg(w)?;
            screen.clear_area_to_right(w, &state_area)?;
            self.preview_area = state_area.clone();
            self.preview_area.height -= 2;
            self.preview_area.top += 2;
            self.dirty = false;
        }
        let styles = &panel_skin.styles;
        w.queue(cursor::MoveTo(state_area.left, 0))?;
        let mut cw = CropWriter::new(w, state_area.width as usize);
        cw.queue_str(&styles.file, &self.file_name)?;
        cw.fill(&styles.file, LONG_SPACE)?;

        debug!("display preview on {:?}", &self.path);
        self.preview.display(w, screen, panel_skin, &self.preview_area)
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
        _w: &mut W,
        internal_exec: &InternalExecution,
        input_invocation: Option<&VerbInvocation>,
        _trigger_type: TriggerType,
        cc: &CmdContext,
        screen: &mut Screen,
    ) -> Result<AppStateCmdResult, ProgramError> {
        use Internal::*;
        let bang = input_invocation
            .map(|inv| inv.bang)
            .unwrap_or(internal_exec.bang);
        Ok(match internal_exec.internal {
            back => AppStateCmdResult::PopState,
            focus | parent => AppStateCmdResult::from_optional_state(
                BrowserState::new(
                    conf::dir().to_path_buf(),
                    TreeOptions::default(),
                    screen,
                    &cc.con,
                    &Dam::unlimited(),
                ),
                bang,
            ),
            Internal::close_panel_ok => AppStateCmdResult::ClosePanel {
                validate_purpose: true,
                id: None,
            },
            Internal::close_panel_cancel => AppStateCmdResult::ClosePanel {
                validate_purpose: false,
                id: None,
            },
            help => AppStateCmdResult::Keep,
            line_down => {
                self.preview.try_scroll(ScrollCommand::Lines(1));
                AppStateCmdResult::Keep
            }
            line_up => {
                self.preview.try_scroll(ScrollCommand::Lines(-1));
                AppStateCmdResult::Keep
            }
            open_stay => match open::that(&Conf::default_location()) {
                Ok(exit_status) => {
                    info!("open returned with exit_status {:?}", exit_status);
                    AppStateCmdResult::Keep
                }
                Err(e) => AppStateCmdResult::DisplayError(format!("{:?}", e)),
            },
            open_leave => {
                AppStateCmdResult::from(Launchable::opener(self.path.clone()))
            }
            page_down => {
                self.preview.try_scroll(ScrollCommand::Pages(1));
                AppStateCmdResult::Keep
            }
            page_up => {
                self.preview.try_scroll(ScrollCommand::Pages(-1));
                AppStateCmdResult::Keep
            }
            Internal::panel_left => {
                if cc.areas.is_first() {
                    AppStateCmdResult::Keep
                } else {
                    // we ask the app to focus the panel to the left
                    AppStateCmdResult::Propagate(Internal::panel_left)
                }
            }
            Internal::panel_right => {
                if cc.areas.is_last() {
                    AppStateCmdResult::Keep
                } else {
                    // we ask the app to focus the panel to the left
                    AppStateCmdResult::Propagate(Internal::panel_right)
                }
            }
            print_path => print::print_path(&Conf::default_location(), &cc.con)?,
            print_relative_path => print::print_relative_path(&Conf::default_location(), &cc.con)?,
            quit => AppStateCmdResult::Quit,
            _ => AppStateCmdResult::Keep,
        })
    }

    // TODO put the hex/txt view here
    fn get_flags(&self) -> Vec<Flag> {
        vec![]
    }

}
