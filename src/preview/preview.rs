use {
    super::*,
    crate::{
        app::*,
        command::ScrollCommand,
        display::*,
        errors::ProgramError,
        hex::HexView,
        image::ImageView,
        pattern::InputPattern,
        skin::PanelSkin,
        syntactic::TextView,
        task_sync::Dam,
        tty::TtyView,
    },
    crokey::crossterm::{
        QueueableCommand,
        cursor,
    },
    std::{
        io,
        path::Path,
    },
    termimad::{
        Area,
        CropWriter,
        SPACE_FILLING,
    },
};

#[allow(clippy::large_enum_variant)]
pub enum Preview {
    Dir(DirView),
    Image(ImageView),
    Text(TextView),
    Hex(HexView),
    Tty(TtyView),
    ZeroLen(ZeroLenFileView),
    IoError(io::Error),
}

impl Preview {
    /// build a preview, never failing (but the preview can be Preview::IOError).
    /// If the preferred mode can't be applied, an other mode is chosen.
    pub fn new(
        path: &Path,
        preferred_mode: Option<PreviewMode>,
        con: &AppContext,
    ) -> Self {
        if path.is_file() {
            match preferred_mode {
                Some(PreviewMode::Hex) => Self::hex(path),
                Some(PreviewMode::Image) => Self::image(path),
                Some(PreviewMode::Text) => Self::unfiltered_text(path, con),
                Some(PreviewMode::Tty) => Self::tty(path),
                None => {
                    // automatic behavior: image, text, hex
                    ImageView::new(path)
                        .map(Self::Image)
                        .unwrap_or_else(|_| Self::unfiltered_text(path, con))
                }
            }
        } else {
            Self::dir(path, InputPattern::none(), &Dam::unlimited(), con)
        }
    }
    /// try to build a preview with the designed mode, return an error
    /// if that wasn't possible
    pub fn with_mode(
        path: &Path,
        mode: PreviewMode,
        con: &AppContext,
    ) -> Result<Self, ProgramError> {
        if path.is_file() {
            match mode {
                PreviewMode::Hex => Ok(HexView::new(path.to_path_buf()).map(Self::Hex)?),
                PreviewMode::Image => ImageView::new(path).map(Self::Image),
                PreviewMode::Tty => TtyView::new(path)
                    .map(Self::Tty)
                    .map_err(ProgramError::from),
                PreviewMode::Text => Ok(TextView::new(
                    path,
                    InputPattern::none(),
                    &mut Dam::unlimited(),
                    con,
                    false,
                )
                .transpose()
                .expect("syntactic view without pattern shouldn't be none")
                .map(Self::Text)?),
            }
        } else {
            Ok(Self::dir(
                path,
                InputPattern::none(),
                &Dam::unlimited(),
                con,
            ))
        }
    }

    /// build a dir preview
    pub fn dir(
        path: &Path,
        pattern: InputPattern,
        dam: &Dam,
        con: &AppContext,
    ) -> Self {
        match DirView::new(path.to_path_buf(), pattern, dam, con) {
            Ok(dv) => Self::Dir(dv),
            Err(e) => Self::IoError(e),
        }
    }

    /// build an image view, unless the file can't be interpreted
    /// as an image, in which case a hex view is used
    pub fn image(path: &Path) -> Self {
        ImageView::new(path)
            .ok()
            .map(Self::Image)
            .unwrap_or_else(|| Self::hex(path))
    }

    /// build an tty view, unless there's an IO error
    pub fn tty(path: &Path) -> Self {
        match TtyView::new(path) {
            Ok(tv) => Self::Tty(tv),
            Err(e) => Self::IoError(e),
        }
    }

    /// build a text preview (maybe with syntaxic coloring) if possible,
    /// a hex (binary) view if content isnt't UTF8, a ZeroLen file if there's
    /// no length (it's probably a linux pseudofile) or a IOError when
    /// there's a IO problem
    pub fn unfiltered_text(
        path: &Path,
        con: &AppContext,
    ) -> Self {
        match TextView::new(
            path,
            InputPattern::none(),
            &mut Dam::unlimited(),
            con,
            false,
        ) {
            Ok(Some(sv)) => Self::Text(sv),
            Err(ProgramError::ZeroLenFile | ProgramError::UnmappableFile) => {
                debug!("zero len or unmappable file - check if system file");
                Self::ZeroLen(ZeroLenFileView::new(path.to_path_buf()))
            }
            Err(ProgramError::SyntectCrashed { details }) => {
                warn!("syntect crashed with message : {details:?}");
                Self::unstyled_text(path, con)
            }
            // not previewable as UTF8 text
            // we'll try reading it as binary
            Err(ProgramError::UnprintableFile) => Self::hex(path),
            _ => Self::hex(path),
        }
    }
    /// build a text preview with no syntax highlighting, if possible
    pub fn unstyled_text(
        path: &Path,
        con: &AppContext,
    ) -> Self {
        match TextView::new(path, InputPattern::none(), &mut Dam::unlimited(), con, true) {
            Ok(Some(sv)) => Self::Text(sv),
            Err(ProgramError::ZeroLenFile | ProgramError::UnmappableFile) => {
                debug!("zero len or unmappable file - check if system file");
                Self::ZeroLen(ZeroLenFileView::new(path.to_path_buf()))
            }
            // not previewable as UTF8 text - we'll try reading it as binary
            Err(ProgramError::UnprintableFile) => Self::hex(path),
            _ => Self::hex(path),
        }
    }
    /// try to build a filtered view. Will return None if
    /// the dam gets an event before it's built
    pub fn filtered(
        &self,
        path: &Path,
        pattern: InputPattern,
        dam: &mut Dam,
        con: &AppContext,
    ) -> Option<Self> {
        if path.is_file() {
            match self {
                Self::Text(_) => {
                    match TextView::new(path, pattern, dam, con, false) {
                        // normal finished loading
                        Ok(Some(sv)) => Some(Self::Text(sv)),

                        // interrupted search
                        Ok(None) => None,

                        // not previewable as UTF8 text
                        // we'll try reading it as binary
                        Err(_) => Some(Self::hex(path)), // FIXME try as unstyled if syntect crashed
                    }
                }
                _ => None, // not filterable
            }
        } else {
            Some(Self::dir(path, pattern, dam, con))
        }
    }
    /// return a hex_view, suitable for binary, or Self::IOError
    /// if there was an error
    pub fn hex(path: &Path) -> Self {
        match HexView::new(path.to_path_buf()) {
            Ok(reader) => Self::Hex(reader),
            Err(e) => {
                // it's unlikely as the file isn't open at this point
                warn!("error while previewing {:?} : {:?}", path, e);
                Self::IoError(e)
            }
        }
    }
    /// Return true when the preview is based on a temporarily incomplete
    /// loading or computing
    pub fn is_partial(&self) -> bool {
        match self {
            Self::Text(sv) => sv.is_partial(),
            _ => false,
        }
    }
    pub fn complete_loading(
        &mut self,
        con: &AppContext,
        dam: &mut Dam,
    ) -> Result<(), ProgramError> {
        match self {
            Self::Text(sv) => sv.complete_loading(con, dam),
            _ => Ok(()),
        }
    }
    /// return the preview_mode, or None if we're on IOError or Directory
    pub fn get_mode(&self) -> Option<PreviewMode> {
        match self {
            Self::Image(_) => Some(PreviewMode::Image),
            Self::Text(_) => Some(PreviewMode::Text),
            Self::ZeroLen(_) => Some(PreviewMode::Text),
            Self::Hex(_) => Some(PreviewMode::Hex),
            Self::Tty(_) => Some(PreviewMode::Tty),
            Self::IoError(_) => None,
            Self::Dir(_) => None,
        }
    }
    pub fn pattern(&self) -> InputPattern {
        match self {
            Self::Dir(dv) => dv.tree.options.pattern.clone(),
            Self::Text(sv) => sv.pattern.clone(),
            _ => InputPattern::none(),
        }
    }
    pub fn try_scroll(
        &mut self,
        cmd: ScrollCommand,
    ) -> bool {
        match self {
            Self::Dir(dv) => dv.try_scroll(cmd),
            Self::Text(sv) => sv.try_scroll(cmd),
            Self::Hex(hv) => hv.try_scroll(cmd),
            Self::Tty(v) => v.try_scroll(cmd),
            _ => false,
        }
    }
    pub fn is_filterable(&self) -> bool {
        matches!(self, Self::Text(_) | Self::Dir(_))
    }

    pub fn get_selected_line(&self) -> Option<String> {
        match self {
            Self::Text(sv) => sv.get_selected_line(),
            _ => None,
        }
    }
    pub fn get_selected_line_number(&self) -> Option<LineNumber> {
        match self {
            Self::Text(sv) => sv.get_selected_line_number(),
            _ => None,
        }
    }
    pub fn try_select_line_number(
        &mut self,
        number: usize,
    ) -> bool {
        match self {
            Self::Text(sv) => sv.try_select_line_number(number),
            _ => false,
        }
    }
    pub fn unselect(&mut self) {
        match self {
            Self::Text(sv) => sv.unselect(),
            Self::Tty(tv) => tv.unselect(),
            _ => {}
        }
    }
    pub fn try_select_y(
        &mut self,
        y: u16,
    ) -> bool {
        match self {
            Self::Dir(dv) => dv.try_select_y(y),
            Self::Text(sv) => sv.try_select_y(y),
            Self::Tty(v) => v.try_select_y(y),
            _ => false,
        }
    }
    pub fn move_selection(
        &mut self,
        dy: i32,
        cycle: bool,
    ) {
        match self {
            Self::Dir(dv) => dv.move_selection(dy, cycle),
            Self::Text(sv) => sv.move_selection(dy, cycle),
            Self::Tty(v) => v.move_selection(dy, cycle),
            Self::Hex(hv) => {
                hv.try_scroll(ScrollCommand::Lines(dy));
            }
            _ => {}
        }
    }

    pub fn previous_match(&mut self) {
        if let Self::Text(sv) = self {
            sv.previous_match();
        } else {
            self.move_selection(-1, true);
        }
    }
    pub fn next_match(&mut self) {
        if let Self::Text(sv) = self {
            sv.next_match();
        } else {
            self.move_selection(1, true);
        }
    }

    pub fn select_first(&mut self) {
        match self {
            Self::Dir(dv) => dv.select_first(),
            Self::Text(sv) => sv.select_first(),
            Self::Hex(hv) => hv.select_first(),
            Self::Tty(v) => v.select_first(),
            _ => {}
        }
    }
    pub fn select_last(&mut self) {
        match self {
            Self::Text(sv) => sv.select_last(),
            Self::Hex(hv) => hv.select_last(),
            Self::Tty(v) => v.select_last(),
            _ => {}
        }
    }
    pub fn display(
        &mut self,
        w: &mut W,
        disc: &DisplayContext,
        area: &Area,
    ) -> Result<(), ProgramError> {
        let panel_skin = &disc.panel_skin;
        let screen = disc.screen;
        let con = &disc.con;
        match self {
            Self::Dir(dv) => dv.display(w, disc, area),
            Self::Image(iv) => time!(iv.display(w, disc, area)),
            Self::Text(sv) => sv.display(w, screen, panel_skin, area, con),
            Self::ZeroLen(zlv) => zlv.display(w, screen, panel_skin, area),
            Self::Hex(hv) => hv.display(w, screen, panel_skin, area),
            Self::Tty(v) => v.display(w, screen, panel_skin, area),
            Self::IoError(err) => {
                let mut y = area.top;
                w.queue(cursor::MoveTo(area.left, y))?;
                let mut cw = CropWriter::new(w, area.width as usize);
                cw.queue_str(&panel_skin.styles.default, "An error prevents the preview:")?;
                cw.fill(&panel_skin.styles.default, &SPACE_FILLING)?;
                y += 1;
                w.queue(cursor::MoveTo(area.left, y))?;
                let mut cw = CropWriter::new(w, area.width as usize);
                cw.queue_g_string(&panel_skin.styles.status_error, err.to_string())?;
                cw.fill(&panel_skin.styles.default, &SPACE_FILLING)?;
                y += 1;
                while y < area.top + area.height {
                    w.queue(cursor::MoveTo(area.left, y))?;
                    let mut cw = CropWriter::new(w, area.width as usize);
                    cw.fill(&panel_skin.styles.default, &SPACE_FILLING)?;
                    y += 1;
                }
                Ok(())
            }
        }
    }
    pub fn display_info(
        &mut self,
        w: &mut W,
        screen: Screen,
        panel_skin: &PanelSkin,
        area: &Area,
    ) -> Result<(), ProgramError> {
        match self {
            Self::Dir(dv) => dv.display_info(w, screen, panel_skin, area),
            Self::Image(iv) => iv.display_info(w, screen, panel_skin, area),
            Self::Text(sv) => sv.display_info(w, screen, panel_skin, area),
            Self::Hex(hv) => hv.display_info(w, screen, panel_skin, area),
            _ => Ok(()),
        }
    }
}
