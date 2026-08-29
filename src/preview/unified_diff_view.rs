use {
    crate::{
        app::{
            AppContext,
            LineNumber,
        },
        command::{
            ScrollCommand,
            move_sel,
        },
        display::{
            Screen,
            W,
        },
        errors::ProgramError,
        git::{
            DiffLineKind,
            FileDiff,
            FileDiffContent,
            diff_worktree_vs_head,
        },
        skin::PanelSkin,
        syntactic::SEPARATOR_FILLING,
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
    std::path::Path,
    termimad::{
        Area,
        CropWriter,
        SPACE_FILLING,
    },
};

/// A displayed row of the diff
#[derive(Debug, Clone, Copy)]
enum Row {
    /// indexes of the hunk and of the line in the hunk
    Line(usize, usize),
    /// a separator between two hunks
    Separator,
}

/// A preview showing the changes of a file since the last commit,
/// as a unified diff
pub struct UnifiedDiffView {
    diff: FileDiff,
    rows: Vec<Row>,
    scroll: usize,
    page_height: usize,
    selection_idx: Option<usize>,
}

impl UnifiedDiffView {
    pub fn new(
        path: &Path,
        con: &AppContext,
    ) -> Result<Self, ProgramError> {
        let diff = diff_worktree_vs_head(path, con.lines_around_diff_hunks)?;
        let mut rows = Vec::new();
        if let FileDiffContent::Text(hunks) = &diff.content {
            for (hunk_idx, hunk) in hunks.iter().enumerate() {
                if hunk_idx > 0 {
                    rows.push(Row::Separator);
                }
                for line_idx in 0..hunk.lines.len() {
                    rows.push(Row::Line(hunk_idx, line_idx));
                }
            }
        }
        Ok(Self {
            diff,
            rows,
            scroll: 0,
            page_height: 0,
            selection_idx: None,
        })
    }
    fn line(
        &self,
        row: Row,
    ) -> Option<&crate::git::DiffLine> {
        match (row, &self.diff.content) {
            (Row::Line(hunk_idx, line_idx), FileDiffContent::Text(hunks)) => {
                hunks.get(hunk_idx).and_then(|hunk| hunk.lines.get(line_idx))
            }
            _ => None,
        }
    }
    fn max_line_number(&self) -> usize {
        match &self.diff.content {
            FileDiffContent::Text(hunks) => hunks
                .last()
                .map(|hunk| {
                    (hunk.old_start + hunk.old_len).max(hunk.new_start + hunk.new_len)
                })
                .unwrap_or(0),
            _ => 0,
        }
    }
    fn ensure_selection_is_visible(&mut self) {
        if self.page_height >= self.rows.len() {
            self.scroll = 0;
        } else if let Some(idx) = self.selection_idx {
            let padding = self.padding();
            if idx < self.scroll + padding || idx + padding > self.scroll + self.page_height {
                if idx <= padding {
                    self.scroll = 0;
                } else if idx + padding > self.rows.len() {
                    self.scroll = self.rows.len() - self.page_height;
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
    /// Return the number, in the current version of the file, of the
    /// selected line or of the closest following one
    pub fn get_selected_line_number(&self) -> Option<LineNumber> {
        let idx = self.selection_idx?;
        self.rows[idx..]
            .iter()
            .find_map(|&row| self.line(row).and_then(|line| line.new_number))
    }
    pub fn get_selected_line(&self) -> Option<String> {
        self.selection_idx
            .and_then(|idx| self.line(self.rows[idx]))
            .map(|line| line.content.clone())
    }
    pub fn try_select_y(
        &mut self,
        y: u16,
    ) -> bool {
        let idx = y as usize + self.scroll;
        if idx < self.rows.len() {
            self.selection_idx = Some(idx);
            true
        } else {
            false
        }
    }
    pub fn select_first(&mut self) {
        if !self.rows.is_empty() {
            self.selection_idx = Some(0);
            self.scroll = 0;
        }
    }
    pub fn select_last(&mut self) {
        if !self.rows.is_empty() {
            self.selection_idx = Some(self.rows.len() - 1);
            self.ensure_selection_is_visible();
        }
    }
    pub fn move_selection(
        &mut self,
        dy: i32,
        cycle: bool,
    ) {
        if let Some(idx) = self.selection_idx {
            self.selection_idx = Some(move_sel(idx, self.rows.len(), dy, cycle));
        } else if !self.rows.is_empty() {
            self.selection_idx = Some(0);
        }
        self.ensure_selection_is_visible();
    }
    pub fn try_scroll(
        &mut self,
        cmd: ScrollCommand,
    ) -> bool {
        let old_scroll = self.scroll;
        self.scroll = cmd.apply(self.scroll, self.rows.len(), self.page_height);
        if let Some(idx) = self.selection_idx {
            if self.scroll == old_scroll {
                let old_selection = self.selection_idx;
                if cmd.is_up() {
                    self.selection_idx = Some(0);
                } else {
                    self.selection_idx = Some(self.rows.len() - 1);
                }
                return self.selection_idx == old_selection;
            } else if idx >= old_scroll && idx < old_scroll + self.page_height {
                if idx + self.scroll < old_scroll {
                    self.selection_idx = Some(0);
                } else if idx + self.scroll - old_scroll >= self.rows.len() {
                    self.selection_idx = Some(self.rows.len() - 1);
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
        let styles = &panel_skin.styles;
        let number_len = self.max_line_number().to_string().len();
        let show_line_numbers = area.width as usize > 2 * number_len + 30;
        let code_width = area.width as usize - 1; // 1 char left for scrollbar
        let scrollbar = area.scrollbar(self.scroll, self.rows.len());
        let scrollbar_fg = styles
            .scrollbar_thumb
            .get_fg()
            .or_else(|| styles.preview.get_fg())
            .unwrap_or(Color::White);
        for y in 0..area.height as usize {
            w.queue(cursor::MoveTo(area.left, y as u16 + area.top))?;
            let mut cw = CropWriter::new(w, code_width);
            let row_idx = self.scroll + y;
            let selected = self.selection_idx == Some(row_idx);
            match self.rows.get(row_idx) {
                Some(Row::Separator) => {
                    cw.queue_unstyled_str(" ")?;
                    cw.fill(&styles.preview_separator, &SEPARATOR_FILLING)?;
                }
                Some(&row @ Row::Line(..)) => {
                    let Some(line) = self.line(row) else {
                        continue;
                    };
                    let (line_style, sign) = match line.kind {
                        DiffLineKind::Context => (&styles.preview, ' '),
                        DiffLineKind::Added => (&styles.diff_added, '+'),
                        DiffLineKind::Removed => (&styles.diff_removed, '-'),
                    };
                    let style = if selected {
                        &styles.selected_line
                    } else {
                        line_style
                    };
                    if show_line_numbers {
                        let number = |n: Option<usize>| match n {
                            Some(n) => format!("{n:>number_len$}"),
                            None => " ".repeat(number_len),
                        };
                        cw.queue_g_string(
                            &styles.diff_line_number,
                            format!(
                                " {} {} ",
                                number(line.old_number),
                                number(line.new_number),
                            ),
                        )?;
                    }
                    if con.show_selection_mark {
                        cw.queue_char(style, if selected { '▶' } else { ' ' })?;
                    }
                    cw.queue_char(style, sign)?;
                    cw.queue_char(style, ' ')?;
                    cw.queue_str(style, &line.content)?;
                    cw.fill(style, &SPACE_FILLING)?;
                }
                None => {
                    cw.fill(&styles.preview, &SPACE_FILLING)?;
                }
            }
            if is_thumb(y + area.top as usize, scrollbar) {
                w.queue(SetForegroundColor(scrollbar_fg))?;
                w.queue(Print('▐'))?;
            } else {
                let bg = styles
                    .preview
                    .get_bg()
                    .or_else(|| styles.default.get_bg())
                    .unwrap_or(Color::Reset);
                w.queue(SetBackgroundColor(bg))?;
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
        let styles = &panel_skin.styles;
        let width = area.width as usize;
        match &self.diff.content {
            FileDiffContent::Text(_) => {
                let insertions = format!("+{}", self.diff.insertions);
                let deletions = format!("-{}", self.diff.deletions);
                let len = insertions.len() + deletions.len();
                if len > width {
                    return Ok(());
                }
                w.queue(cursor::MoveTo(area.left + (width - len) as u16, area.top))?;
                styles.git_insertions.queue(w, insertions)?;
                styles.git_deletions.queue(w, deletions)?;
            }
            FileDiffContent::Binary | FileDiffContent::Unchanged => {
                let s = if matches!(self.diff.content, FileDiffContent::Binary) {
                    "binary"
                } else {
                    "no change"
                };
                if s.len() > width {
                    return Ok(());
                }
                w.queue(cursor::MoveTo(area.left + (width - s.len()) as u16, area.top))?;
                styles.default.queue(w, s)?;
            }
        }
        Ok(())
    }
}

fn is_thumb(
    y: usize,
    scrollbar: Option<(u16, u16)>,
) -> bool {
    scrollbar.is_some_and(|(top, bottom)| {
        let y = y as u16;
        top <= y && y <= bottom
    })
}
