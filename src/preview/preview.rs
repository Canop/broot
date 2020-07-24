
use {
    crate::{
        command::{ScrollCommand},
        display::{Screen, W},
        errors::ProgramError,
        hex::HexView,
        pattern::Pattern,
        skin::PanelSkin,
        syntactic::SyntacticView,
    },
    std::{
        io,
        path::{Path},
    },
    termimad::{Area},
};

pub enum Preview {
    Syntactic(SyntacticView),
    Hex(HexView),
    IOError,
}

impl Preview {
    pub fn new(
        path: &Path,
        pattern: Pattern,
        desired_selection: Option<usize>,
    ) -> Self {
        if let Ok(view) = SyntacticView::new(path, pattern, desired_selection) {
            return Self::Syntactic(view);
        }
        match HexView::new(path.to_path_buf()) {
            Ok(reader) => Self::Hex(reader),
            Err(e) => {
                warn!("error while previewing {:?} : {:?}", path, e);
                Self::IOError
            }
        }
    }
    pub fn try_scroll(
        &mut self,
        cmd: ScrollCommand,
    ) -> bool {
        match self {
            Self::Syntactic(sv) => sv.try_scroll(cmd),
            Self::Hex(hv) => hv.try_scroll(cmd),
            _ => false,
        }
    }
    pub fn is_filterable(&self) -> bool {
        match self {
            Self::Syntactic(_) => true,
            _ => false,
        }
    }

    pub fn get_selected_line_number(&self) -> Option<usize> {
        match self {
            Self::Syntactic(sv) => sv.get_selected_line_number(),
            _ => None,
        }
    }
    pub fn try_select_line_number(&mut self, number: usize) -> io::Result<bool> {
        match self {
            Self::Syntactic(sv) => sv.try_select_line_number(number),
            _ => Ok(false),
        }
    }
    pub fn unselect(&mut self) {
        match self {
            Self::Syntactic(sv) => sv.unselect(),
            _ => {}
        }
    }
    pub fn try_select_y(&mut self, y: u16) -> io::Result<bool> {
        match self {
            Self::Syntactic(sv) => sv.try_select_y(y),
            _ => Ok(false),
        }
    }
    pub fn select_previous_line(&mut self) -> io::Result<()> {
        match self {
            Self::Syntactic(sv) => sv.select_previous_line(),
            Self::Hex(hv) => {
                hv.try_scroll(ScrollCommand::Lines(-1));
                Ok(())
            }
            _ => Ok(()),
        }
    }
    pub fn select_next_line(&mut self) -> io::Result<()> {
        match self {
            Self::Syntactic(sv) => sv.select_next_line(),
            Self::Hex(hv) => {
                hv.try_scroll(ScrollCommand::Lines(1));
                Ok(())
            }
            _ => Ok(()),
        }
    }
    pub fn select_first(&mut self) -> io::Result<()> {
        match self {
            Self::Syntactic(sv) => sv.select_first(),
            Self::Hex(hv) => Ok(hv.select_first()),
            _ => Ok(()),
        }
    }
    pub fn select_last(&mut self) -> io::Result<()> {
        match self {
            Self::Syntactic(sv) => sv.select_last(),
            Self::Hex(hv) => Ok(hv.select_last()),
            _ => Ok(()),
        }
    }
    pub fn display(
        &mut self,
        w: &mut W,
        screen: &Screen,
        panel_skin: &PanelSkin,
        area: &Area,
    ) -> Result<(), ProgramError> {
        match self {
            Self::Syntactic(sv) => sv.display(w, screen, panel_skin, area),
            Self::Hex(hv) => hv.display(w, screen, panel_skin, area),
            Self::IOError => {
                debug!("nothing to display: IOError");
                // FIXME clear area
                Ok(())
            }
        }
    }
    pub fn display_info(
        &mut self,
        w: &mut W,
        screen: &Screen,
        panel_skin: &PanelSkin,
        area: &Area,
    ) -> Result<(), ProgramError> {
        match self {
            Self::Syntactic(sv) => sv.display_info(w, screen, panel_skin, area),
            //Self::Hex(hv) => hv.display(w, screen, panel_skin, area),
            _ => Ok(()),
        }
    }
}
