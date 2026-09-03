use {
    super::*,
    crate::{
        app::{
            AppContext,
            LineNumber,
        },
        command::ScrollCommand,
        display::{
            Overflow,
            Rows,
            Screen,
            TAB_WIDTH,
            Viewport,
            W,
            is_thumb,
            row_starts,
        },
        errors::*,
        pattern::{
            InputPattern,
            NameMatch,
        },
        skin::PanelSkin,
        task_sync::Dam,
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
    once_cell::sync::Lazy,
    std::{
        borrow::Cow,
        fs::File,
        io::{
            BufRead,
            BufReader,
        },
        path::{
            Path,
            PathBuf,
        },
        str,
    },
    syntect::highlighting::Style,
    termimad::{
        Area,
        CropWriter,
        Filling,
        SPACE_FILLING,
    },
};

pub static SEPARATOR_FILLING: Lazy<Filling> = Lazy::new(|| Filling::from_char('─'));

/// Homogeneously colored piece of a line
#[derive(Debug, Clone)]
pub struct Region {
    pub fg: Color,
    pub string: String,
}

/// when the file is bigger, we don't style it and we don't keep
/// it in memory: we just keep the offsets of the lines in the file.
const MAX_SIZE_FOR_STYLING: u64 = 2_000_000;

/// Size of what's initially loaded (rest is loaded when user in background)
/// Must be greater than MAX_SIZE_FOR_STYLING
const INITIAL_LOAD: usize = 4_000_000;

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
pub enum DisplayLine {
    Content(Line),
    Separator,
}

#[derive(Debug)]
pub struct Line {
    pub number: LineNumber,   // starting at 1
    pub start: usize,         // offset in the file, in bytes
    pub len: usize,           // len in bytes
    pub regions: Vec<Region>, // not always computed
    pub name_match: Option<NameMatch>,
}

/// A text viewer, which can display a text file with syntax coloring if it's not too big.
///
/// In some cases, only the beginning of the file is read at first, and the rest is read
/// in background.
pub struct TextView {
    pub path: PathBuf,
    pub pattern: InputPattern,
    lines: Vec<DisplayLine>,
    /// the file, when lines aren't kept in memory
    mmap: Option<Mmap>,
    viewport: Viewport,
    content_lines_count: usize,   // number of lines excluding separators
    total_lines_count: usize,     // including lines not filtered out
    partial: bool,
}

impl DisplayLine {
    pub fn line_number(&self) -> Option<LineNumber> {
        match self {
            DisplayLine::Content(line) => Some(line.number),
            DisplayLine::Separator => None,
        }
    }
    pub fn is_match(&self) -> bool {
        match self {
            DisplayLine::Content(line) => line.name_match.is_some(),
            DisplayLine::Separator => false,
        }
    }
}

/// The lines of a text view, for layout by the viewport
struct TextRows<'v> {
    lines: &'v [DisplayLine],
    mmap: Option<&'v Mmap>,
}
impl TextRows<'_> {
    /// Return the content of a line not kept in memory
    fn read(
        &self,
        line: &Line,
    ) -> Option<String> {
        self.mmap
            .and_then(|mmap| mmap.get(line.start..line.start + line.len))
            // we copy the slice, as the file may change
            .and_then(|bytes| String::from_utf8(bytes.to_vec()).ok())
            .map(|s| printable_line(&s).into_owned())
    }
}
impl Rows for TextRows<'_> {
    fn len(&self) -> usize {
        self.lines.len()
    }
    fn row_count(
        &self,
        idx: usize,
        width: usize,
    ) -> usize {
        match &self.lines[idx] {
            DisplayLine::Separator => 1,
            DisplayLine::Content(line) if !line.regions.is_empty() => crate::display::row_count(
                line.regions.iter().flat_map(|r| r.string.chars()),
                width,
            ),
            DisplayLine::Content(line) => self
                .read(line)
                .map_or(1, |s| crate::display::row_count(s.chars(), width)),
        }
    }
    fn width_hint(
        &self,
        idx: usize,
    ) -> usize {
        match &self.lines[idx] {
            DisplayLine::Separator => 1,
            // the length in bytes overestimates the width in cells,
            // exactly for ASCII, without reading the line
            DisplayLine::Content(line) => line.len,
        }
    }
}

impl TextView {
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
        let allow_partial = pattern.is_none();
        let mut sv = Self {
            path: path.to_path_buf(),
            pattern,
            lines: Vec::new(),
            mmap: None,
            viewport: Viewport::default(),
            content_lines_count: 0,
            total_lines_count: 0,
            partial: false,
        };
        if sv.read_lines(dam, con, no_style, allow_partial)? {
            sv.viewport.select_first(&TextRows {
                lines: &sv.lines,
                mmap: sv.mmap.as_ref(),
            });
            Ok(Some(sv))
        } else {
            Ok(None)
        }
    }

    pub fn is_partial(&self) -> bool {
        self.partial
    }
    /// If the load was partial, complete it now
    pub fn complete_loading(
        &mut self,
        con: &AppContext,
        dam: &mut Dam,
    ) -> Result<(), ProgramError> {
        if self.partial {
            self.partial = false;
            self.read_lines(dam, con, true, false)?;
        }
        Ok(())
    }

    /// Return true when there was no interruption
    fn read_lines(
        &mut self,
        dam: &mut Dam,
        con: &AppContext,
        no_style: bool,
        initial_load: bool,
    ) -> Result<bool, ProgramError> {
        let f = File::open(&self.path)?;
        // if we detect the file isn't mappable, we'll
        // let the ZeroLenFilePreview try to read it
        let mmap = unsafe { Mmap::map(&f) }.map_err(|_| ProgramError::UnmappableFile)?;
        let md = f.metadata()?;
        if md.len() == 0 {
            return Err(ProgramError::ZeroLenFile);
        }
        let with_style = !no_style && md.len() < MAX_SIZE_FOR_STYLING;
        let mut reader = BufReader::new(f);
        let mut content_lines = Vec::new();
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
        // lines are read back from the file when they aren't stored styled
        self.mmap = if highlighter.is_some() { None } else { Some(mmap) };
        let pattern = &self.pattern.pattern;
        while reader.read_line(&mut line)? > 0 {
            number += 1;
            self.total_lines_count += 1;
            let start = offset;
            offset += line.len();
            // We clean the line to prevent TTY rendering from being broken.
            // We don't remove '\n' or '\r' at this point because some syntax sets
            // need them for correct detection of comments. See #477
            // Those chars are removed on printing, later on.
            let clean_line = printable_line(&line);
            let name_match = pattern.search_string(&clean_line);
            let regions = if let Some(highlighter) = highlighter.as_mut() {
                highlighter
                    .highlight_line(&clean_line, &SYNTAXER.syntax_set)
                    .map_err(|e| ProgramError::SyntectCrashed {
                        details: e.to_string(),
                    })?
                    .iter()
                    .map(Region::from_syntect)
                    .collect()
            } else {
                Vec::new()
            };
            content_lines.push(Line {
                regions,
                start,
                len: line.len(),
                name_match,
                number,
            });
            line.clear();
            if dam.has_event() {
                info!("event interrupted preview filtering");
                self.partial = true;
                return Ok(false);
            }
            if initial_load && offset > INITIAL_LOAD {
                info!("partial load");
                self.partial = true;
                break;
            }
        }
        let mut must_add_separators = false;
        if !pattern.is_empty() {
            let lines_before = con.lines_before_match_in_preview;
            let lines_after = con.lines_after_match_in_preview;
            if lines_before + lines_after > 0 {
                let mut kept = vec![false; content_lines.len()];
                for (i, line) in content_lines.iter().enumerate() {
                    if line.name_match.is_some() {
                        for j in i.saturating_sub(lines_before)
                            ..(i + lines_after + 1).min(content_lines.len())
                        {
                            kept[j] = true;
                        }
                    }
                }
                for i in 1..kept.len() - 1 {
                    if !kept[i] && kept[i - 1] && kept[i + 1] {
                        kept[i] = true;
                    }
                }
                content_lines.retain(|line| kept[line.number - 1]);
                must_add_separators = true;
            } else {
                content_lines.retain(|line| line.name_match.is_some());
            }
        }
        self.lines.clear();
        self.content_lines_count = content_lines.len();
        for line in content_lines {
            if must_add_separators {
                if let Some(last_number) = self.lines.last().and_then(|l| l.line_number()) {
                    if line.number > last_number + 1 {
                        self.lines.push(DisplayLine::Separator);
                    }
                }
            }
            self.lines.push(DisplayLine::Content(line));
        }
        Ok(true)
    }

    /// Give the count of lines which can be seen when scrolling,
    /// total count including filtered ones
    pub fn line_counts(&self) -> (usize, usize) {
        (self.lines.len(), self.total_lines_count)
    }

    pub fn get_selected_line(&self) -> Option<String> {
        self.viewport
            .selection()
            .and_then(|idx| self.lines.get(idx))
            .and_then(|line| match line {
                DisplayLine::Content(line) => Some(line),
                DisplayLine::Separator => None,
            })
            .and_then(|line| {
                let mapped;
                let mmap = match &self.mmap {
                    Some(mmap) => mmap,
                    None => {
                        mapped = File::open(&self.path)
                            .and_then(|file| unsafe { Mmap::map(&file) })
                            .ok()?;
                        &mapped
                    }
                };
                TextRows {
                    lines: &self.lines,
                    mmap: Some(mmap),
                }
                .read(line)
            })
    }

    pub fn get_selected_line_number(&self) -> Option<LineNumber> {
        self.viewport
            .selection()
            .and_then(|idx| self.lines[idx].line_number())
    }
    pub fn try_select_y(
        &mut self,
        y: u16,
    ) -> bool {
        self.viewport.try_select_y(y, &TextRows {
            lines: &self.lines,
            mmap: self.mmap.as_ref(),
        })
    }

    pub fn select_first(&mut self) {
        self.viewport.select_first(&TextRows {
            lines: &self.lines,
            mmap: self.mmap.as_ref(),
        });
    }
    pub fn select_last(&mut self) {
        self.viewport.select_last(&TextRows {
            lines: &self.lines,
            mmap: self.mmap.as_ref(),
        });
    }

    pub fn try_select_line_number(
        &mut self,
        number: LineNumber,
    ) -> bool {
        // this could obviously be optimized
        for (idx, line) in self.lines.iter().enumerate() {
            if line.line_number() == Some(number) {
                let rows = TextRows {
                    lines: &self.lines,
                    mmap: self.mmap.as_ref(),
                };
                self.viewport.select(idx, &rows);
                return true;
            }
        }
        false
    }

    pub fn move_selection(
        &mut self,
        dy: i32,
        cycle: bool,
    ) {
        self.viewport.move_selection(dy, cycle, &TextRows {
            lines: &self.lines,
            mmap: self.mmap.as_ref(),
        });
    }

    pub fn previous_match(&mut self) {
        let s = self.viewport.selection().unwrap_or(0);
        for d in 1..self.lines.len() {
            let idx = (self.lines.len() + s - d) % self.lines.len();
            if self.lines[idx].is_match() {
                let rows = TextRows {
                    lines: &self.lines,
                    mmap: self.mmap.as_ref(),
                };
                self.viewport.select(idx, &rows);
                return;
            }
        }
    }
    pub fn next_match(&mut self) {
        let s = self.viewport.selection().unwrap_or(0);
        for d in 1..self.lines.len() {
            let idx = (s + d) % self.lines.len();
            if self.lines[idx].is_match() {
                let rows = TextRows {
                    lines: &self.lines,
                    mmap: self.mmap.as_ref(),
                };
                self.viewport.select(idx, &rows);
                return;
            }
        }
    }

    pub fn try_scroll(
        &mut self,
        cmd: ScrollCommand,
    ) -> bool {
        self.viewport.try_scroll(cmd, &TextRows {
            lines: &self.lines,
            mmap: self.mmap.as_ref(),
        })
    }

    pub fn max_line_number(&self) -> Option<LineNumber> {
        for line in self.lines.iter().rev() {
            if let Some(n) = line.line_number() {
                return Some(n);
            }
        }
        None
    }

    pub fn get_content_line(
        &self,
        idx: usize,
    ) -> Option<&Line> {
        self.lines.get(idx).and_then(|line| match line {
            DisplayLine::Content(line) => Some(line),
            DisplayLine::Separator => None,
        })
    }

    pub fn display(
        &mut self,
        w: &mut W,
        _screen: Screen,
        panel_skin: &PanelSkin,
        area: &Area,
        con: &AppContext,
        overflow: Overflow,
    ) -> Result<(), ProgramError> {
        let rows = TextRows {
            lines: &self.lines,
            mmap: self.mmap.as_ref(),
        };
        let max_number_len = self.max_line_number().unwrap_or(0).to_string().len();
        let show_line_number = area.width > 55 || (self.pattern.is_some() && area.width > 8);
        let code_width = area.width as usize - 1; // 1 char left for scrollbar
        let gutter_width = if show_line_number { max_number_len + 2 } else { 1 }
            + usize::from(con.show_selection_mark);
        let text_width = code_width.saturating_sub(gutter_width).max(1);
        self.viewport
            .set_layout(area.height as usize, text_width, overflow, &rows);
        let positions = self.viewport.visible_rows(&rows);
        let scrollbar = self.viewport.scrollbar(area, &rows);
        let styles = &panel_skin.styles;
        let normal_fg = styles
            .preview
            .get_fg()
            .or_else(|| styles.default.get_fg())
            .unwrap_or(Color::Reset);
        let normal_bg = styles
            .preview
            .get_bg()
            .or_else(|| styles.default.get_bg())
            .unwrap_or(Color::Reset);
        let selection_bg = styles
            .selected_line
            .get_bg()
            .unwrap_or(Color::AnsiValue(240));
        let match_bg = styles
            .preview_match
            .get_bg()
            .unwrap_or(Color::AnsiValue(28));
        let scrollbar_fg = styles
            .scrollbar_thumb
            .get_fg()
            .or_else(|| styles.preview.get_fg())
            .unwrap_or(Color::White);
        // regions and row starts of the line being drawn, computed once per line
        let mut laid_out: Option<(usize, Cow<'_, [Region]>, Vec<usize>)> = None;
        for y in 0..area.height as usize {
            w.queue(cursor::MoveTo(area.left, y as u16 + area.top))?;
            let mut cw = CropWriter::new(w, code_width);
            let Some(&pos) = positions.get(y) else {
                cw.fill(&styles.preview, &SPACE_FILLING)?;
                w.queue(SetBackgroundColor(normal_bg))?;
                w.queue(Print(' '))?;
                continue;
            };
            let selected = self.viewport.is_selected(pos.line);
            let bg = if selected { selection_bg } else { normal_bg };
            match &self.lines[pos.line] {
                DisplayLine::Separator => {
                    cw.w.queue(SetBackgroundColor(bg))?;
                    cw.queue_unstyled_str(" ")?;
                    cw.fill(&styles.preview_separator, &SEPARATOR_FILLING)?;
                }
                DisplayLine::Content(line) => {
                    if laid_out.as_ref().is_none_or(|(idx, _, _)| *idx != pos.line) {
                        let regions = if line.regions.is_empty() && line.len > 0 {
                            match rows.read(line) {
                                Some(string) => Cow::Owned(vec![Region {
                                    fg: normal_fg,
                                    string,
                                }]),
                                None => {
                                    warn!("file truncated since parsing");
                                    Cow::Owned(Vec::new())
                                }
                            }
                        } else {
                            Cow::Borrowed(line.regions.as_slice())
                        };
                        let starts = if overflow == Overflow::Wrap {
                            row_starts(regions.iter().flat_map(|r| r.string.chars()), text_width)
                        } else {
                            Vec::new()
                        };
                        laid_out = Some((pos.line, regions, starts));
                    }
                    let (_, regions, starts) = laid_out.as_ref().unwrap();
                    let regions: &[Region] = regions;
                    cw.w.queue(SetBackgroundColor(bg))?;
                    if show_line_number {
                        if pos.sub == 0 {
                            cw.queue_g_string(
                                &styles.preview_line_number,
                                format!(" {:w$} ", line.number, w = max_number_len),
                            )?;
                        } else {
                            cw.queue_g_string(
                                &styles.preview_line_number,
                                " ".repeat(max_number_len + 2),
                            )?;
                        }
                    } else {
                        cw.queue_unstyled_str(" ")?;
                    }
                    cw.w.queue(SetBackgroundColor(bg))?;
                    if con.show_selection_mark {
                        cw.queue_unstyled_char(if selected && pos.sub == 0 { '▶' } else { ' ' })?;
                    }
                    // chars of the line displayed on this row
                    let from = if pos.sub == 0 { 0 } else { starts[pos.sub - 1] };
                    let to = starts.get(pos.sub).copied().unwrap_or(usize::MAX);
                    let pos_list = line.name_match.as_ref().map(|nm| &nm.pos);
                    if overflow == Overflow::NoWrap && pos_list.is_none() {
                        for region in regions {
                            cw.w.queue(SetForegroundColor(region.fg))?;
                            let s = region.string.trim_end_matches(is_char_end_of_line);
                            if s.contains('\t') {
                                cw.queue_unstyled_str(&s.replace('\t', &" ".repeat(TAB_WIDTH)))?;
                            } else {
                                cw.queue_unstyled_str(s)?;
                            }
                        }
                    } else {
                        // chars are buffered into runs of same style
                        let pos_list = pos_list.map(|v| v.as_slice()).unwrap_or(&[]);
                        let mut pos_idx = pos_list.partition_point(|&p| p < from);
                        let mut ci = 0; // index of the char in the line
                        let mut run = String::new();
                        'regions: for region in regions {
                            flush(&mut cw, &mut run)?;
                            cw.w.queue(SetForegroundColor(region.fg))?;
                            for c in region.string.chars() {
                                if ci >= to {
                                    break 'regions;
                                }
                                if ci >= from && !is_char_end_of_line(c) {
                                    let matched = pos_list.get(pos_idx) == Some(&ci);
                                    if matched {
                                        flush(&mut cw, &mut run)?;
                                        cw.w.queue(SetBackgroundColor(match_bg))?;
                                        pos_idx += 1;
                                    }
                                    if c == '\t' {
                                        run.extend(std::iter::repeat_n(' ', TAB_WIDTH));
                                    } else {
                                        run.push(c);
                                    }
                                    if matched {
                                        flush(&mut cw, &mut run)?;
                                        cw.w.queue(SetBackgroundColor(bg))?;
                                    }
                                }
                                ci += 1;
                            }
                        }
                        flush(&mut cw, &mut run)?;
                    }
                }
            }
            cw.fill(
                if selected {
                    &styles.selected_line
                } else {
                    &styles.preview
                },
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

    fn info(
        &self,
        width: usize,
    ) -> String {
        if self.is_partial() {
            let s = "loading...";
            let s  = if s.len() > width {
                ""
            } else {
                s
            };
            return s.to_string();
        }
        let mut s = if self.pattern.is_some() {
            format!("{}/{}", self.content_lines_count, self.total_lines_count)
        } else {
            format!("{}", self.total_lines_count)
        };
        if s.len() > width {
            return "".to_string();
        }
        if s.len() + "lines: ".len() < width {
            s = format!("lines: {s}");
        }
        s
    }

    pub fn display_info(
        &mut self,
        w: &mut W,
        _screen: Screen,
        panel_skin: &PanelSkin,
        area: &Area,
    ) -> Result<(), ProgramError> {
        let width = area.width as usize;
        let s = self.info(width);
        w.queue(cursor::MoveTo(
            area.left + area.width - s.len() as u16,
            area.top,
        ))?;
        panel_skin.styles.default.queue(w, s)?;
        Ok(())
    }
}

/// Tell whether the character must be replaced to prevent rendering from being broken
pub fn is_char_unprintable(c: char) -> bool {
    match c {
        '\u{8}' => true, // backspace
        '\u{b}'..='\u{e}' => true,
        '\u{84}'..='\u{85}' => true,
        '\u{1a}'..'\u{1c}' => true,
        '\u{89}'..='\u{9f}' => true,
        _ => false,
    }
}

fn printable_line(line: &str) -> Cow<'_, str> {
    if line.chars().any(is_char_unprintable) {
        let replacement = line.replace(is_char_unprintable, "�");
        Cow::Owned(replacement)
    } else {
        Cow::Borrowed(line)
    }
}

/// Write the buffered run of chars, if any
fn flush(
    cw: &mut CropWriter<'_, W>,
    run: &mut String,
) -> Result<(), ProgramError> {
    if !run.is_empty() {
        cw.queue_unstyled_str(run)?;
        run.clear();
    }
    Ok(())
}

fn is_char_end_of_line(c: char) -> bool {
    c == '\n' || c == '\r'
}
