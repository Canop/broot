use {
    super::*,
    crate::{
        command::ScrollCommand,
        display::{
            Screen,
            UnwrappedRows,
            Viewport,
            W,
            is_thumb,
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

impl TtyView {
    pub fn new(path: &Path) -> Result<Self, io::Error> {
        let mut sv = Self {
            path: path.to_path_buf(),
            lines: Vec::new(),
            viewport: Viewport::default(),
            total_lines_count: 0,
        };
        sv.read_lines()?;
        sv.select_first();
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

    pub fn unselect(&mut self) {
        self.viewport.unselect();
    }
    pub fn try_select_y(
        &mut self,
        y: u16,
    ) -> bool {
        self.viewport.try_select_y(y, &UnwrappedRows(self.lines.len()))
    }

    pub fn select_first(&mut self) {
        self.viewport.select_first(&UnwrappedRows(self.lines.len()));
    }
    pub fn select_last(&mut self) {
        self.viewport.select_last(&UnwrappedRows(self.lines.len()));
    }

    pub fn move_selection(
        &mut self,
        dy: i32,
        cycle: bool,
    ) {
        self.viewport.move_selection(dy, cycle, &UnwrappedRows(self.lines.len()));
    }

    pub fn try_scroll(
        &mut self,
        cmd: ScrollCommand,
    ) -> bool {
        self.viewport.try_scroll(cmd, &UnwrappedRows(self.lines.len()))
    }

    pub fn display(
        &mut self,
        w: &mut W,
        _screen: Screen,
        panel_skin: &PanelSkin,
        area: &Area,
    ) -> Result<(), ProgramError> {
        let rows = UnwrappedRows(self.lines.len());
        self.viewport
            .set_layout(area.height as usize, area.width as usize, false, &rows);
        let line_count = area.height as usize;
        let styles = &panel_skin.styles;
        let bg = styles
            .preview
            .get_bg()
            .or_else(|| styles.default.get_bg())
            .unwrap_or(Color::AnsiValue(238));
        let content_width = area.width as usize - 1; // 1 char left for scrollbar
        let scrollbar = self.viewport.scrollbar(area, &rows);
        let scrollbar_fg = styles
            .scrollbar_thumb
            .get_fg()
            .or_else(|| styles.preview.get_fg())
            .unwrap_or(Color::White);
        for y in 0..line_count {
            let line_idx = self.viewport.scroll().line + y;
            let mut allowed = content_width;
            w.queue(cursor::MoveTo(area.left, y as u16 + area.top))?;
            if let Some(tline) = self.lines.get(line_idx) {
                w.queue(SetBackgroundColor(bg))?;
                allowed -= tline.draw_in(w, allowed)?;
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
