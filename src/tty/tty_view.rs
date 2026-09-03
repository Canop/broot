use {
    super::*,
    crate::{
        command::ScrollCommand,
        display::{
            Overflow,
            Rows,
            Screen,
            Viewport,
            W,
            is_thumb,
            row_starts,
        },
        errors::*,
        skin::PanelSkin,
    },
    crokey::crossterm::{
        QueueableCommand,
        cursor,
        style::{
            Color,
            Print,
            SetBackgroundColor,
            SetForegroundColor,
        },
    },
    memmap2::Mmap,
    std::{
        fs::File,
        io::{
            self,
            BufRead,
            BufReader,
        },
        path::{
            Path,
            PathBuf,
        },
    },
    termimad::Area,
};

pub struct TtyView {
    pub path: PathBuf,
    lines: Vec<TLine>,
    viewport: Viewport,
    total_lines_count: usize,
}

/// The lines of a tty view, for layout by the viewport
struct TtyRows<'v> {
    lines: &'v [TLine],
}
impl Rows for TtyRows<'_> {
    fn len(&self) -> usize {
        self.lines.len()
    }
    fn row_count(
        &self,
        idx: usize,
        width: usize,
    ) -> usize {
        crate::display::row_count(
            self.lines[idx].strings.iter().flat_map(|ts| ts.raw.chars()),
            width,
        )
    }
    fn width_hint(
        &self,
        idx: usize,
    ) -> usize {
        // the length in bytes overestimates the width in cells,
        // exactly for ASCII
        self.lines[idx].strings.iter().map(|ts| ts.raw.len()).sum()
    }
}

impl TtyView {
    pub fn new(path: &Path) -> Result<Self, io::Error> {
        let mut sv = Self {
            path: path.to_path_buf(),
            lines: Vec::new(),
            viewport: Viewport::default(),
            total_lines_count: 0,
        };
        sv.read_lines()?;
        Ok(sv)
    }

    fn read_lines(&mut self) -> Result<(), io::Error> {
        let f = File::open(&self.path)?;
        {
            // if we detect the file isn't mappable, we'll
            // let the ZeroLenFilePreview try to read it
            let mmap = unsafe { Mmap::map(&f) };
            if mmap.is_err() {
                return Err(io::Error::other("unmappable file"));
            }
        }
        let md = f.metadata()?;
        if md.len() == 0 {
            return Err(io::Error::other("zero length file"));
        }
        let mut reader = BufReader::new(f);
        self.lines.clear();
        let mut line = String::new();
        self.total_lines_count = 0;
        while reader.read_line(&mut line)? > 0 {
            self.total_lines_count += 1;
            let tline = TLine::from_tty(&line);
            self.lines.push(tline);
            line.clear();
        }
        Ok(())
    }

    pub fn go_to_top(&mut self) {
        self.viewport.scroll_to_top();
    }
    pub fn go_to_bottom(&mut self) {
        let rows = TtyRows { lines: &self.lines };
        self.viewport.scroll_to_bottom(&rows);
    }

    pub fn try_scroll(
        &mut self,
        cmd: ScrollCommand,
    ) -> bool {
        let rows = TtyRows { lines: &self.lines };
        self.viewport.try_scroll(cmd, &rows)
    }

    pub fn display(
        &mut self,
        w: &mut W,
        _screen: Screen,
        panel_skin: &PanelSkin,
        area: &Area,
        overflow: Overflow,
    ) -> Result<(), ProgramError> {
        let rows = TtyRows { lines: &self.lines };
        let content_width = area.width as usize - 1; // 1 char left for scrollbar
        self.viewport
            .set_layout(area.height as usize, content_width, overflow, &rows);
        let positions = self.viewport.visible_rows(&rows);
        let styles = &panel_skin.styles;
        let bg = styles
            .preview
            .get_bg()
            .or_else(|| styles.default.get_bg())
            .unwrap_or(Color::Reset);
        let scrollbar = self.viewport.scrollbar(area, &rows);
        let scrollbar_fg = styles
            .scrollbar_thumb
            .get_fg()
            .or_else(|| styles.preview.get_fg())
            .unwrap_or(Color::White);
        // row starts of the line being drawn, computed once per line
        let mut laid_out: Option<(usize, Vec<usize>)> = None;
        for y in 0..area.height as usize {
            let mut allowed = content_width;
            w.queue(cursor::MoveTo(area.left, y as u16 + area.top))?;
            if let Some(&pos) = positions.get(y) {
                let tline = &self.lines[pos.line];
                w.queue(SetBackgroundColor(bg))?;
                allowed -= if overflow == Overflow::Wrap {
                    if laid_out.as_ref().is_none_or(|(idx, _)| *idx != pos.line) {
                        let chars = tline.strings.iter().flat_map(|ts| ts.raw.chars());
                        laid_out = Some((pos.line, row_starts(chars, content_width)));
                    }
                    let starts = &laid_out.as_ref().unwrap().1;
                    let from = if pos.sub == 0 { 0 } else { starts[pos.sub - 1] };
                    let to = starts.get(pos.sub).copied().unwrap_or(usize::MAX);
                    tline.draw_range_in(w, allowed, from, to)?
                } else {
                    tline.draw_in(w, allowed)?
                };
            }
            w.queue(SetBackgroundColor(bg))?;
            for _ in 0..allowed {
                w.queue(Print(' '))?;
            }
            if is_thumb(y + area.top as usize, scrollbar) {
                w.queue(SetForegroundColor(scrollbar_fg))?;
                w.queue(Print('▐'))?;
            } else {
                w.queue(Print(' '))?;
            }
        }
        Ok(())
    }

    pub fn display_info(
        &mut self,
        w: &mut W,
        _screen: Screen,
        panel_skin: &PanelSkin,
        area: &Area,
    ) -> Result<(), ProgramError> {
        let width = area.width as usize;
        let mut s = format!("{}", self.total_lines_count);
        if s.len() > width {
            return Ok(());
        }
        if s.len() + "lines: ".len() < width {
            s = format!("lines: {s}");
        }
        w.queue(cursor::MoveTo(
            area.left + area.width - s.len() as u16,
            area.top,
        ))?;
        panel_skin.styles.default.queue(w, s)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tty_row_counts() {
        let mut lines = Vec::new();
        let mut l = TLine::default();
        l.add_tstring("\u{1b}[31m", "abcde");
        l.add_tstring("\u{1b}[32m", "fghij");
        lines.push(l); // 10 cells
        let mut l = TLine::default();
        l.add_tstring("\u{1b}[36m", "日本");
        l.add_tstring("\u{1b}[35m", "語間");
        lines.push(l); // 4 wide chars, 8 cells
        let rows = TtyRows { lines: &lines };
        assert_eq!(rows.len(), 2);
        assert_eq!(rows.row_count(0, 10), 1);
        assert_eq!(rows.row_count(0, 5), 2);
        assert_eq!(rows.row_count(0, 4), 3);
        // a wide char doesn't straddle rows: 5 cells hold only 2 wide chars
        assert_eq!(rows.row_count(1, 8), 1);
        assert_eq!(rows.row_count(1, 5), 2);
        assert_eq!(rows.row_count(1, 3), 4);
        assert_eq!(rows.width_hint(0), 10);
    }
}
