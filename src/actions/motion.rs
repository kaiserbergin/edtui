use std::cmp::min;

use crate::{
    helper::{find_matching_bracket, skip_empty_lines, skip_whitespace_for_selection},
    state::selection::set_selection_with_lines,
    state::view::{
        col_to_visual_row_in_wrapped, visual_pos_to_logical_col,
    },
    view::line_wrapper::LineWrapper,
};
use jagged::Index2;

use super::Execute;
use crate::{
    helper::{max_col, max_col_normal, skip_whitespace, skip_whitespace_rev},
    EditorMode, EditorState,
};

#[derive(Clone, Debug, Copy)]
pub struct MoveForward(pub usize);

impl Execute for MoveForward {
    fn execute(&mut self, state: &mut EditorState) {
        let wrap = state.view.wrap && state.view.screen_area.width > 0;
        for _ in 0..self.0 {
            if state.cursor.col >= max_col(&state.lines, &state.cursor, state.mode) {
                // In wrap mode, cross to the start of the next logical line.
                if wrap && state.cursor.row < state.lines.len().saturating_sub(1) {
                    state.cursor.row += 1;
                    state.cursor.col = 0;
                } else {
                    break;
                }
            } else {
                state.cursor.col += 1;
            }
        }
        state.update_desired_display_col();
        if state.mode == EditorMode::Visual {
            set_selection_with_lines(&mut state.selection, state.cursor, &state.lines);
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct MoveBackward(pub usize);

impl Execute for MoveBackward {
    fn execute(&mut self, state: &mut EditorState) {
        let wrap = state.view.wrap && state.view.screen_area.width > 0;
        for _ in 0..self.0 {
            if state.cursor.col == 0 {
                // In wrap mode, cross to the end of the previous logical line.
                if wrap && state.cursor.row > 0 {
                    state.cursor.row -= 1;
                    state.cursor.col = max_col(&state.lines, &state.cursor, state.mode);
                } else {
                    break;
                }
            } else {
                let mc = max_col(&state.lines, &state.cursor, state.mode);
                if state.cursor.col > mc {
                    state.cursor.col = mc;
                }
                state.cursor.col = state.cursor.col.saturating_sub(1);
            }
        }
        state.update_desired_display_col();
        if state.mode == EditorMode::Visual {
            set_selection_with_lines(&mut state.selection, state.cursor, &state.lines);
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct MoveUp(pub usize);

impl Execute for MoveUp {
    fn execute(&mut self, state: &mut EditorState) {
        let wrap = state.view.wrap;
        let width = state.view.screen_area.width as usize;
        let tab_width = state.view.tab_width;

        for _ in 0..self.0 {
            if wrap && width > 0 {
                move_up_visual(state, width, tab_width);
            } else {
                if state.cursor.row == 0 {
                    break;
                }
                state.cursor.row = state.cursor.row.saturating_sub(1);
                place_cursor_at_desired_display_col(state);
            }
        }
        if state.mode == EditorMode::Visual {
            set_selection_with_lines(&mut state.selection, state.cursor, &state.lines);
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct MoveDown(pub usize);

impl Execute for MoveDown {
    fn execute(&mut self, state: &mut EditorState) {
        let wrap = state.view.wrap;
        let width = state.view.screen_area.width as usize;
        let tab_width = state.view.tab_width;

        for _ in 0..self.0 {
            if wrap && width > 0 {
                move_down_visual(state, width, tab_width);
            } else {
                if state.cursor.row >= state.lines.len().saturating_sub(1) {
                    break;
                }
                state.cursor.row += 1;
                place_cursor_at_desired_display_col(state);
            }
        }
        if state.mode == EditorMode::Visual {
            set_selection_with_lines(&mut state.selection, state.cursor, &state.lines);
        }
    }
}

/// Places the cursor column on the current row at the logical column
/// corresponding to `state.desired_display_col`, clamping to the line's
/// max column. Works for both wrap and no-wrap: wraps the line to find
/// the first visual row containing the current column and resolves the
/// desired display column within it. For no-wrap the line has a single
/// visual row so the display column maps directly.
fn place_cursor_at_desired_display_col(state: &mut EditorState) {
    let row = state.cursor.row;
    let width = state.view.screen_area.width as usize;
    let tab_width = state.view.tab_width;
    let desired = state.desired_display_col;

    let line = match state.lines.get(jagged::index::RowIndex::new(row)) {
        Some(l) => l,
        None => {
            state.cursor.col = 0;
            return;
        }
    };

    // When wrap is off, treat the whole line as a single visual row.
    let effective_width = if state.view.wrap && width > 0 { width } else { usize::MAX };
    let wrapped = LineWrapper::wrap_line(line, effective_width, tab_width);
    // Use visual row 0 (for no-wrap this is the only row; for wrap after a
    // row change without visual context we start at the first visual row).
    let new_col = visual_pos_to_logical_col(&wrapped, 0, desired, tab_width);
    let max = max_col(&state.lines, &Index2::new(row, 0), state.mode);
    state.cursor.col = new_col.min(max);
}

/// Move cursor up by one visual (screen) line when wrap is on.
/// Uses `state.desired_display_col` so the cursor keeps the same visual
/// horizontal position across lines.
fn move_up_visual(state: &mut EditorState, width: usize, tab_width: usize) {
    let row = state.cursor.row;
    let col = state.cursor.col;
    let desired = state.desired_display_col;

    let line = match state.lines.get(jagged::index::RowIndex::new(row)) {
        Some(l) => l,
        None => return,
    };

    let wrapped = LineWrapper::wrap_line(line, width, tab_width);
    let visual_row = col_to_visual_row_in_wrapped(&wrapped, col);

    if visual_row > 0 {
        // Move to the previous visual line within the same logical line.
        let new_col = visual_pos_to_logical_col(&wrapped, visual_row - 1, desired, tab_width);
        let max = max_col(&state.lines, &Index2::new(row, 0), state.mode);
        state.cursor.col = new_col.min(max);
    } else {
        // Move to the last visual line of the previous logical line.
        if row == 0 {
            return;
        }
        let prev_row = row - 1;
        let prev_line = match state.lines.get(jagged::index::RowIndex::new(prev_row)) {
            Some(l) => l,
            None => return,
        };
        let prev_wrapped = LineWrapper::wrap_line(prev_line, width, tab_width);
        let last_visual = prev_wrapped.len().saturating_sub(1);
        let new_col = visual_pos_to_logical_col(&prev_wrapped, last_visual, desired, tab_width);
        let max = max_col(&state.lines, &Index2::new(prev_row, 0), state.mode);
        state.cursor.row = prev_row;
        state.cursor.col = new_col.min(max);
    }
}

/// Move cursor down by one visual (screen) line when wrap is on.
/// Uses `state.desired_display_col` so the cursor keeps the same visual
/// horizontal position across lines.
fn move_down_visual(state: &mut EditorState, width: usize, tab_width: usize) {
    let row = state.cursor.row;
    let col = state.cursor.col;
    let desired = state.desired_display_col;

    let line = match state.lines.get(jagged::index::RowIndex::new(row)) {
        Some(l) => l,
        None => return,
    };

    let wrapped = LineWrapper::wrap_line(line, width, tab_width);
    let visual_row = col_to_visual_row_in_wrapped(&wrapped, col);
    let total_visual = wrapped.len().max(1);

    if visual_row + 1 < total_visual {
        // Move to the next visual line within the same logical line.
        let new_col = visual_pos_to_logical_col(&wrapped, visual_row + 1, desired, tab_width);
        let max = max_col(&state.lines, &Index2::new(row, 0), state.mode);
        state.cursor.col = new_col.min(max);
    } else {
        // Move to the first visual line of the next logical line.
        if row >= state.lines.len().saturating_sub(1) {
            return;
        }
        let next_row = row + 1;
        let next_line = match state.lines.get(jagged::index::RowIndex::new(next_row)) {
            Some(l) => l,
            None => return,
        };
        let next_wrapped = LineWrapper::wrap_line(next_line, width, tab_width);
        let new_col = visual_pos_to_logical_col(&next_wrapped, 0, desired, tab_width);
        let max = max_col(&state.lines, &Index2::new(next_row, 0), state.mode);
        state.cursor.row = next_row;
        state.cursor.col = new_col.min(max);
    }
}

/// Move one word forward. Breaks on the first character that is not of
/// the same class as the initial character or breaks on line ending.
/// Furthermore, after the first break, whitespaces are skipped.
#[derive(Clone, Debug, Copy)]
pub struct MoveWordForward(pub usize);

impl Execute for MoveWordForward {
    fn execute(&mut self, state: &mut EditorState) {
        if state.lines.is_empty() {
            return;
        }

        state.clamp_column();

        for _ in 0..self.0 {
            move_word_forward(state);
        }

        state.update_desired_display_col();
        if state.mode == EditorMode::Visual {
            set_selection_with_lines(&mut state.selection, state.cursor, &state.lines);
        }
    }
}

fn move_word_forward(state: &mut EditorState) {
    let mut start_char_class = CharacterClass::from(state.lines.get(state.cursor));

    if state.mode == EditorMode::Visual {
        let next_char_class = CharacterClass::from(state.lines.get(Index2::new(
            state.cursor.row,
            state.cursor.col.saturating_add(1),
        )));
        if next_char_class != start_char_class {
            // state.cursor = Index2::new(state.cursor.row, state.cursor.col.saturating_add(1));
            start_char_class = next_char_class;
        }
    }

    let start_index = match (
        state.lines.is_last_col(state.cursor),
        state.lines.is_last_row(state.cursor),
    ) {
        (true, true) => return,
        (true, false) => {
            state.cursor = Index2::new(state.cursor.row.saturating_add(1), 0);
            return;
        }
        _ => Index2::new(state.cursor.row, state.cursor.col.saturating_add(1)),
    };

    for (next_char, index) in state.lines.iter().from(start_index) {
        if CharacterClass::from(next_char) != start_char_class {
            if state.mode == EditorMode::Visual {
                if index.col == 0 && index.row > state.cursor.row {
                    // Different class started on next line → stay at end of current line
                    state.cursor = Index2::new(
                        state.cursor.row,
                        state.lines.last_col_index(state.cursor.row),
                    );
                } else {
                    state.cursor = Index2::new(index.row, index.col.saturating_sub(1));
                    skip_whitespace_for_selection(&state.lines, &mut state.cursor);
                }
            } else {
                state.cursor = index;
                skip_whitespace(&state.lines, &mut state.cursor);
            }
            return;
        }
    }
    state.cursor = Index2::new(
        state.cursor.row,
        state.lines.last_col_index(state.cursor.row),
    );
}

/// Move one word forward to the end of the word.
#[derive(Clone, Debug, Copy)]
pub struct MoveWordForwardToEndOfWord(pub usize);
impl Execute for MoveWordForwardToEndOfWord {
    fn execute(&mut self, state: &mut EditorState) {
        if state.lines.is_empty() {
            return;
        }

        state.clamp_column();

        for _ in 0..self.0 {
            move_word_forward_to_end_of_word(state);
        }

        state.update_desired_display_col();
        if state.mode == EditorMode::Visual {
            set_selection_with_lines(&mut state.selection, state.cursor, &state.lines);
        }
    }
}

fn move_word_forward_to_end_of_word(state: &mut EditorState) {
    let mut start_index = match (
        state.lines.is_last_col(state.cursor),
        state.lines.is_last_row(state.cursor),
    ) {
        (true, true) => return,
        (true, false) => Index2::new(state.cursor.row.saturating_add(1), 0),
        _ => Index2::new(state.cursor.row, state.cursor.col.saturating_add(1)),
    };
    skip_empty_lines(&state.lines, &mut start_index.row);
    skip_whitespace(&state.lines, &mut start_index);
    let start_char_class = CharacterClass::from(state.lines.get(start_index));

    for (next_char, index) in state.lines.iter().from(start_index) {
        // Break loop if characters don't belong to the same class
        if CharacterClass::from(next_char) != start_char_class {
            break;
        }
        state.cursor = index;

        // Break loop if it reaches the end of the line
        if state.lines.is_last_col(index) {
            break;
        }
    }
}

/// Move one word forward. Breaks on the first character that is not of
/// the same class as the initial character or breaks on line starts.
/// Skips whitespaces if necessary.
#[derive(Clone, Debug, Copy)]
pub struct MoveWordBackward(pub usize);

impl Execute for MoveWordBackward {
    fn execute(&mut self, state: &mut EditorState) {
        if state.lines.is_empty() {
            return;
        }

        let max_col = max_col(&state.lines, &state.cursor, state.mode);
        if state.cursor.col > max_col {
            state.cursor.col = max_col;
        }

        for _ in 0..self.0 {
            move_word_backward(state);
        }

        state.update_desired_display_col();
        if state.mode == EditorMode::Visual {
            set_selection_with_lines(&mut state.selection, state.cursor, &state.lines);
        }
    }
}

fn move_word_backward(state: &mut EditorState) {
    let mut start_index = state.cursor;
    if start_index.row == 0 && start_index.col == 0 {
        return;
    }

    if start_index.col == 0 {
        state.cursor.row = start_index.row.saturating_sub(1);
        state.cursor.col = state.lines.last_col_index(state.cursor.row);
        return;
    }

    start_index.col = start_index.col.saturating_sub(1);
    skip_whitespace_rev(&state.lines, &mut start_index);
    let start_char_class = CharacterClass::from(state.lines.get(start_index));

    for (next_char, i) in state.lines.iter().from(start_index).rev() {
        // Break loop if it reaches the start of the line
        if i.col == 0 {
            start_index = i;
            break;
        }
        // Break loop if characters don't belong to the same class
        if CharacterClass::from(next_char) != start_char_class {
            break;
        }
        start_index = i;
    }

    state.cursor = start_index;
}

// Move the cursor to the start of the line.
#[derive(Clone, Debug, Copy)]
pub struct MoveToStartOfLine();

impl Execute for MoveToStartOfLine {
    fn execute(&mut self, state: &mut EditorState) {
        state.cursor.col = 0;
        state.update_desired_display_col();

        if state.mode == EditorMode::Visual {
            set_selection_with_lines(&mut state.selection, state.cursor, &state.lines);
        }
    }
}
// move to the first non-whitespace character in the line.
#[derive(Clone, Debug, Copy)]
pub struct MoveToFirst();

impl Execute for MoveToFirst {
    fn execute(&mut self, state: &mut EditorState) {
        state.cursor.col = 0;
        skip_whitespace(&state.lines, &mut state.cursor);
        state.update_desired_display_col();

        if state.mode == EditorMode::Visual {
            set_selection_with_lines(&mut state.selection, state.cursor, &state.lines);
        }
    }
}

// Move the cursor to the end of the line.
#[derive(Clone, Debug, Copy)]
pub struct MoveToEndOfLine();

impl Execute for MoveToEndOfLine {
    fn execute(&mut self, state: &mut EditorState) {
        state.cursor.col = max_col(&state.lines, &state.cursor, state.mode);
        state.update_desired_display_col();

        if state.mode == EditorMode::Visual {
            set_selection_with_lines(&mut state.selection, state.cursor, &state.lines);
        }
    }
}

// Move the cursor to the start of the buffer.
#[derive(Clone, Debug, Copy)]
pub struct MoveToFirstRow();

impl Execute for MoveToFirstRow {
    fn execute(&mut self, state: &mut EditorState) {
        state.cursor.row = 0;
        state.clamp_column();
        state.update_desired_display_col();

        if state.mode == EditorMode::Visual {
            set_selection_with_lines(&mut state.selection, state.cursor, &state.lines);
        }
    }
}

// Move the cursor to the end of the buffer.
#[derive(Clone, Debug, Copy)]
pub struct MoveToLastRow();

impl Execute for MoveToLastRow {
    fn execute(&mut self, state: &mut EditorState) {
        state.cursor.row = state.lines.len().saturating_sub(1);
        state.clamp_column();
        state.update_desired_display_col();

        if state.mode == EditorMode::Visual {
            set_selection_with_lines(&mut state.selection, state.cursor, &state.lines);
        }
    }
}

// Move the cursor to the closing bracket.
#[derive(Clone, Debug, Copy)]
pub struct MoveToMatchinBracket();

impl Execute for MoveToMatchinBracket {
    fn execute(&mut self, state: &mut EditorState) {
        let max_col = max_col_normal(&state.lines, &state.cursor);
        let index = Index2::new(state.cursor.row, state.cursor.col.min(max_col));
        if let Some(index) = find_matching_bracket(&state.lines, index) {
            state.cursor = index;
            if state.mode == EditorMode::Visual {
                set_selection_with_lines(&mut state.selection, state.cursor, &state.lines);
            }
        };
    }
}

#[derive(Clone, Debug, Copy)]
pub struct MoveHalfPageDown();

impl Execute for MoveHalfPageDown {
    fn execute(&mut self, state: &mut EditorState) {
        if state.view.wrap {
            let jump_visual = state.view.num_rows / 2;
            for _ in 0..jump_visual {
                let width = state.view.screen_area.width as usize;
                let tab_width = state.view.tab_width;
                if width == 0 {
                    break;
                }
                move_down_visual(state, width, tab_width);
            }
        } else {
            let jump_rows = state.view.num_rows / 2;
            state.cursor.row = min(state.cursor.row + jump_rows, state.lines.last_row_index());
            place_cursor_at_desired_display_col(state);
        }

        if state.mode == EditorMode::Visual {
            set_selection_with_lines(&mut state.selection, state.cursor, &state.lines);
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct MoveHalfPageUp();

impl Execute for MoveHalfPageUp {
    fn execute(&mut self, state: &mut EditorState) {
        if state.view.wrap {
            let jump_visual = state.view.num_rows / 2;
            for _ in 0..jump_visual {
                let width = state.view.screen_area.width as usize;
                let tab_width = state.view.tab_width;
                if width == 0 {
                    break;
                }
                move_up_visual(state, width, tab_width);
            }
        } else {
            let jump_rows = state.view.num_rows / 2;
            state.cursor.row = state.cursor.row.saturating_sub(jump_rows);
            place_cursor_at_desired_display_col(state);
        }

        if state.mode == EditorMode::Visual {
            set_selection_with_lines(&mut state.selection, state.cursor, &state.lines);
        }
    }
}

#[derive(Debug, Clone, Eq)]
pub(crate) enum CharacterClass {
    Unknown,
    Alphanumeric,
    Punctuation,
    Whitespace,
}

impl From<&char> for CharacterClass {
    fn from(value: &char) -> Self {
        if value.is_ascii_alphanumeric() {
            return Self::Alphanumeric;
        }
        if value.is_ascii_punctuation() {
            return Self::Punctuation;
        }
        if value.is_ascii_whitespace() {
            return Self::Whitespace;
        }
        Self::Unknown
    }
}

impl From<Option<&char>> for CharacterClass {
    fn from(value: Option<&char>) -> Self {
        value.map_or(CharacterClass::Unknown, Self::from)
    }
}

impl PartialEq for CharacterClass {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (CharacterClass::Unknown, _) | (_, CharacterClass::Unknown) => false,
            _ => std::mem::discriminant(self) == std::mem::discriminant(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Index2, Lines};

    use super::*;
    fn test_state() -> EditorState {
        EditorState::new(Lines::from("Hello World!\n\n123."))
    }

    fn test_state_advanced() -> EditorState {
        EditorState::new(Lines::from("He!lo World!\n\n123. .\n. d"))
    }

    #[test]
    fn test_move_word_forward_visual_mode() {
        let mut state = test_state_advanced();
        state.mode = EditorMode::Visual;

        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 1));

        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 2));

        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 5));

        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 10));

        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 11));

        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(1, 0));

        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(2, 0));

        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(2, 2));

        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(2, 4));
        
        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(3, 1));
    }

    #[test]
    fn test_move_forward() {
        let mut state = test_state();

        MoveForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 1));

        MoveForward(10).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 11));

        MoveForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 11));
    }

    #[test]
    fn test_move_backward() {
        let mut state = test_state();
        state.cursor = Index2::new(0, 11);

        MoveBackward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 10));

        MoveBackward(10).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 0));

        MoveBackward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 0));
    }

    #[test]
    fn test_move_down() {
        // "Hello World!\n\n123."
        // Line 0: "Hello World!" (12 chars, max col 11)
        // Line 1: "" (empty, max col 0)
        // Line 2: "123." (4 chars, max col 3)
        let mut state = test_state();
        state.cursor = Index2::new(0, 6);
        state.desired_display_col = 6;

        // Move down to empty line — clamps to col 0.
        MoveDown(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(1, 0));

        // Move down to "123." — desired 6 clamps to max col 3.
        MoveDown(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(2, 3));

        // Already on last row — no change.
        MoveDown(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(2, 3));
    }

    #[test]
    fn test_move_up() {
        // "Hello World!\n\n123."
        let mut state = test_state();
        state.cursor = Index2::new(2, 2);
        state.desired_display_col = 2;

        // Move up to empty line — clamps to col 0.
        MoveUp(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(1, 0));

        // Move up to "Hello World!" — desired 2 fits, lands on col 2.
        MoveUp(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 2));

        // Already on first row — no change.
        MoveUp(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 2));
    }

    #[test]
    fn test_move_down_preserves_desired_display_col() {
        // After moving to a short line and then to a longer line,
        // the cursor should restore to the original desired column.
        // "Hello World!\n\n123.\nAnother longer line"
        let mut state = EditorState::new(Lines::from("Hello World!\n\n123.\nAnother longer line"));
        // Move right to col 6, which sets desired_display_col.
        MoveForward(6).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 6));
        assert_eq!(state.desired_display_col, 6);

        // Move down to empty line — col 0.
        MoveDown(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(1, 0));
        // desired_display_col should still be 6.
        assert_eq!(state.desired_display_col, 6);

        // Move down to "123." — clamped to 3.
        MoveDown(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(2, 3));
        assert_eq!(state.desired_display_col, 6);

        // Move down to "Another longer line" — back to col 6.
        MoveDown(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(3, 6));
        assert_eq!(state.desired_display_col, 6);
    }

    #[test]
    fn test_move_up_preserves_desired_display_col() {
        // Similar to above but moving upward.
        let mut state = EditorState::new(Lines::from("Another longer line\n123.\n\nHello World!"));
        state.cursor = Index2::new(3, 0);
        MoveForward(6).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(3, 6));
        assert_eq!(state.desired_display_col, 6);

        // Move up to empty line — col 0.
        MoveUp(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(2, 0));
        assert_eq!(state.desired_display_col, 6);

        // Move up to "123." — clamped to 3.
        MoveUp(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(1, 3));
        assert_eq!(state.desired_display_col, 6);

        // Move up to "Another longer line" — back to col 6.
        MoveUp(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 6));
        assert_eq!(state.desired_display_col, 6);
    }

    #[test]
    fn test_horizontal_move_resets_desired_display_col() {
        let mut state = test_state();
        MoveForward(6).execute(&mut state);
        assert_eq!(state.desired_display_col, 6);

        // Moving backward changes desired_display_col.
        MoveBackward(2).execute(&mut state);
        assert_eq!(state.desired_display_col, 4);
        assert_eq!(state.cursor.col, 4);
    }

    #[test]
    fn test_move_word_forward() {
        let mut state = test_state();

        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 6));

        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 11));

        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(1, 0));

        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(2, 0));

        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(2, 3));
    }

    #[test]
    fn test_move_word_forward_with_punctuation() {
        let mut state = EditorState::new(Lines::from("forward (w)"));

        // Start at 'f', move forward through "forward" and skip space to land on '('
        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 8));

        // At '(', move through '(' and skip space (none) to land on 'w'
        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 9));

        // At 'w', move through 'w' to land on ')'
        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 10));
    }

    #[test]
    fn test_move_word_forward_punctuation_detailed() {
        // Test the exact case from the bug report
        let mut state = EditorState::new(Lines::from("forward (w)"));

        // Start at 'f' (position 0)
        assert_eq!(state.cursor, Index2::new(0, 0));

        // First move: from 'f' through "forward" to '('
        MoveWordForward(1).execute(&mut state);
        assert_eq!(
            state.cursor,
            Index2::new(0, 8),
            "Should move from 'f' to '(' after skipping whitespace"
        );

        // Second move: from '(' to 'w' (this was the bug - it went to ')' instead)
        MoveWordForward(1).execute(&mut state);
        assert_eq!(
            state.cursor,
            Index2::new(0, 9),
            "Should move from '(' to 'w', not to ')'"
        );

        // Third move: from 'w' to ')'
        MoveWordForward(1).execute(&mut state);
        assert_eq!(
            state.cursor,
            Index2::new(0, 10),
            "Should move from 'w' to ')'"
        );

        // Test with multiple punctuation characters
        let mut state = EditorState::new(Lines::from("hello()world"));

        // From 'h' to '('
        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 5));

        // From '(' through ')' to 'w' (consecutive punctuation treated as one word)
        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 7));

        // From 'w' through "world" to end
        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 11));
    }

    #[test]
    fn test_move_word_forward_out_of_bounds() {
        let mut state = test_state();

        state.cursor = Index2::new(0, 99);
        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(1, 0));
    }

    #[test]
    fn test_move_word_forward_to_end_of_word() {
        let mut state = test_state();

        MoveWordForwardToEndOfWord(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 4));

        MoveWordForwardToEndOfWord(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 10));

        MoveWordForwardToEndOfWord(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 11));

        MoveWordForwardToEndOfWord(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(2, 2));

        MoveWordForwardToEndOfWord(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(2, 3));

        MoveWordForwardToEndOfWord(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(2, 3));
    }

    #[test]
    fn test_move_word_backward() {
        let mut state = test_state();
        state.cursor = Index2::new(2, 3);

        MoveWordBackward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(2, 0));

        MoveWordBackward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(1, 0));

        MoveWordBackward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 11));

        MoveWordBackward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 6));

        MoveWordBackward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 0));

        MoveWordBackward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 0));
    }

    #[test]
    fn test_move_to_start() {
        let mut state = test_state();
        state.cursor = Index2::new(0, 2);

        MoveToStartOfLine().execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 0));
    }

    #[test]
    fn test_move_to_end() {
        let mut state = test_state();
        state.cursor = Index2::new(0, 2);

        MoveToEndOfLine().execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 11));
    }

    #[test]
    fn test_move_to_first() {
        let mut state = EditorState::new(Lines::from(" Hello"));
        state.cursor = Index2::new(0, 3);

        MoveToFirst().execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 1));
    }
}
