use {
    super::*,
    crate::{
        app::{AppContext, LineNumber},
        command::{ScrollCommand, move_sel},
        display::{Screen, W},
        errors::*,
        pattern::{InputPattern, NameMatch},
        skin::PanelSkin,
        task_sync::Dam,
    },
    crokey::crossterm::{
        cursor,
        style::{Color, Print, SetBackgroundColor, SetForegroundColor},
        QueueableCommand,
    },
    memmap2::Mmap,
    once_cell::sync::Lazy,
    std::{
        fs::File,
        io::{BufRead, BufReader},
        path::{Path, PathBuf},
        str,
    },
    syntect::highlighting::Style,
    termimad::{Area, CropWriter, SPACE_FILLING},
};

/// Homogeneously colored piece of a line
#[derive(Debug)]
pub struct Region {
    pub fg: Color,
    pub string: String,
}

/// when the file is bigger, we don't style it and we don't keep
/// it in memory: we just keep the offsets of the lines in the
/// file.
const MAX_SIZE_FOR_STYLING: u64 = 2_000_000;

impl Region {
    pub fn from_syntect(region: &(Style, &str)) -> Self {
        let fg = Color::Rgb {
            r: region.0.foreground.r,
            g: region.0.foreground.g,
            b: region.0.foreground.b,
        };
        let string = region.1.to_string();
        Self { fg, string }
    }
}

#[derive(Debug)]
pub struct Line {
    pub number: LineNumber,   // starting at 1
    pub start: usize,         // offset in the file, in bytes
    pub len: usize,           // len in bytes
    pub regions: Vec<Region>, // not always computed
    pub name_match: Option<NameMatch>,
}

pub struct SyntacticView {
    pub path: PathBuf,
    pub pattern: InputPattern,
    lines: Vec<Line>,
    scroll: usize,
    page_height: usize,
    selection_idx: Option<usize>, // index in lines of the selection, if any
    total_lines_count: usize,     // including lines not filtered out
}

impl SyntacticView {

    /// Return a prepared text view with syntax coloring if possible.
    /// May return Ok(None) only when a pattern is given and there
    /// was an event before the end of filtering.
    pub fn new(
        path: &Path,
        pattern: InputPattern,
        dam: &mut Dam,
        con: &AppContext,
        no_style: bool,
    ) -> Result<Option<Self>, ProgramError> {
        let mut sv = Self {
            path: path.to_path_buf(),
            pattern,
            lines: Vec::new(),
            scroll: 0,
            page_height: 0,
            selection_idx: None,
            total_lines_count: 0,
        };
        if sv.read_lines(dam, con, no_style)? {
            sv.select_first();
            Ok(Some(sv))
        } else {
            Ok(None)
        }
    }

    /// Return true when there was no interruption
    fn read_lines(
        &mut self,
        dam: &mut Dam,
        con: &AppContext,
        no_style: bool,
    ) -> Result<bool, ProgramError> {
        let f = File::open(&self.path)?;
        {
            // if we detect the file isn't mappable, we'll
            // let the ZeroLenFilePreview try to read it
            let mmap = unsafe { Mmap::map(&f) };
            if mmap.is_err() {
                return Err(ProgramError::UnmappableFile);
            }
        }
        let md = f.metadata()?;
        if md.len() == 0 {
            return Err(ProgramError::ZeroLenFile);
        }
        let with_style = !no_style && md.len() < MAX_SIZE_FOR_STYLING;
        let mut reader = BufReader::new(f);
        self.lines.clear();
        let mut line = String::new();
        self.total_lines_count = 0;
        let mut offset = 0;
        let mut number = 0;
        static SYNTAXER: Lazy<Syntaxer> = Lazy::new(Syntaxer::default);
        let mut highlighter = if with_style {
            SYNTAXER.highlighter_for(&self.path, con)
        } else {
            None
        };
        let pattern = &self.pattern.pattern;
        while reader.read_line(&mut line)? > 0 {
            number += 1;
            self.total_lines_count += 1;
            let start = offset;
            offset += line.len();
            for c in line.chars() {
                if !is_char_printable(c) {
                    debug!("unprintable char: {:?}", c);
                    return Err(ProgramError::UnprintableFile);
                }
            }
            // We don't remove '\n' or '\r' at this point because some syntax sets
            // need them for correct detection of comments. See #477
            // Those chars are removed on printing
            if pattern.is_empty() || pattern.score_of_string(&line).is_some() {
                let name_match = pattern.search_string(&line);
                let regions = if let Some(highlighter) = highlighter.as_mut() {
                    highlighter
                        .highlight(&line, &SYNTAXER.syntax_set)
                        .map_err(|e| ProgramError::SyntectCrashed { details: e.to_string() })?
                        .iter()
                        .map(Region::from_syntect)
                        .collect()
                } else {
                    Vec::new()
                };
                self.lines.push(Line {
                    regions,
                    start,
                    len: line.len(),
                    name_match,
                    number,
                });
            }
            line.clear();
            if dam.has_event() {
                info!("event interrupted preview filtering");
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Give the count of lines which can be seen when scrolling,
    /// total count including filtered ones
    pub fn line_counts(&self) -> (usize, usize) {
        (self.lines.len(), self.total_lines_count)
    }

    fn ensure_selection_is_visible(&mut self) {
        if self.page_height >= self.lines.len() {
            self.scroll = 0;
        } else if let Some(idx) = self.selection_idx {
            let padding = self.padding();
            if idx < self.scroll + padding || idx + padding > self.scroll + self.page_height {
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

    pub fn get_selected_line(&self) -> Option<String> {
        self.selection_idx
            .and_then(|idx| self.lines.get(idx))
            .and_then(|line| {
                File::open(&self.path)
                    .and_then(|file| unsafe { Mmap::map(&file) })
                    .ok()
                    .filter(|mmap| mmap.len() >= line.start + line.len)
                    .and_then(|mmap| {
                        String::from_utf8(
                            (mmap[line.start..line.start + line.len]).to_vec(),
                        ).ok()
                    })
            })
    }

    pub fn get_selected_line_number(&self) -> Option<LineNumber> {
        self.selection_idx
            .map(|idx| self.lines[idx].number)
    }
    pub fn unselect(&mut self) {
        self.selection_idx = None;
    }
    pub fn try_select_y(&mut self, y: u16) -> bool {
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

    pub fn try_select_line_number(&mut self, number: LineNumber) -> bool {
        // this could obviously be optimized
        for (idx, line) in self.lines.iter().enumerate() {
            if line.number == number {
                self.selection_idx = Some(idx);
                self.ensure_selection_is_visible();
                return true;
            }
        }
        false
    }

    pub fn move_selection(&mut self, dy: i32, cycle: bool) {
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
            } else  if idx >= old_scroll && idx < old_scroll + self.page_height {
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
        con: &AppContext,
    ) -> Result<(), ProgramError> {
        if area.height as usize != self.page_height {
            self.page_height = area.height as usize;
            self.ensure_selection_is_visible();
        }
        let max_number_len = self.lines.last().map_or(0, |l|l.number).to_string().len();
        let show_line_number = area.width > 55 || ( self.pattern.is_some() && area.width > 8 );
        let line_count = area.height as usize;
        let styles = &panel_skin.styles;
        let normal_fg  = styles.preview.get_fg()
            .or_else(|| styles.default.get_fg())
            .unwrap_or(Color::AnsiValue(252));
        let normal_bg = styles.preview.get_bg()
            .or_else(|| styles.default.get_bg())
            .unwrap_or(Color::AnsiValue(238));
        let selection_bg = styles.selected_line.get_bg()
            .unwrap_or(Color::AnsiValue(240));
        let match_bg = styles.preview_match.get_bg().unwrap_or(Color::AnsiValue(28));
        let code_width = area.width as usize - 1; // 1 char left for scrollbar
        let scrollbar = area.scrollbar(self.scroll, self.lines.len());
        let scrollbar_fg = styles.scrollbar_thumb.get_fg()
            .or_else(|| styles.preview.get_fg())
            .unwrap_or(Color::White);
        for y in 0..line_count {
            w.queue(cursor::MoveTo(area.left, y as u16 + area.top))?;
            let mut cw = CropWriter::new(w, code_width);
            let line_idx = self.scroll + y;
            let selected = self.selection_idx == Some(line_idx);
            let bg = if selected { selection_bg } else { normal_bg };
            let mut op_mmap: Option<Mmap> = None;
            if let Some(line) = self.lines.get(line_idx) {
                let mut regions = &line.regions;
                let regions_ur;
                if regions.is_empty() && line.len > 0 {
                    if op_mmap.is_none() {
                        let file = File::open(&self.path)?;
                        let mmap = unsafe { Mmap::map(&file)? };
                        op_mmap = Some(mmap);
                    }
                    if op_mmap.as_ref().unwrap().len() < line.start + line.len {
                        warn!("file truncated since parsing");
                    } else {
                        // an UTF8 error can only happen if file modified during display
                        let string = String::from_utf8(
                            // we copy the memmap slice, as it's not immutable
                            (op_mmap.unwrap()[line.start..line.start + line.len]).to_vec(),
                        )
                        .unwrap_or_else(|_| "Bad UTF8".to_string());
                        regions_ur = vec![Region {
                            fg: normal_fg,
                            string,
                        }];
                        regions = &regions_ur;
                    }
                }
                cw.w.queue(SetBackgroundColor(bg))?;
                if show_line_number {
                    cw.queue_g_string(
                        &styles.preview_line_number,
                        format!(" {:w$} ", line.number, w = max_number_len),
                    )?;
                } else {
                    cw.queue_unstyled_str(" ")?;
                }
                cw.w.queue(SetBackgroundColor(bg))?;
                if con.show_selection_mark {
                    cw.queue_unstyled_char(if selected { '▶' } else { ' ' })?;
                }
                if let Some(nm) = &line.name_match {
                    let mut dec = 0;
                    let pos = &nm.pos;
                    let mut pos_idx: usize = 0;
                    for content in regions {
                        let s = content.string.trim_end_matches(is_char_end_of_line);
                        cw.w.queue(SetForegroundColor(content.fg))?;
                        if pos_idx < pos.len() {
                            for (cand_idx, cand_char) in s.chars().enumerate() {
                                if pos_idx < pos.len() && pos[pos_idx] == cand_idx + dec {
                                    cw.w.queue(SetBackgroundColor(match_bg))?;
                                    cw.queue_unstyled_char(cand_char)?;
                                    cw.w.queue(SetBackgroundColor(bg))?;
                                    pos_idx += 1;
                                } else {
                                    cw.queue_unstyled_char(cand_char)?;
                                }
                            }
                            dec += s.chars().count();
                        } else {
                            cw.queue_unstyled_str(s)?;
                        }
                    }
                } else {
                    for content in regions {
                        cw.w.queue(SetForegroundColor(content.fg))?;
                        cw.queue_unstyled_str(content.string.trim_end_matches(is_char_end_of_line))?;
                    }
                }
            }
            cw.fill(
                if selected { &styles.selected_line } else { &styles.preview },
                &SPACE_FILLING,
            )?;
            w.queue(SetBackgroundColor(bg))?;
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
        let mut s = if self.pattern.is_some() {
            format!("{}/{}", self.lines.len(), self.total_lines_count)
        } else {
            format!("{}", self.total_lines_count)
        };
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

fn is_thumb(y: usize, scrollbar: Option<(u16, u16)>) -> bool {
    scrollbar.map_or(false, |(sctop, scbottom)| {
        let y = y as u16;
        sctop <= y && y <= scbottom
    })
}

/// Tell whether the character is normal enough to be displayed by the
/// syntactic view (if not we'll use a hex view)
fn is_char_printable(c: char) -> bool {
    // the tab is printable because it's replaced by spaces
    c == '\t' || c == '\n' || c == '\r' || !c.is_control()
}

fn is_char_end_of_line(c: char) -> bool {
    c == '\n' || c == '\r'
}

