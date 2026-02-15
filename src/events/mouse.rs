use crossterm::event::{MouseEvent as CTMouseEvent, MouseEventKind};
use jagged::Index2;

use crate::{
    actions::{Execute, SwitchMode},
    helper::char_width,
    state::selection::set_selection,
    state::view::{
        logical_row_to_first_visual_line, total_visual_lines, visual_line_to_logical,
        visual_pos_to_logical_col,
    },
    view::line_wrapper::LineWrapper,
    EditorMode, EditorState,
};

/// The number of lines to scroll per scroll wheel event.
const SCROLL_LINES: usize = 1;

/// Handles a mouse event.
#[derive(Clone, Debug, Default)]
pub struct MouseEventHandler {}

impl MouseEventHandler {
    pub fn on_event<E>(event: E, state: &mut EditorState)
    where
        E: Into<MouseEvent>,
    {
        let event = event.into();
        if event == MouseEvent::None {
            return;
        }

        // Handle scroll events
        match event {
            MouseEvent::ScrollUp(mouse) => {
                if Self::is_position_within_bounds(&mouse, state) {
                    Self::handle_scroll_up(state);
                }
                return;
            }
            MouseEvent::ScrollDown(mouse) => {
                if Self::is_position_within_bounds(&mouse, state) {
                    Self::handle_scroll_down(state);
                }
                return;
            }
            _ => {}
        }

        // Check if the mouse event is within the editor's screen area
        if !Self::is_within_bounds(&event, state) {
            return;
        }

        if let MouseEvent::Down(_) = event {
            state.selection = None;
            if state.mode == EditorMode::Visual {
                SwitchMode(EditorMode::Normal).execute(state);
            }
        }

        if let MouseEvent::Drag(_) = event {
            if state.mode != EditorMode::Visual {
                SwitchMode(EditorMode::Visual).execute(state);
            }
            set_selection(&mut state.selection, state.cursor);
        }

        match event {
            MouseEvent::Down(mouse) | MouseEvent::Up(mouse) | MouseEvent::Drag(mouse) => {
                let lines = &state.lines;
                let cursor = mouse_position_to_cursor_position(state, &mouse, state.view.tab_width);
                let last_row = lines.last_row_index();
                let last_col = lines.last_col_index(cursor.row);

                // row is out of bounds
                if last_row < cursor.row {
                    let last_col = lines.last_col_index(last_row);
                    state.cursor = Index2::new(last_row, last_col);
                // col is out of bounds
                } else if last_col < cursor.col {
                    state.cursor = Index2::new(cursor.row, last_col);
                } else {
                    state.cursor = cursor;
                }

                // Update desired display column so subsequent up/down
                // movements aim at the clicked visual column.
                state.update_desired_display_col();

                if let MouseEvent::Drag(_) = event {
                    set_selection(&mut state.selection, state.cursor);
                }
            }
            MouseEvent::ScrollUp(_) | MouseEvent::ScrollDown(_) | MouseEvent::None => (),
        };
    }

    fn handle_scroll_up(state: &mut EditorState) {
        if state.view.wrap {
            Self::scroll_visual_lines(state, -(SCROLL_LINES as isize));
        } else {
            state.view.viewport.y = state.view.viewport.y.saturating_sub(SCROLL_LINES);
        }
        Self::clamp_cursor_to_viewport(state);
    }

    fn handle_scroll_down(state: &mut EditorState) {
        if state.view.wrap {
            Self::scroll_visual_lines(state, SCROLL_LINES as isize);
        } else {
            let last_visible_row = state.view.viewport.y + state.view.num_rows.saturating_sub(1);
            if last_visible_row >= state.lines.last_row_index() {
                return;
            }
            let max_viewport_y = state.lines.len().saturating_sub(1);
            state.view.viewport.y = (state.view.viewport.y + SCROLL_LINES).min(max_viewport_y);
        }
        Self::clamp_cursor_to_viewport(state);
    }

    /// Scrolls the viewport by `delta` visual lines (positive = down, negative = up).
    fn scroll_visual_lines(state: &mut EditorState, delta: isize) {
        let width = state.view.content_width();
        let tab_width = state.view.tab_width;

        let current_visual = logical_row_to_first_visual_line(
            &state.lines,
            width,
            tab_width,
            state.view.viewport.y,
        ) + state.view.viewport_visual_skip;

        let total = total_visual_lines(&state.lines, width, tab_width);
        let height = state.view.content_height();

        let new_visual = if delta < 0 {
            current_visual.saturating_sub(delta.unsigned_abs())
        } else {
            // Don't scroll past the point where the last visual line is at the bottom.
            let max_start = total.saturating_sub(height);
            (current_visual + delta as usize).min(max_start)
        };

        let (row, skip) = visual_line_to_logical(&state.lines, width, tab_width, new_visual);
        state.view.viewport.y = row;
        state.view.viewport_visual_skip = skip;
    }

    fn clamp_cursor_to_viewport(state: &mut EditorState) {
        if state.view.wrap {
            Self::clamp_cursor_to_viewport_wrap(state);
        } else {
            Self::clamp_cursor_to_viewport_nowrap(state);
        }
    }

    fn clamp_cursor_to_viewport_nowrap(state: &mut EditorState) {
        let viewport_y = state.view.viewport.y;
        let viewport_height = state.view.num_rows;

        if viewport_height == 0 {
            return;
        }

        let viewport_bottom = viewport_y + viewport_height.saturating_sub(1);

        if state.cursor.row < viewport_y {
            state.cursor.row = viewport_y;
            state.clamp_column();
        } else if state.cursor.row > viewport_bottom {
            state.cursor.row = viewport_bottom.min(state.lines.last_row_index());
            state.clamp_column();
        }
    }

    fn clamp_cursor_to_viewport_wrap(state: &mut EditorState) {
        let width = state.view.content_width();
        let tab_width = state.view.tab_width;
        let height = state.view.num_rows;

        if height == 0 || width == 0 {
            return;
        }

        let viewport_visual_start = logical_row_to_first_visual_line(
            &state.lines,
            width,
            tab_width,
            state.view.viewport.y,
        ) + state.view.viewport_visual_skip;

        let viewport_visual_end = viewport_visual_start + height.saturating_sub(1);

        let cursor_visual = crate::state::view::cursor_to_visual_line(
            &state.lines,
            width,
            tab_width,
            state.cursor.row,
            state.cursor.col,
        );

        if cursor_visual < viewport_visual_start {
            // Move cursor to the first visible visual line.
            let (row, skip) =
                visual_line_to_logical(&state.lines, width, tab_width, viewport_visual_start);
            state.cursor.row = row;
            let line = state.lines.get(jagged::index::RowIndex::new(row));
            if let Some(line) = line {
                let wrapped = LineWrapper::wrap_line(line, width, tab_width);
                state.cursor.col = visual_pos_to_logical_col(&wrapped, skip, 0, tab_width);
            } else {
                state.cursor.col = 0;
            }
            state.clamp_column();
        } else if cursor_visual > viewport_visual_end {
            // Move cursor to the last visible visual line.
            let (row, skip) =
                visual_line_to_logical(&state.lines, width, tab_width, viewport_visual_end);
            state.cursor.row = row.min(state.lines.last_row_index());
            let line = state.lines.get(jagged::index::RowIndex::new(state.cursor.row));
            if let Some(line) = line {
                let wrapped = LineWrapper::wrap_line(line, width, tab_width);
                state.cursor.col = visual_pos_to_logical_col(&wrapped, skip, 0, tab_width);
            } else {
                state.cursor.col = 0;
            }
            state.clamp_column();
        }
    }

    /// Checks if the mouse event occurred within the editor's screen area.
    fn is_within_bounds(event: &MouseEvent, state: &EditorState) -> bool {
        let mouse = match event {
            MouseEvent::Down(pos) | MouseEvent::Up(pos) | MouseEvent::Drag(pos) => pos,
            MouseEvent::ScrollUp(pos) | MouseEvent::ScrollDown(pos) => pos,
            MouseEvent::None => return false,
        };

        Self::is_position_within_bounds(mouse, state)
    }

    fn is_position_within_bounds(mouse: &MousePosition, state: &EditorState) -> bool {
        let area = &state.view.screen_area;
        let x: usize = area.x.into();
        let y: usize = area.y.into();
        let width: usize = area.width.into();
        let height: usize = area.height.into();

        mouse.col >= x && mouse.col < x + width && mouse.row >= y && mouse.row < y + height
    }
}

fn mouse_position_to_cursor_position(
    state: &EditorState,
    mouse: &MousePosition,
    tab_width: usize,
) -> Index2 {
    let mut row_index = state.view.viewport.y;
    let mut col_index = state.view.viewport.x;

    // Global -> editor coordinates
    let mouse = Index2::new(
        mouse.row.saturating_sub(state.view.screen_area.y.into()),
        mouse.col.saturating_sub(state.view.screen_area.x.into()),
    );

    if !state.view.wrap {
        return Index2::new(
            mouse.row.saturating_add(row_index),
            mouse.col.saturating_add(col_index),
        );
    }

    let visual_skip = state.view.viewport_visual_skip;
    let mut row_screen_index = 0;
    let mut is_first_line = true;

    for line in state.lines.iter_row().skip(row_index) {
        let wrapped_line = LineWrapper::wrap_line(
            line,
            state.view.screen_area.width.into(),
            state.view.tab_width,
        );
        let full_len = wrapped_line.len().max(1);

        // For the first logical line, we may have skipped some visual lines.
        let visible_start = if is_first_line { visual_skip } else { 0 };
        let visible_len = full_len.saturating_sub(visible_start);

        if row_screen_index + visible_len > mouse.row {
            // The mouse is on this logical line. Compute the visual row within
            // the full wrapped representation.
            let visual_row_in_visible = mouse.row.saturating_sub(row_screen_index);
            let actual_visual_row = visible_start + visual_row_in_visible;
            let adjusted_mouse = Index2::new(actual_visual_row, mouse.col);
            col_index = find_cursor_column_in_wrapped_line(&wrapped_line, &adjusted_mouse, tab_width);
            break;
        }
        row_screen_index += visible_len;
        row_index += 1;
        is_first_line = false;
    }

    Index2::new(row_index, col_index)
}

fn find_cursor_column_in_wrapped_line(
    line: &[Vec<char>],
    mouse: &Index2,
    tab_width: usize,
) -> usize {
    let Some(l) = line.get(mouse.row) else {
        return 0;
    };

    let col_offset: usize = line.iter().take(mouse.row).map(Vec::len).sum();
    let mut current_width = 0;
    let mut col_index = 0;

    for &ch in l {
        let char_width = char_width(ch, tab_width);

        if current_width + char_width > mouse.col {
            break;
        }

        current_width += char_width;
        col_index += 1;
    }

    col_offset + col_index
}

/// Represents a mouse event.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum MouseEvent {
    /// A mouse press event.
    Down(MousePosition),

    /// A mouse release event.
    Up(MousePosition),

    /// A mouse Drag event.
    Drag(MousePosition),

    /// A scroll up (wheel up) event.
    ScrollUp(MousePosition),

    /// A scroll down (wheel down) event.
    ScrollDown(MousePosition),

    /// A mouse event that is not handled by the editor.
    None,
}

impl From<CTMouseEvent> for MouseEvent {
    fn from(event: CTMouseEvent) -> Self {
        match event.kind {
            MouseEventKind::Down(_) => Self::Down(MousePosition::new(event.row, event.column)),
            MouseEventKind::Up(_) => Self::Up(MousePosition::new(event.row, event.column)),
            MouseEventKind::Drag(_) => Self::Drag(MousePosition::new(event.row, event.column)),
            MouseEventKind::ScrollUp => Self::ScrollUp(MousePosition::new(event.row, event.column)),
            MouseEventKind::ScrollDown => {
                Self::ScrollDown(MousePosition::new(event.row, event.column))
            }
            _ => Self::None,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct MousePosition {
    /// The row that the event occurred on.
    pub(crate) row: usize,
    /// The column that the event occurred on.
    pub(crate) col: usize,
}

impl MousePosition {
    /// Creates a new `MousePosition` instance.
    fn new(row: u16, col: u16) -> Self {
        Self {
            row: row.into(),
            col: col.into(),
        }
    }
}
