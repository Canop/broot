use {
    super::*,
    crate::{
        command::{
            ScrollCommand,
            move_sel,
        },
        display::{
            Screen,
            W,
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
    scroll: usize,
    page_height: usize,
    selection_idx: Option<usize>, // index in lines of the selection, if any
    total_lines_count: usize,
}

impl TtyView {
    pub fn new(path: &Path) -> Result<Self, io::Error> {
        let mut sv = Self {
            path: path.to_path_buf(),
            lines: Vec::new(),
            scroll: 0,
            page_height: 0,
            selection_idx: None,
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

    fn ensure_selection_is_visible(&mut self) {
        if self.page_height >= self.lines.len() {
            self.scroll = 0;
        } else if let Some(idx) = self.selection_idx {
            let padding = self.padding();
            if idx < self.scroll + padding
                || idx + padding > self.scroll + self.page_height
            {
                if idx <= padding {
                    self.scroll = 0;
                } else if idx + padding > self.lines.len() {
                    self.scroll = self.lines.len() - self.page_height;
                } else if idx < self.scroll + self.page_height / 2 {
                    self.scroll = idx - padding;
                } else {
                    self.scroll = idx + padding - self.page_height;
                }
            }
        }
    }

    fn padding(&self) -> usize {
        (self.page_height / 4).min(4)
    }

    pub fn unselect(&mut self) {
        self.selection_idx = None;
    }
    pub fn try_select_y(
        &mut self,
        y: u16,
    ) -> bool {
        let idx = y as usize + self.scroll;
        if idx < self.lines.len() {
            self.selection_idx = Some(idx);
            true
        } else {
            false
        }
    }

    pub fn select_first(&mut self) {
        if !self.lines.is_empty() {
            self.selection_idx = Some(0);
            self.scroll = 0;
        }
    }
    pub fn select_last(&mut self) {
        self.selection_idx = Some(self.lines.len() - 1);
        if self.page_height < self.lines.len() {
            self.scroll = self.lines.len() - self.page_height;
        }
    }

    pub fn move_selection(
        &mut self,
        dy: i32,
        cycle: bool,
    ) {
        if let Some(idx) = self.selection_idx {
            self.selection_idx = Some(move_sel(idx, self.lines.len(), dy, cycle));
        } else if !self.lines.is_empty() {
            self.selection_idx = Some(0)
        }
        self.ensure_selection_is_visible();
    }

    pub fn try_scroll(
        &mut self,
        cmd: ScrollCommand,
    ) -> bool {
        let old_scroll = self.scroll;
        self.scroll = cmd.apply(self.scroll, self.lines.len(), self.page_height);
        if let Some(idx) = self.selection_idx {
            if self.scroll == old_scroll {
                let old_selection = self.selection_idx;
                if cmd.is_up() {
                    self.selection_idx = Some(0);
                } else {
                    self.selection_idx = Some(self.lines.len() - 1);
                }
                return self.selection_idx == old_selection;
            } else if idx >= old_scroll && idx < old_scroll + self.page_height {
                if idx + self.scroll < old_scroll {
                    self.selection_idx = Some(0);
                } else if idx + self.scroll - old_scroll >= self.lines.len() {
                    self.selection_idx = Some(self.lines.len() - 1);
                } else {
                    self.selection_idx = Some(idx + self.scroll - old_scroll);
                }
            }
        }
        self.scroll != old_scroll
    }

    pub fn display(
        &mut self,
        w: &mut W,
        _screen: Screen,
        panel_skin: &PanelSkin,
        area: &Area,
    ) -> Result<(), ProgramError> {
        if area.height as usize != self.page_height {
            self.page_height = area.height as usize;
            self.ensure_selection_is_visible();
        }
        let line_count = area.height as usize;
        let styles = &panel_skin.styles;
        let bg = styles
            .preview
            .get_bg()
            .or_else(|| styles.default.get_bg())
            .unwrap_or(Color::AnsiValue(238));
        let content_width = area.width as usize - 1; // 1 char left for scrollbar
        let scrollbar = area.scrollbar(self.scroll, self.lines.len());
        let scrollbar_fg = styles
            .scrollbar_thumb
            .get_fg()
            .or_else(|| styles.preview.get_fg())
            .unwrap_or(Color::White);
        for y in 0..line_count {
            let line_idx = self.scroll + y;
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
        w.queue(cursor::MoveTo(area.left + area.width - s.len() as u16, area.top))?;
        panel_skin.styles.default.queue(w, s)?;
        Ok(())
    }
}

fn is_thumb(
    y: usize,
    scrollbar: Option<(u16, u16)>,
) -> bool {
    scrollbar.map_or(false, |(sctop, scbottom)| {
        let y = y as u16;
        sctop <= y && y <= scbottom
    })
}
