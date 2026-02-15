use crate::{
    helper::char_width,
    view::line_wrapper::LineWrapper,
    view::LineNumbers,
    Lines,
};
use ratatui_core::layout::Rect;

/// Represents the (x, y) offset of the editor's viewport.
/// It represents the top-left local editor coordinate.
#[derive(Debug, Clone)]
pub(crate) struct ViewState {
    /// The offset of the viewport.
    pub(crate) viewport: Offset,
    /// When wrap is on, the number of visual lines to skip at the top of the
    /// first visible logical line (viewport.y). This allows sub-line scrolling
    /// within a wrapped paragraph.
    pub(crate) viewport_visual_skip: usize,
    /// The number of visual rows that are displayed on the viewport.
    /// When wrap is on, this counts screen rows (visual lines), not logical lines.
    pub(crate) num_rows: usize,
    /// Sets the area (starting upper-left corner of the terminal window) where
    /// the editor text is rendered to.
    ///
    /// Required to calculate the mouse position in relation to the text within the editor.
    pub(crate) screen_area: Rect,
    /// Whether the lines are wrapped.
    pub(crate) wrap: bool,
    /// The number of spaces used to display a tab.
    pub(crate) tab_width: usize,
    /// Line numbers configuration.
    pub(crate) line_numbers: LineNumbers,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            viewport: Offset::default(),
            viewport_visual_skip: 0,
            num_rows: 0,
            screen_area: Rect::default(),
            wrap: true,
            tab_width: 2,
            line_numbers: LineNumbers::None,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub(crate) struct Offset {
    /// The x-offset.
    pub(crate) x: usize,
    /// The y-offset.
    pub(crate) y: usize,
}

impl Offset {
    pub(crate) fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

impl From<Rect> for Offset {
    fn from(value: Rect) -> Self {
        Self {
            x: value.x as usize,
            y: value.y as usize,
        }
    }
}

impl ViewState {
    /// Sets the editors area on the screen.
    ///
    /// Equivalent to the upper left coordinate of the editor in the
    /// buffers coordinate system.
    pub(crate) fn set_screen_area<T: Into<Rect>>(&mut self, area: T) {
        self.screen_area = area.into();
    }

    /// Returns the content width used for wrapping (from screen_area).
    pub(crate) fn content_width(&self) -> usize {
        self.screen_area.width as usize
    }

    /// Returns the content height (from screen_area).
    pub(crate) fn content_height(&self) -> usize {
        self.screen_area.height as usize
    }

    /// Updates the viewports horizontal offset.
    pub(crate) fn update_viewport_horizontal(
        &mut self,
        width: usize,
        cursor_col: usize,
        line: Option<&Vec<char>>,
    ) -> usize {
        let Some(line) = line else {
            self.viewport.x = 0;
            return self.viewport.x;
        };

        // scroll left
        if cursor_col < self.viewport.x {
            self.viewport.x = cursor_col;
            return self.viewport.x;
        }

        // Iterate forward from the viewport.x position and calculate width
        let mut max_cursor_pos = self.viewport.x;
        let mut current_width = 0;
        for &ch in line.iter().skip(self.viewport.x) {
            current_width += char_width(ch, self.tab_width);
            if current_width >= width {
                break;
            }
            max_cursor_pos += 1;
        }

        // scroll right
        if cursor_col > max_cursor_pos {
            let mut backward_width = 0;
            let mut new_viewport_x = cursor_col;

            // Iterate backward from max_cursor_pos to find the first fitting character
            for i in (0..=cursor_col).rev() {
                let char_width = match line.get(i) {
                    Some(&ch) => char_width(ch, self.tab_width),
                    None => 1,
                };
                backward_width += char_width;
                if backward_width >= width {
                    break;
                }
                new_viewport_x = new_viewport_x.saturating_sub(1);
            }

            self.viewport.x = new_viewport_x;
        }

        self.viewport.x
    }

    /// Updates the view ports vertical offset.
    pub(crate) fn update_viewport_vertical(&mut self, height: usize, cursor_row: usize) -> usize {
        let max_cursor_pos = height.saturating_sub(1) + self.viewport.y;

        // scroll up
        if cursor_row < self.viewport.y {
            self.viewport.y = cursor_row;
        }

        // scroll down
        if cursor_row >= max_cursor_pos {
            self.viewport.y += cursor_row.saturating_sub(max_cursor_pos);
        }

        self.viewport.y
    }

    /// Updates the viewport's vertical offset when wrap is on.
    /// Uses visual-line space for all calculations.
    /// Returns (viewport.y, viewport_visual_skip).
    pub(crate) fn update_viewport_vertical_wrap(
        &mut self,
        width: usize,
        height: usize,
        cursor_row: usize,
        cursor_col: usize,
        lines: &Lines,
    ) -> (usize, usize) {
        self.ensure_cursor_visible_wrap(lines, width, height, cursor_row, cursor_col);
        (self.viewport.y, self.viewport_visual_skip)
    }

    /// Updates the number of rows that are currently shown on the viewport.
    /// Refers to the number of editor lines, not visual lines.
    pub(crate) fn update_num_rows(&mut self, num_rows: usize) {
        self.num_rows = num_rows;
    }

    /// Ensures the cursor is visible in the viewport when wrap is on.
    /// Works entirely in visual-line space.
    ///
    /// `cursor_row` and `cursor_col` are logical coordinates.
    /// `width` is the content area width, `height` is the content area height.
    fn ensure_cursor_visible_wrap(
        &mut self,
        lines: &Lines,
        width: usize,
        height: usize,
        cursor_row: usize,
        cursor_col: usize,
    ) {
        if height == 0 || width == 0 {
            return;
        }

        // Compute the cursor's global visual line index.
        let cursor_visual = cursor_to_visual_line(lines, width, self.tab_width, cursor_row, cursor_col);

        // Compute the viewport's global visual line start.
        let viewport_visual_start =
            logical_row_to_first_visual_line(lines, width, self.tab_width, self.viewport.y)
                + self.viewport_visual_skip;

        // Scroll up: cursor is above the viewport.
        if cursor_visual < viewport_visual_start {
            let (row, skip) = visual_line_to_logical(lines, width, self.tab_width, cursor_visual);
            self.viewport.y = row;
            self.viewport_visual_skip = skip;
            return;
        }

        // Scroll down: cursor is below the viewport.
        let viewport_visual_end = viewport_visual_start + height.saturating_sub(1);

        // When the cursor is on the last visible row and its display column
        // reaches (or exceeds) the content width, the cursor sits at the
        // right edge and would be rendered off-screen. Treat it as already
        // being on the next visual line so the viewport scrolls to show it.
        let effective_cursor_visual = if cursor_visual == viewport_visual_end {
            let display_col =
                cursor_display_col(lines, cursor_row, cursor_col, width, self.tab_width);
            if display_col >= width {
                cursor_visual + 1
            } else {
                cursor_visual
            }
        } else {
            cursor_visual
        };

        if effective_cursor_visual > viewport_visual_end {
            // The new viewport start should place the cursor on the last visible row.
            let new_start_visual =
                effective_cursor_visual.saturating_sub(height.saturating_sub(1));
            let (row, skip) = visual_line_to_logical(lines, width, self.tab_width, new_start_visual);
            self.viewport.y = row;
            self.viewport_visual_skip = skip;
        }
    }
}

// ============================================================================
// Visual line helpers (standalone functions)
// ============================================================================

/// Returns the number of visual (screen) lines a single logical line occupies
/// when wrapped at `width`. Uses the same word-boundary-aware wrapping as the
/// renderer so that line counts match exactly.
pub(crate) fn visual_line_count_for_row(line: &[char], width: usize, tab_width: usize) -> usize {
    if width == 0 {
        return 1;
    }
    // wrap_line returns [] for empty lines, but an empty line still occupies
    // one visual row on screen.
    LineWrapper::wrap_line(line, width, tab_width).len().max(1)
}

/// Returns the global visual line index of the first visual line of `logical_row`.
pub(crate) fn logical_row_to_first_visual_line(
    lines: &Lines,
    width: usize,
    tab_width: usize,
    logical_row: usize,
) -> usize {
    let mut visual = 0;
    for (i, line) in lines.iter_row().enumerate() {
        if i >= logical_row {
            break;
        }
        visual += visual_line_count_for_row(line, width, tab_width);
    }
    visual
}

/// Returns the global visual line index that the cursor (logical_row, logical_col)
/// maps to.
pub(crate) fn cursor_to_visual_line(
    lines: &Lines,
    width: usize,
    tab_width: usize,
    logical_row: usize,
    logical_col: usize,
) -> usize {
    let base = logical_row_to_first_visual_line(lines, width, tab_width, logical_row);

    // Within the logical row, determine which visual line the column is on.
    let line = match lines.get(jagged::index::RowIndex::new(logical_row)) {
        Some(line) => line,
        None => return base,
    };

    let wrapped = LineWrapper::wrap_line(line, width, tab_width);
    let visual_row_in_line = col_to_visual_row_in_wrapped(&wrapped, logical_col);
    base + visual_row_in_line
}

/// Converts a global visual line index to (logical_row, visual_skip_within_row).
pub(crate) fn visual_line_to_logical(
    lines: &Lines,
    width: usize,
    tab_width: usize,
    visual_index: usize,
) -> (usize, usize) {
    let mut remaining = visual_index;
    for (i, line) in lines.iter_row().enumerate() {
        let count = visual_line_count_for_row(line, width, tab_width);
        if remaining < count {
            return (i, remaining);
        }
        remaining -= count;
    }
    // Past the end — clamp to the last visual line of the last logical line.
    let last_row = lines.len().saturating_sub(1);
    let last_line = lines
        .get(jagged::index::RowIndex::new(last_row))
        .map_or(&[][..], |v| v);
    let count = visual_line_count_for_row(last_line, width, tab_width);
    (last_row, count.saturating_sub(1))
}

/// Returns the total number of visual lines across all logical lines.
pub(crate) fn total_visual_lines(lines: &Lines, width: usize, tab_width: usize) -> usize {
    lines
        .iter_row()
        .map(|line| visual_line_count_for_row(line, width, tab_width))
        .sum::<usize>()
        .max(1)
}

/// Given wrapped sub-lines for a single logical line, determine which visual
/// row a logical column falls on.
pub(crate) fn col_to_visual_row_in_wrapped(wrapped: &[Vec<char>], logical_col: usize) -> usize {
    let mut char_offset = 0;
    for (i, sub_line) in wrapped.iter().enumerate() {
        let sub_len = sub_line.len();
        if logical_col < char_offset + sub_len || i + 1 == wrapped.len() {
            return i;
        }
        char_offset += sub_len;
    }
    0
}

/// Given the wrapped sub-lines for a logical line, return the logical column
/// range `[start, end)` for the given visual row index within that line.
pub(crate) fn visual_row_col_range(wrapped: &[Vec<char>], visual_row: usize) -> (usize, usize) {
    let mut start = 0;
    for (i, sub_line) in wrapped.iter().enumerate() {
        let end = start + sub_line.len();
        if i == visual_row {
            return (start, end);
        }
        start = end;
    }
    (start, start)
}

/// Given a visual line index within a logical line and a preferred display
/// column, compute the logical column that best matches that display column.
/// `wrapped` is the result of `LineWrapper::wrap_line`.
pub(crate) fn visual_pos_to_logical_col(
    wrapped: &[Vec<char>],
    visual_row: usize,
    preferred_col: usize,
    tab_width: usize,
) -> usize {
    let (col_start, col_end) = visual_row_col_range(wrapped, visual_row);

    let sub_line = match wrapped.get(visual_row) {
        Some(l) => l,
        None => return col_start,
    };

    // Walk the sub-line accumulating display width until we reach preferred_col.
    let mut display_w = 0;
    for (i, &ch) in sub_line.iter().enumerate() {
        if display_w >= preferred_col {
            return col_start + i;
        }
        display_w += char_width(ch, tab_width);
    }

    // preferred_col is past the end of this visual line — clamp to last char.
    if col_end > col_start {
        col_end.saturating_sub(1)
    } else {
        col_start
    }
}

/// Computes the display column (width-based) for a given logical column
/// within a visual row of wrapped text.
pub(crate) fn logical_col_to_display_col(
    line: &[char],
    wrapped: &[Vec<char>],
    visual_row: usize,
    logical_col: usize,
    tab_width: usize,
) -> usize {
    let (col_start, _) = visual_row_col_range(wrapped, visual_row);
    let local_col = logical_col.saturating_sub(col_start);

    let sub_line = match wrapped.get(visual_row) {
        Some(l) => l,
        None => return 0,
    };

    let mut display_w = 0;
    for (i, &ch) in sub_line.iter().enumerate() {
        if i >= local_col {
            break;
        }
        display_w += char_width(ch, tab_width);
    }
    let _ = line; // kept for API consistency
    display_w
}

/// Returns the display (visual/screen) column of a cursor at the given
/// logical (row, col) position. Takes wrapping into account so it returns
/// the horizontal offset within the visual line the cursor sits on.
pub(crate) fn cursor_display_col(
    lines: &Lines,
    row: usize,
    col: usize,
    width: usize,
    tab_width: usize,
) -> usize {
    let line = match lines.get(jagged::index::RowIndex::new(row)) {
        Some(l) => l,
        None => return 0,
    };
    let wrapped = LineWrapper::wrap_line(line, width, tab_width);
    let visual_row = col_to_visual_row_in_wrapped(&wrapped, col);
    logical_col_to_display_col(line, &wrapped, visual_row, col, tab_width)
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! update_view_vertical_test {
        ($name:ident: {
        view: $view:expr,
        height: $height:expr,
        cursor: $cursor:expr,
        expected: $expected:expr
    }) => {
            #[test]
            fn $name() {
                // given
                let mut view = $view;

                // when
                let offset = view.update_viewport_vertical($height, $cursor);

                // then
                assert_eq!(offset, $expected);
            }
        };
    }

    macro_rules! update_view_horizontal_test {
        ($name:ident: {
        view: $view:expr,
        width: $width:expr,
        cursor: $cursor:expr,
        expected: $expected:expr
    }) => {
            #[test]
            fn $name() {
                // given
                let mut view = $view;
                let line = vec![];

                // when
                let offset = view.update_viewport_horizontal($width, $cursor, Some(&line));

                // then
                assert_eq!(offset, $expected);
            }
        };
    }

    // cursor above viewport → scroll up
    update_view_vertical_test!(
        scroll_up: {
            view: ViewState{
                viewport: Offset::new(0, 1),
                ..Default::default()
            },
            height:  2,
            cursor: 0,
            expected: 0
        }
    );

    // cursor below viewport → scroll down
    update_view_vertical_test!(
        scroll_down: {
            view: ViewState{
                viewport: Offset::new(0, 0),
                ..Default::default()
            },
            height:  2,
            cursor: 2,
            expected: 1
        }
    );

    // cursor left of viewport → scroll left
    update_view_horizontal_test!(
        scroll_left: {
            view: ViewState{
                viewport: Offset::new(1, 0),
                ..Default::default()
            },
            width: 2,
            cursor: 0,
            expected: 0
        }
    );

    // cursor right of viewport → scroll right
    update_view_horizontal_test!(
        scroll_right: {
            view: ViewState{
                viewport: Offset::new(0, 0),
                ..Default::default()
            },
            width: 2,
            cursor: 2,
            expected: 1
        }
    );

    // ====================================================================
    // Visual line helper tests
    // ====================================================================

    fn make_lines(text: &str) -> Lines {
        Lines::from(text)
    }

    #[test]
    fn test_visual_line_count_for_row_no_wrap() {
        // A short line that fits in 80 columns → 1 visual line.
        let line: Vec<char> = "Hello".chars().collect();
        assert_eq!(visual_line_count_for_row(&line, 80, 4), 1);
    }

    #[test]
    fn test_visual_line_count_for_row_wraps() {
        // 10 chars wrapped at width 4 → 3 visual lines (4, 4, 2).
        let line: Vec<char> = "0123456789".chars().collect();
        assert_eq!(visual_line_count_for_row(&line, 4, 4), 3);
    }

    #[test]
    fn test_visual_line_count_for_row_empty() {
        let line: Vec<char> = vec![];
        assert_eq!(visual_line_count_for_row(&line, 80, 4), 1);
    }

    #[test]
    fn test_logical_row_to_first_visual_line() {
        // Two logical lines: "0123456789" (wraps to 3 at width 4) and "ab".
        let lines = make_lines("0123456789\nab");
        // Line 0 starts at visual 0.
        assert_eq!(logical_row_to_first_visual_line(&lines, 4, 4, 0), 0);
        // Line 1 starts at visual 3 (after 3 visual lines of line 0).
        assert_eq!(logical_row_to_first_visual_line(&lines, 4, 4, 1), 3);
    }

    #[test]
    fn test_cursor_to_visual_line() {
        let lines = make_lines("0123456789\nab");
        // Cursor at (0, 0) → visual line 0.
        assert_eq!(cursor_to_visual_line(&lines, 4, 4, 0, 0), 0);
        // Cursor at (0, 5) → on the second visual line of row 0 → visual 1.
        assert_eq!(cursor_to_visual_line(&lines, 4, 4, 0, 5), 1);
        // Cursor at (0, 9) → on the third visual line of row 0 → visual 2.
        assert_eq!(cursor_to_visual_line(&lines, 4, 4, 0, 9), 2);
        // Cursor at (1, 0) → visual 3.
        assert_eq!(cursor_to_visual_line(&lines, 4, 4, 1, 0), 3);
    }

    #[test]
    fn test_visual_line_to_logical() {
        let lines = make_lines("0123456789\nab");
        assert_eq!(visual_line_to_logical(&lines, 4, 4, 0), (0, 0));
        assert_eq!(visual_line_to_logical(&lines, 4, 4, 1), (0, 1));
        assert_eq!(visual_line_to_logical(&lines, 4, 4, 2), (0, 2));
        assert_eq!(visual_line_to_logical(&lines, 4, 4, 3), (1, 0));
    }

    #[test]
    fn test_visual_line_to_logical_clamped() {
        let lines = make_lines("ab");
        // Visual line 99 is past the end → clamp to last visual line.
        let (row, skip) = visual_line_to_logical(&lines, 80, 4, 99);
        assert_eq!(row, 0);
        assert_eq!(skip, 0);
    }

    #[test]
    fn test_total_visual_lines() {
        let lines = make_lines("0123456789\nab");
        // Line 0 → 3 visual lines at width 4; Line 1 → 1; Total = 4.
        assert_eq!(total_visual_lines(&lines, 4, 4), 4);
    }

    #[test]
    fn test_col_to_visual_row_in_wrapped() {
        let line: Vec<char> = "0123456789".chars().collect();
        let wrapped = LineWrapper::wrap_line(&line, 4, 4);
        // wrapped: ["0123", "4567", "89"]
        assert_eq!(col_to_visual_row_in_wrapped(&wrapped, 0), 0);
        assert_eq!(col_to_visual_row_in_wrapped(&wrapped, 3), 0);
        assert_eq!(col_to_visual_row_in_wrapped(&wrapped, 4), 1);
        assert_eq!(col_to_visual_row_in_wrapped(&wrapped, 8), 2);
    }

    #[test]
    fn test_visual_row_col_range() {
        let line: Vec<char> = "0123456789".chars().collect();
        let wrapped = LineWrapper::wrap_line(&line, 4, 4);
        assert_eq!(visual_row_col_range(&wrapped, 0), (0, 4));
        assert_eq!(visual_row_col_range(&wrapped, 1), (4, 8));
        assert_eq!(visual_row_col_range(&wrapped, 2), (8, 10));
    }

    #[test]
    fn test_visual_pos_to_logical_col() {
        let line: Vec<char> = "0123456789".chars().collect();
        let wrapped = LineWrapper::wrap_line(&line, 4, 4);
        // At visual row 0, display col 2 → logical col 2.
        assert_eq!(visual_pos_to_logical_col(&wrapped, 0, 2, 4), 2);
        // At visual row 1, display col 0 → logical col 4.
        assert_eq!(visual_pos_to_logical_col(&wrapped, 1, 0, 4), 4);
        // At visual row 1, display col 2 → logical col 6.
        assert_eq!(visual_pos_to_logical_col(&wrapped, 1, 2, 4), 6);
        // At visual row 2, display col 99 (past end) → clamp to last char (9).
        assert_eq!(visual_pos_to_logical_col(&wrapped, 2, 99, 4), 9);
    }

    #[test]
    fn test_ensure_cursor_visible_wrap_scroll_down() {
        // Scenario: viewport at (0,0), height 3, cursor moves to logical row 1
        // which starts at visual line 3 (past the viewport).
        let lines = make_lines("0123456789\nab");
        let mut view = ViewState {
            tab_width: 4,
            ..Default::default()
        };
        view.viewport.y = 0;
        view.viewport_visual_skip = 0;

        // Cursor at (1, 0) → visual line 3 (past height 3: [0,1,2]).
        view.ensure_cursor_visible_wrap(&lines, 4, 3, 1, 0);
        // Viewport should scroll so visual line 3 is at the bottom (visual start = 1).
        assert_eq!(view.viewport.y, 0);
        assert_eq!(view.viewport_visual_skip, 1);
    }

    #[test]
    fn test_ensure_cursor_visible_wrap_scroll_up() {
        let lines = make_lines("0123456789\nab");
        let mut view = ViewState {
            tab_width: 4,
            ..Default::default()
        };
        // Viewport starts at visual line 2 (row 0, skip 2).
        view.viewport.y = 0;
        view.viewport_visual_skip = 2;

        // Cursor at (0, 0) → visual line 0, which is before viewport visual start (2).
        view.ensure_cursor_visible_wrap(&lines, 4, 3, 0, 0);
        // Viewport should scroll up to visual line 0.
        assert_eq!(view.viewport.y, 0);
        assert_eq!(view.viewport_visual_skip, 0);
    }

    #[test]
    fn test_ensure_cursor_visible_wrap_scroll_at_full_visual_line_edge() {
        // "0123456789ab" at width 4 → 3 visual lines: ["0123", "4567", "89ab"].
        // The last segment "89ab" is full (4 chars = width).
        // Viewport height 3, shows visual lines 0,1,2. viewport_visual_end = 2.
        // Cursor at (0, 12) = line.len(). cursor_visual maps to 2 (last segment fallback).
        // cursor_display_col = 4 = width → at right edge → treat as visual 3 → scroll.
        let lines = make_lines("0123456789ab");
        let mut view = ViewState {
            tab_width: 4,
            ..Default::default()
        };
        view.viewport.y = 0;
        view.viewport_visual_skip = 0;

        view.ensure_cursor_visible_wrap(&lines, 4, 3, 0, 12);
        // Viewport should scroll: new start = visual 1 → (row 0, skip 1).
        assert_eq!(view.viewport.y, 0);
        assert_eq!(view.viewport_visual_skip, 1);
    }

    #[test]
    fn test_ensure_cursor_visible_wrap_no_scroll_at_partial_visual_line() {
        // "0123456789" at width 4 → 3 visual lines: ["0123", "4567", "89"].
        // The last segment "89" has only 2 chars — NOT full.
        // Cursor at (0, 10) = line.len(). cursor_display_col = 2 < 4 → no scroll.
        let lines = make_lines("0123456789");
        let mut view = ViewState {
            tab_width: 4,
            ..Default::default()
        };
        view.viewport.y = 0;
        view.viewport_visual_skip = 0;

        view.ensure_cursor_visible_wrap(&lines, 4, 3, 0, 10);
        // Viewport unchanged.
        assert_eq!(view.viewport.y, 0);
        assert_eq!(view.viewport_visual_skip, 0);
    }

    #[test]
    fn test_ensure_cursor_visible_wrap_no_scroll_cursor_mid_line() {
        // Cursor in the middle of the last visible line — should never scroll.
        let lines = make_lines("0123456789ab");
        let mut view = ViewState {
            tab_width: 4,
            ..Default::default()
        };
        view.viewport.y = 0;
        view.viewport_visual_skip = 0;

        // Cursor at (0, 10) → visual line 2, display col 2 < width 4. No scroll.
        view.ensure_cursor_visible_wrap(&lines, 4, 3, 0, 10);
        assert_eq!(view.viewport.y, 0);
        assert_eq!(view.viewport_visual_skip, 0);
    }
}
