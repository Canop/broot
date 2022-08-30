use {
    crate::{
        display::{Screen, W},
        errors::ProgramError,
        skin::PanelSkin,
    },
    char_reader::CharReader,
    crokey::crossterm::{
        cursor,
        QueueableCommand,
    },
    std::{
        fs::File,
        path::PathBuf,
    },
    termimad::{Area, CropWriter, SPACE_FILLING},
};

/// a (light) display for a file declaring a size 0,
/// as happens for many system "files", for example in /proc
pub struct ZeroLenFileView {
    path: PathBuf,
}

impl ZeroLenFileView {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
    pub fn display(
        &mut self,
        w: &mut W,
        _screen: Screen,
        panel_skin: &PanelSkin,
        area: &Area,
    ) -> Result<(), ProgramError> {
        let styles = &panel_skin.styles;
        let line_count = area.height as usize;
        let file = File::open(&self.path)?;
        let mut reader = CharReader::new(file);
        // line_len here is in chars, and we crop in cols, but it's OK because both
        // are usually identical for system files and we crop later anyway
        let line_len = area.width as usize;
        for y in 0..line_count {
            w.queue(cursor::MoveTo(area.left, y as u16 + area.top))?;
            let mut cw = CropWriter::new(w, area.width as usize);
            let cw = &mut cw;
            if let Some(line) = reader.next_line(line_len, 15_000)? {
                cw.queue_str(&styles.default, &line)?;
            }
            cw.fill(&styles.default, &SPACE_FILLING)?;
        }
        Ok(())
    }
}
