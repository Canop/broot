use {
    super::wrap::Overflow,
    crate::command::{
        ScrollCommand,
        move_sel,
    },
    termimad::Area,
};

/// Position of a screen row: a line of the view, and a row in this
/// line (always 0 when lines aren't wrapped)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct RowPos {
    pub line: usize,
    pub sub: usize,
}

/// The lines of a view, as needed by the viewport to lay them out
pub trait Rows {
    fn len(&self) -> usize;
    /// Return the number of screen rows the line takes when wrapped
    /// at this width (at least 1)
    fn row_count(
        &self,
        idx: usize,
        width: usize,
    ) -> usize;
    /// Return a cheap approximation of the width of the line in cells,
    /// used only to estimate the row count of lines which aren't laid
    /// out, for the scrollbar. It doesn't have to be exact: scroll
    /// positions are computed from real row counts.
    fn width_hint(
        &self,
        idx: usize,
    ) -> usize;
}

/// Scroll and selection state of a view made of lines, some of them
/// displayed in a page, possibly wrapped.
///
/// Only the lines around the displayed page are ever laid out,
/// which keeps operations cheap whatever the size of the view.
#[derive(Debug, Default, Clone)]
pub struct Viewport {
    scroll: RowPos,
    page_height: usize,
    width: usize,
    overflow: Overflow,
    selection: Option<usize>,
    total_rows: Option<TotalRowsEstimate>,
}

/// Estimated total number of rows of a view, valid only for the
/// layout it was computed for
#[derive(Debug, Clone, Copy)]
struct TotalRowsEstimate {
    /// number of lines of the view
    len: usize,
    /// wrapping width, in cells
    width: usize,
    /// estimated total number of rows
    rows: usize,
}

impl TotalRowsEstimate {
    fn is_for(
        &self,
        len: usize,
        width: usize,
    ) -> bool {
        self.len == len && self.width == width
    }
}

impl Viewport {
    pub fn scroll(&self) -> RowPos {
        self.scroll
    }
    pub fn page_height(&self) -> usize {
        self.page_height
    }
    pub fn selection(&self) -> Option<usize> {
        self.selection
    }
    pub fn is_selected(
        &self,
        idx: usize,
    ) -> bool {
        self.selection == Some(idx)
    }
    /// Select the line and scroll if needed to make it visible
    pub fn select(
        &mut self,
        idx: usize,
        rows: &impl Rows,
    ) {
        self.selection = Some(idx);
        self.ensure_selection_is_visible(rows);
    }
    /// Set the dimensions of the page and whether lines are wrapped,
    /// keeping the selection visible
    pub fn set_layout(
        &mut self,
        page_height: usize,
        width: usize,
        overflow: Overflow,
        rows: &impl Rows,
    ) {
        let width = width.max(1);
        if page_height == self.page_height && width == self.width && overflow == self.overflow {
            return;
        }
        self.page_height = page_height;
        self.width = width;
        self.overflow = overflow;
        self.total_rows = None;
        self.scroll.sub = 0;
        self.ensure_selection_is_visible(rows);
    }
    /// Number of rows of the line, in the current layout
    fn rc(
        &self,
        rows: &impl Rows,
        line: usize,
    ) -> usize {
        if self.overflow == Overflow::Wrap && self.width > 0 {
            rows.row_count(line, self.width).max(1)
        } else {
            1
        }
    }
    fn last_pos(
        &self,
        rows: &impl Rows,
    ) -> Option<RowPos> {
        let len = rows.len();
        (len > 0).then(|| RowPos {
            line: len - 1,
            sub: self.rc(rows, len - 1) - 1,
        })
    }
    /// Move down n rows, returning None when there aren't enough rows
    fn step_down(
        &self,
        rows: &impl Rows,
        mut pos: RowPos,
        mut n: usize,
    ) -> Option<RowPos> {
        let len = rows.len();
        loop {
            if pos.line >= len {
                return None;
            }
            let rc = self.rc(rows, pos.line);
            if pos.sub + n < rc {
                pos.sub += n;
                return Some(pos);
            }
            n -= rc - pos.sub;
            pos = RowPos {
                line: pos.line + 1,
                sub: 0,
            };
        }
    }
    /// Move up n rows, stopping at the first row
    fn step_up(
        &self,
        rows: &impl Rows,
        mut pos: RowPos,
        mut n: usize,
    ) -> RowPos {
        loop {
            if n <= pos.sub {
                pos.sub -= n;
                return pos;
            }
            if pos.line == 0 {
                return RowPos::default();
            }
            n -= pos.sub + 1;
            pos.line -= 1;
            pos.sub = self.rc(rows, pos.line) - 1;
        }
    }
    /// Number of rows from `from` to `to`, which must not be before `from`
    fn distance(
        &self,
        rows: &impl Rows,
        from: RowPos,
        to: RowPos,
    ) -> usize {
        let mut d = 0;
        for line in from.line..to.line {
            d += self.rc(rows, line);
        }
        d + to.sub - from.sub
    }
    fn fits_in_page(
        &self,
        rows: &impl Rows,
    ) -> bool {
        let mut total = 0;
        for line in 0..rows.len() {
            total += self.rc(rows, line);
            if total > self.page_height {
                return false;
            }
        }
        true
    }
    /// The scroll position showing the last row at the bottom of the page
    fn max_scroll(
        &self,
        rows: &impl Rows,
    ) -> RowPos {
        match self.last_pos(rows) {
            Some(last) => self.step_up(rows, last, self.page_height.saturating_sub(1)),
            None => RowPos::default(),
        }
    }
    fn padding(&self) -> usize {
        (self.page_height / 4).min(4)
    }
    fn ensure_selection_is_visible(
        &mut self,
        rows: &impl Rows,
    ) {
        if self.page_height == 0 {
            return;
        }
        if self.fits_in_page(rows) {
            self.scroll = RowPos::default();
            return;
        }
        let Some(sel) = self.selection else {
            self.scroll = self.scroll.min(self.max_scroll(rows));
            return;
        };
        let padding = self.padding();
        let start = RowPos { line: sel, sub: 0 };
        let end = RowPos {
            line: sel,
            sub: self.rc(rows, sel) - 1,
        };
        if start < self.scroll || self.distance(rows, self.scroll, start) < padding {
            self.scroll = self.step_up(rows, start, padding);
        } else if self.distance(rows, self.scroll, end) + padding >= self.page_height {
            self.scroll = self.step_up(rows, end, self.page_height - 1 - padding);
            if start < self.scroll {
                // the line is taller than the page: show its start
                self.scroll = start;
            }
        }
        self.scroll = self.scroll.min(self.max_scroll(rows));
    }
    /// Scroll to the top of the view
    pub fn scroll_to_top(&mut self) {
        self.scroll = RowPos::default();
    }
    /// Scroll to show the last row at the bottom of the page
    pub fn scroll_to_bottom(
        &mut self,
        rows: &impl Rows,
    ) {
        self.scroll = self.max_scroll(rows);
    }
    /// Select the line displayed at the given y in the page, if any
    pub fn try_select_y(
        &mut self,
        y: u16,
        rows: &impl Rows,
    ) -> bool {
        match self.step_down(rows, self.scroll, y as usize) {
            Some(pos) => {
                self.selection = Some(pos.line);
                true
            }
            None => false,
        }
    }
    pub fn select_first(
        &mut self,
        rows: &impl Rows,
    ) {
        if rows.len() > 0 {
            self.selection = Some(0);
            self.scroll = RowPos::default();
        }
    }
    pub fn select_last(
        &mut self,
        rows: &impl Rows,
    ) {
        if rows.len() > 0 {
            self.select(rows.len() - 1, rows);
        }
    }
    pub fn move_selection(
        &mut self,
        dy: i32,
        cycle: bool,
        rows: &impl Rows,
    ) {
        if let Some(idx) = self.selection {
            self.selection = Some(move_sel(idx, rows.len(), dy, cycle));
        } else if rows.len() > 0 {
            self.selection = Some(0);
        }
        self.ensure_selection_is_visible(rows);
    }
    /// Scroll, moving the selection along when it was visible,
    /// and return whether something changed
    pub fn try_scroll(
        &mut self,
        cmd: ScrollCommand,
        rows: &impl Rows,
    ) -> bool {
        let n = cmd.to_lines(self.page_height);
        let old_scroll = self.scroll;
        let max_scroll = self.max_scroll(rows);
        self.scroll = if n < 0 {
            self.step_up(rows, self.scroll, n.unsigned_abs() as usize)
        } else {
            self.step_down(rows, self.scroll, n as usize)
                .unwrap_or(max_scroll)
                .min(max_scroll)
        };
        if let Some(idx) = self.selection {
            if self.scroll == old_scroll {
                let old_selection = self.selection;
                self.selection = Some(if cmd.is_up() { 0 } else { rows.len() - 1 });
                return self.selection != old_selection;
            }
            let sel_start = RowPos { line: idx, sub: 0 };
            if old_scroll <= sel_start {
                let y = self.distance(rows, old_scroll, sel_start);
                if y < self.page_height {
                    // the selection was visible: we keep it at the same y
                    self.selection = Some(
                        self.step_down(rows, self.scroll, y)
                            .map_or(rows.len() - 1, |pos| pos.line),
                    );
                }
            }
        }
        self.scroll != old_scroll
    }
    /// Return the positions of the rows of the page, from top to bottom
    pub fn visible_rows(
        &self,
        rows: &impl Rows,
    ) -> Vec<RowPos> {
        let mut positions = Vec::with_capacity(self.page_height);
        let mut pos = self.scroll;
        while positions.len() < self.page_height && pos.line < rows.len() {
            positions.push(pos);
            match self.step_down(rows, pos, 1) {
                Some(next) => pos = next,
                None => break,
            }
        }
        positions
    }
    /// Return the scrollbar of the page, based on an estimate of
    /// the row counts when lines are wrapped
    pub fn scrollbar(
        &mut self,
        area: &Area,
        rows: &impl Rows,
    ) -> Option<(u16, u16)> {
        let len = rows.len();
        if self.overflow == Overflow::NoWrap || self.width == 0 {
            return area.scrollbar(self.scroll.line, len);
        }
        let width = self.width;
        let estimate = |line: usize| rows.width_hint(line).max(1).div_ceil(width);
        let total = match self.total_rows {
            Some(tre) if tre.is_for(len, width) => tre.rows,
            _ => {
                let rows = (0..len).map(estimate).sum();
                self.total_rows = Some(TotalRowsEstimate { len, width, rows });
                rows
            }
        };
        let current = (0..self.scroll.line).map(estimate).sum::<usize>() + self.scroll.sub;
        area.scrollbar(current, total)
    }
}

/// Tell whether the row at this y (from the top of the screen)
/// is part of the scrollbar thumb
pub fn is_thumb(
    y: usize,
    scrollbar: Option<(u16, u16)>,
) -> bool {
    scrollbar.is_some_and(|(top, bottom)| {
        let y = y as u16;
        top <= y && y <= bottom
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// lines which are never wrapped
    struct UnwrappedRows(usize);
    impl Rows for UnwrappedRows {
        fn len(&self) -> usize {
            self.0
        }
        fn row_count(
            &self,
            _idx: usize,
            _width: usize,
        ) -> usize {
            1
        }
        fn width_hint(
            &self,
            _idx: usize,
        ) -> usize {
            1
        }
    }

    /// lines given by their width in cells
    struct Widths(Vec<usize>);
    impl Rows for Widths {
        fn len(&self) -> usize {
            self.0.len()
        }
        fn row_count(
            &self,
            idx: usize,
            width: usize,
        ) -> usize {
            self.0[idx].max(1).div_ceil(width)
        }
        fn width_hint(
            &self,
            idx: usize,
        ) -> usize {
            self.0[idx]
        }
    }
    fn pos(
        line: usize,
        sub: usize,
    ) -> RowPos {
        RowPos { line, sub }
    }

    #[test]
    fn unwrapped_selection_stays_visible() {
        let rows = UnwrappedRows(100);
        let mut vp = Viewport::default();
        vp.set_layout(10, 80, Overflow::NoWrap, &rows);
        vp.select_first(&rows);
        assert_eq!(vp.selection(), Some(0));
        assert_eq!(vp.scroll(), pos(0, 0));
        vp.select(50, &rows);
        let scroll = vp.scroll().line;
        assert!(scroll <= 50 && 50 < scroll + 10);
        vp.select_last(&rows);
        assert_eq!(vp.selection(), Some(99));
        assert_eq!(vp.scroll(), pos(90, 0));
        vp.move_selection(1, true, &rows);
        assert_eq!(vp.selection(), Some(0));
        assert_eq!(vp.scroll(), pos(0, 0));
        vp.move_selection(-1, false, &rows);
        assert_eq!(vp.selection(), Some(0));
    }

    #[test]
    fn unwrapped_scroll_moves_visible_selection() {
        let rows = UnwrappedRows(100);
        let mut vp = Viewport::default();
        vp.set_layout(10, 80, Overflow::NoWrap, &rows);
        vp.select_first(&rows);
        assert!(vp.try_scroll(ScrollCommand::Lines(3), &rows));
        assert_eq!(vp.scroll(), pos(3, 0));
        assert_eq!(vp.selection(), Some(3));
        assert!(vp.try_scroll(ScrollCommand::Pages(1), &rows));
        assert_eq!(vp.scroll(), pos(13, 0));
        assert_eq!(vp.selection(), Some(13));
        // the selection keeps its y while scrolling
        vp.select(85, &rows);
        assert_eq!(vp.scroll(), pos(78, 0));
        assert!(vp.try_scroll(ScrollCommand::Pages(1), &rows));
        assert_eq!(vp.scroll(), pos(88, 0));
        assert_eq!(vp.selection(), Some(95));
        // the last row stays at the bottom, then scrolling selects the last line
        assert!(vp.try_scroll(ScrollCommand::Pages(1), &rows));
        assert_eq!(vp.scroll(), pos(90, 0));
        assert_eq!(vp.selection(), Some(97));
        assert!(vp.try_scroll(ScrollCommand::Lines(1), &rows));
        assert_eq!(vp.scroll(), pos(90, 0));
        assert_eq!(vp.selection(), Some(99));
        assert!(!vp.try_scroll(ScrollCommand::Lines(1), &rows));
    }

    #[test]
    fn empty_view() {
        let rows = UnwrappedRows(0);
        let mut vp = Viewport::default();
        vp.set_layout(10, 80, Overflow::Wrap, &rows);
        vp.select_first(&rows);
        vp.select_last(&rows);
        vp.move_selection(1, true, &rows);
        assert_eq!(vp.selection(), None);
        assert!(!vp.try_select_y(0, &rows));
        assert!(vp.visible_rows(&rows).is_empty());
    }

    #[test]
    fn wrapped_rows() {
        // rows: 1, 3, 1, 2, 1
        let rows = Widths(vec![5, 25, 5, 15, 5]);
        let mut vp = Viewport::default();
        vp.set_layout(3, 10, Overflow::Wrap, &rows);
        vp.select_first(&rows);
        assert_eq!(
            vp.visible_rows(&rows),
            vec![pos(0, 0), pos(1, 0), pos(1, 1)]
        );
        assert!(vp.try_select_y(2, &rows));
        assert_eq!(vp.selection(), Some(1));
        assert!(!vp.try_select_y(3, &rows) || vp.selection() == Some(1));
        // selecting line 2 brings its row into view
        vp.select(2, &rows);
        assert_eq!(vp.scroll(), pos(1, 1));
        assert_eq!(
            vp.visible_rows(&rows),
            vec![pos(1, 1), pos(1, 2), pos(2, 0)]
        );
        // scrolling one row up keeps the selection at the same y
        assert!(vp.try_scroll(ScrollCommand::Lines(-1), &rows));
        assert_eq!(vp.scroll(), pos(1, 0));
        assert_eq!(vp.selection(), Some(1));
        // the last row of the view ends at the bottom of the page
        vp.select_last(&rows);
        assert_eq!(vp.scroll(), pos(3, 0));
        assert_eq!(
            vp.visible_rows(&rows),
            vec![pos(3, 0), pos(3, 1), pos(4, 0)]
        );
        // page down from the start
        vp.select_first(&rows);
        vp.try_scroll(ScrollCommand::Pages(1), &rows);
        assert_eq!(vp.scroll(), pos(1, 2));
        assert_eq!(vp.selection(), Some(1));
    }

    #[test]
    fn line_taller_than_page_shows_its_start() {
        let rows = Widths(vec![5, 100, 5]);
        let mut vp = Viewport::default();
        vp.set_layout(3, 10, Overflow::Wrap, &rows);
        vp.select_first(&rows);
        vp.move_selection(1, false, &rows);
        assert_eq!(vp.scroll(), pos(1, 0));
        vp.move_selection(1, false, &rows);
        assert_eq!(vp.scroll(), pos(1, 8));
    }

    #[test]
    fn layout_change_keeps_selection_visible() {
        let rows = Widths(vec![5, 25, 5, 15, 5]);
        let mut vp = Viewport::default();
        vp.set_layout(3, 10, Overflow::NoWrap, &rows);
        vp.select(4, &rows);
        assert_eq!(vp.scroll(), pos(2, 0));
        vp.set_layout(3, 10, Overflow::Wrap, &rows);
        assert_eq!(vp.scroll(), pos(3, 0));
        vp.set_layout(3, 10, Overflow::NoWrap, &rows);
        assert_eq!(vp.scroll(), pos(2, 0));
    }
}
