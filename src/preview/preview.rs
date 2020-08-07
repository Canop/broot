
use {
    super::PreviewMode,
    crate::{
        app::{AppContext, LineNumber},
        command::{ScrollCommand},
        display::{Screen, W},
        errors::ProgramError,
        hex::HexView,
        image::ImageView,
        pattern::InputPattern,
        skin::PanelSkin,
        syntactic::SyntacticView,
        task_sync::Dam,
    },
    std::path::Path,
    termimad::Area,
};

pub enum Preview {
    Image(ImageView),
    Syntactic(SyntacticView),
    Hex(HexView),
    IOError,
}

impl Preview {
    /// build a preview, never failing (but the preview can be Preview::IOError).
    /// If the prefered mode can't be applied, an other mode is chosen.
    pub fn new(
        path: &Path,
        prefered_mode: Option<PreviewMode>,
        con: &AppContext,
    ) -> Self {
        match prefered_mode {
            Some(PreviewMode::Hex) => Self::hex(path),
            Some(PreviewMode::Image) => Self::image(path),
            Some(PreviewMode::Text) => Self::unfiltered_text(path, con),
            None => {
                // automatic behavior: image, text, hex
                ImageView::new(path)
                    .map(Self::Image)
                    .unwrap_or_else(|_| Self::unfiltered_text(path, con))

            }
        }
    }
    /// try to build a preview with the designed mode, return an error
    /// if that wasn't possible
    pub fn with_mode(
        path: &Path,
        mode: PreviewMode,
        con: &AppContext,
    ) -> Result<Self, ProgramError> {
        match mode {
            PreviewMode::Hex => {
                Ok(HexView::new(path.to_path_buf()).map(Self::Hex)?)
            }
            PreviewMode::Image => {
                ImageView::new(path).map(Self::Image)
            }
            PreviewMode::Text => {
                Ok(SyntacticView::new(path, InputPattern::none(), &mut Dam::unlimited(), con)
                    .transpose()
                    .expect("syntactic view without pattern shouldn't be none")
                    .map(Self::Syntactic)?)
            }
        }
    }
    /// build an image view, unless the file can't be interpreted
    /// as an image, in which case a hex view is used
    pub fn image(path: &Path) -> Self {
        ImageView::new(path).ok()
            .map(Self::Image)
            .unwrap_or_else(|| Self::hex(path))

    }
    /// build a text preview (maybe with syntaxic coloring) if possible,
    /// a hex (binary) view if content isnt't UTF8, or a IOError when
    /// there's a IO problem
    pub fn unfiltered_text(
        path: &Path,
        con: &AppContext,
    ) -> Self {
        match SyntacticView::new(path, InputPattern::none(), &mut Dam::unlimited(), con) {
            Ok(Some(sv)) => Self::Syntactic(sv),
            // not previewable as UTF8 text
            // we'll try reading it as binary
            _ => Self::hex(path),
        }
    }
    /// try to build a filtered text view. Will return None if
    /// the dam gets an event before it's built
    pub fn filtered(
        &self,
        path: &Path,
        pattern: InputPattern,
        dam: &mut Dam,
        con: &AppContext,
    ) -> Option<Self> {
        match self {
            Self::Syntactic(_) => {
                match SyntacticView::new(path, pattern, dam, con) {

                    // normal finished loading
                    Ok(Some(sv)) => Some(Self::Syntactic(sv)),

                    // interrupted search
                    Ok(None) => None,

                    // not previewable as UTF8 text
                    // we'll try reading it as binary
                    Err(_) => Some(Self::hex(path)),
                }
            }
            _ => None, // not filterable
        }
    }
    /// return a hex_view, suitable for binary, or Self::IOError
    /// if there was an error
    pub fn hex(path: &Path) -> Self {
        match HexView::new(path.to_path_buf()) {
            Ok(reader) => Self::Hex(reader),
            Err(e) => {
                warn!("error while previewing {:?} : {:?}", path, e);
                Self::IOError
            }
        }
    }
    /// return the preview_mode, or None if we're on IOError
    pub fn get_mode(&self) -> Option<PreviewMode> {
        match self {
            Self::Image(_) => Some(PreviewMode::Image),
            Self::Syntactic(_) => Some(PreviewMode::Text),
            Self::Hex(_) => Some(PreviewMode::Hex),
            Self::IOError => None,
        }
    }
    pub fn pattern(&self) -> InputPattern {
        match self {
            Self::Syntactic(sv) => sv.pattern.clone(),
            _ => InputPattern::none(),
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

    pub fn get_selected_line_number(&self) -> Option<LineNumber> {
        match self {
            Self::Syntactic(sv) => sv.get_selected_line_number(),
            _ => None,
        }
    }
    pub fn try_select_line_number(&mut self, number: usize) -> bool {
        match self {
            Self::Syntactic(sv) => sv.try_select_line_number(number),
            _ => false,
        }
    }
    pub fn unselect(&mut self) {
        if let Self::Syntactic(sv) = self {
            sv.unselect();
        }
    }
    pub fn try_select_y(&mut self, y: u16) -> bool {
        match self {
            Self::Syntactic(sv) => sv.try_select_y(y),
            _ => false,
        }
    }
    pub fn select_previous_line(&mut self) {
        match self {
            Self::Syntactic(sv) => sv.select_previous_line(),
            Self::Hex(hv) => {
                hv.try_scroll(ScrollCommand::Lines(-1));
            }
            _ => {}
        }
    }
    pub fn select_next_line(&mut self) {
        match self {
            Self::Syntactic(sv) => sv.select_next_line(),
            Self::Hex(hv) => {
                hv.try_scroll(ScrollCommand::Lines(1));
            }
            _ => {}
        }
    }
    pub fn select_first(&mut self) {
        match self {
            Self::Syntactic(sv) => sv.select_first(),
            Self::Hex(hv) => hv.select_first(),
            _ => {}
        }
    }
    pub fn select_last(&mut self) {
        match self {
            Self::Syntactic(sv) => sv.select_last(),
            Self::Hex(hv) => hv.select_last(),
            _ => {}
        }
    }
    pub fn display(
        &mut self,
        w: &mut W,
        screen: &Screen,
        panel_skin: &PanelSkin,
        area: &Area,
        con: &AppContext,
    ) -> Result<(), ProgramError> {
        match self {
            Self::Image(iv) => iv.display(w, screen, panel_skin, area, con),
            Self::Syntactic(sv) => sv.display(w, screen, panel_skin, area),
            Self::Hex(hv) => hv.display(w, screen, panel_skin, area),
            Self::IOError => {
                debug!("nothing to display: IOError");
                // FIXME clear area - but it's hard to fall on that case
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
            Self::Image(iv) => iv.display_info(w, screen, panel_skin, area),
            Self::Syntactic(sv) => sv.display_info(w, screen, panel_skin, area),
            Self::Hex(hv) => hv.display_info(w, screen, panel_skin, area),
            _ => Ok(()),
        }
    }
}
