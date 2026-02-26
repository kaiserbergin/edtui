use jagged::index::RowIndex;

use super::Execute;
use crate::{
    clipboard::ClipboardTrait,
    helper::{is_out_of_bounds, max_col_insert},
    state::selection::Selection,
    state::view::{cursor_visual_line_range, visual_line_count_for_row},
    EditorState, Index2, Lines,
};

/// Deletes a character at the current cursor position. Does not
/// move the cursor position unless it is at the end of the line
/// Intended to be called in normal mode.
#[derive(Clone, Debug, Copy)]
pub struct RemoveChar(pub usize);

impl Execute for RemoveChar {
    fn execute(&mut self, state: &mut EditorState) {
        state.capture();
        state.clamp_column();
        for _ in 0..self.0 {
            let lines = &mut state.lines;
            let index = &mut state.cursor;

            if is_out_of_bounds(lines, index) {
                return;
            }

            let _ = lines.remove(*index);
            index.col = index.col.min(
                lines
                    .len_col(index.row)
                    .unwrap_or_default()
                    .saturating_sub(1),
            );
        }
    }
}

/// Replaces the character under the cursor with a given character.
/// Intended to be called in normal mode.
#[derive(Clone, Debug, Copy)]
pub struct ReplaceChar(pub char);

impl Execute for ReplaceChar {
    fn execute(&mut self, state: &mut EditorState) {
        let index = state.cursor;
        if is_out_of_bounds(&state.lines, &index) {
            return;
        }
        state.capture();
        if let Some(ch) = state.lines.get_mut(index) {
            *ch = self.0;
        };
    }
}

/// Deletes a character to the left of the current cursor. Deletes
/// the line break if the the cursor is in column zero.
/// Intended to be called in insert mode.
#[derive(Clone, Debug, Copy)]
pub struct DeleteChar(pub usize);

impl Execute for DeleteChar {
    fn execute(&mut self, state: &mut EditorState) {
        state.capture();
        for _ in 0..self.0 {
            delete_char(&mut state.lines, &mut state.cursor);
        }
        state.update_desired_display_col();
    }
}

fn delete_char(lines: &mut Lines, index: &mut Index2) {
    fn move_left(lines: &Lines, index: &mut Index2) {
        if index.col > 0 {
            index.col -= 1;
        } else if index.row > 0 {
            index.row -= 1;
            index.col = lines.len_col(index.row).unwrap_or_default();
        }
    }

    let len_col = lines.len_col(index.row).unwrap_or_default();
    if len_col == 0 && index.row == 0 {
        return;
    }

    if index.col > len_col {
        index.col = len_col;
    }

    if index.col == 0 {
        let mut rest = lines.split_off(*index);
        move_left(lines, index);
        lines.merge(&mut rest);
    } else {
        let max_col = max_col_insert(lines, index);
        index.col = index.col.min(max_col);
        move_left(lines, index);
        let _ = lines.remove(*index);
    }
}

/// Deletes the character at the current cursor position.
/// If at the end of a line, deletes the newline character.
/// Intended to be called in insert mode.
#[derive(Clone, Debug, Copy)]
pub struct DeleteCharForward(pub usize);

impl Execute for DeleteCharForward {
    fn execute(&mut self, state: &mut EditorState) {
        state.capture();
        state.clamp_column();
        for _ in 0..self.0 {
            delete_char_forward(&mut state.lines, &mut state.cursor);
        }
        state.update_desired_display_col();
    }
}

fn delete_char_forward(lines: &mut Lines, index: &mut Index2) {
    let Some(row) = lines.get(RowIndex::new(index.row)) else {
        return;
    };

    let row_len = row.len();

    // If cursor is at or past the end of the line, delete the newline
    if index.col >= row_len {
        if index.row + 1 >= lines.len() {
            return;
        }

        lines.join_lines(index.row);
        return;
    }

    let _ = lines.remove(*index);
}

/// Deletes the current line (or current visual line when wrap is on).
#[derive(Clone, Debug, Copy)]
pub struct DeleteLine(pub usize);

impl Execute for DeleteLine {
    fn execute(&mut self, state: &mut EditorState) {
        state.capture();
        let wrap = state.view.wrap && state.view.screen_area.width > 0;
        for _ in 0..self.0 {
            if state.cursor.row >= state.lines.len() {
                break;
            }
            if wrap {
                let width = state.view.screen_area.width as usize;
                let tab_width = state.view.tab_width;
                let row = state.cursor.row;
                let col = state.cursor.col;

                let visual_count = state
                    .lines
                    .get(jagged::index::RowIndex::new(row))
                    .map_or(1, |l| visual_line_count_for_row(l, width, tab_width));

                if visual_count <= 1 {
                    let row_index = RowIndex::new(row);
                    let deleted_line = state.lines.remove(row_index).iter().collect::<String>();
                    state.clip.set_text(String::from('\n') + &deleted_line);
                    state.cursor.col = 0;
                    state.cursor.row = row.min(state.lines.len().saturating_sub(1));
                } else {
                    let (col_start, col_end) =
                        cursor_visual_line_range(&state.lines, row, col, width, tab_width);
                    let row_index = RowIndex::new(row);
                    if let Some(line) = state.lines.get_mut(row_index) {
                        let end = col_end.min(line.len());
                        let start = col_start.min(end);
                        let deleted: String = line.drain(start..end).collect();
                        state.clip.set_text(deleted);
                        let new_len = line.len();
                        state.cursor.col =
                            col_start.min(if new_len == 0 { 0 } else { new_len - 1 });
                    }
                }
            } else {
                let row_index = RowIndex::new(state.cursor.row);
                let deleted_line = state.lines.remove(row_index).iter().collect::<String>();
                state.clip.set_text(String::from('\n') + &deleted_line);
                state.cursor.col = 0;
                state.cursor.row = state.cursor.row.min(state.lines.len().saturating_sub(1));
            }
        }
    }
}

/// Deletes from the current cursor position to the first non-whitespace character of the
/// line (or visual line when wrap is on).
#[derive(Clone, Debug, Copy)]
pub struct DeleteToFirstCharOfLine;

impl Execute for DeleteToFirstCharOfLine {
    fn execute(&mut self, state: &mut EditorState) {
        state.capture();

        let wrap = state.view.wrap && state.view.screen_area.width > 0;
        let col = state.cursor.col;
        let row_num = state.cursor.row;

        let (line_start, line_end) = if wrap {
            let width = state.view.screen_area.width as usize;
            let tab_width = state.view.tab_width;
            cursor_visual_line_range(&state.lines, row_num, col, width, tab_width)
        } else {
            let line_len = state
                .lines
                .get(jagged::index::RowIndex::new(row_num))
                .map_or(0, |l| l.len());
            (0, line_len)
        };

        let row_index = RowIndex::new(row_num);
        let Some(row) = state.lines.get_mut(row_index) else {
            return;
        };

        let clamped_end = line_end.min(row.len());
        let first_char = row[line_start..clamped_end]
            .iter()
            .position(|c| !c.is_whitespace())
            .map(|p| line_start + p)
            .unwrap_or(line_start);

        let anchor = if col <= first_char { line_start } else { first_char };

        if anchor < col && col <= row.len() {
            let deleted = row.drain(anchor..col).collect();
            state.clip.set_text(deleted);
        }

        state.cursor.col = anchor;
    }
}

/// Deletes from the current cursor position to the end of the line (or visual line when
/// wrap is on).
#[derive(Clone, Debug, Copy)]
pub struct DeleteToEndOfLine;

impl Execute for DeleteToEndOfLine {
    fn execute(&mut self, state: &mut EditorState) {
        if is_out_of_bounds(&state.lines, &state.cursor) {
            return;
        }
        state.capture();

        let wrap = state.view.wrap && state.view.screen_area.width > 0;
        let cursor_col = state.cursor.col;
        let cursor_row = state.cursor.row;

        let drain_end = if wrap {
            let width = state.view.screen_area.width as usize;
            let tab_width = state.view.tab_width;
            let (_, col_end) =
                cursor_visual_line_range(&state.lines, cursor_row, cursor_col, width, tab_width);
            col_end
        } else {
            usize::MAX
        };

        let Some(row) = state.lines.get_mut(RowIndex::new(cursor_row)) else {
            return;
        };
        let end = drain_end.min(row.len());
        let deleted: String = row.drain(cursor_col..end).collect();
        state.cursor.col = cursor_col.saturating_sub(1);
        state.clip.set_text(deleted);
    }
}

/// Deletes the current selection.
#[derive(Clone, Debug)]
pub struct DeleteSelection;

impl Execute for DeleteSelection {
    fn execute(&mut self, state: &mut EditorState) {
        if let Some(selection) = state.selection.take() {
            state.capture();
            let drained = delete_selection(state, &selection);
            state.clip.set_text(drained.into());
        }
        state.selection = None;
    }
}

pub(crate) fn delete_selection(state: &mut EditorState, selection: &Selection) -> Lines {
    state.cursor = selection.start();
    state.clamp_column();
    selection.extract_from(&mut state.lines)
}

/// Joins line below to the current line.
#[derive(Clone, Debug, Copy)]
pub struct JoinLineWithLineBelow;

impl Execute for JoinLineWithLineBelow {
    fn execute(&mut self, state: &mut EditorState) {
        if state.cursor.row + 1 >= state.lines.len() {
            return;
        }
        state.capture();
        state.lines.join_lines(state.cursor.row);
    }
}

#[cfg(test)]
mod tests {
    use crate::state::selection::Selection;
    use crate::EditorMode;
    use crate::Index2;
    use crate::Lines;

    use super::*;
    fn test_state() -> EditorState {
        EditorState::new(Lines::from("Hello World!\n\n123."))
    }

    #[test]
    fn test_remove_char() {
        let mut state = test_state();

        state.cursor = Index2::new(0, 4);
        RemoveChar(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 4));
        assert_eq!(state.lines, Lines::from("Hell World!\n\n123."));

        state.cursor = Index2::new(0, 10);
        RemoveChar(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 9));
        assert_eq!(state.lines, Lines::from("Hell World\n\n123."));
    }

    #[test]
    fn test_replace_char() {
        let mut state = test_state();

        state.cursor = Index2::new(0, 4);
        ReplaceChar('x').execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 4));
        assert_eq!(state.lines, Lines::from("Hellx World!\n\n123."));

        // do nothing on empty line
        state.cursor = Index2::new(1, 0);
        ReplaceChar('x').execute(&mut state);
        assert_eq!(state.cursor, Index2::new(1, 0));
        assert_eq!(state.lines, Lines::from("Hellx World!\n\n123."));

        // do nothing if out of bounds
        state.cursor = Index2::new(99, 0);
        ReplaceChar('x').execute(&mut state);
        assert_eq!(state.cursor, Index2::new(99, 0));
        assert_eq!(state.lines, Lines::from("Hellx World!\n\n123."));
    }

    #[test]
    fn test_delete_char() {
        let mut state = test_state();

        state.cursor = Index2::new(0, 5);
        DeleteChar(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 4));
        assert_eq!(state.lines, Lines::from("Hell World!\n\n123."));

        state.cursor = Index2::new(0, 11);
        DeleteChar(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 10));
        assert_eq!(state.lines, Lines::from("Hell World\n\n123."));
    }

    #[test]
    fn test_delete_char_empty_line() {
        let mut state = test_state();
        state.cursor = Index2::new(1, 99);

        DeleteChar(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 12));
        assert_eq!(state.lines, Lines::from("Hello World!\n123."));

        let mut state = EditorState::new(Lines::from("\nb"));
        state.cursor = Index2::new(0, 1);
        DeleteChar(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 1));
        assert_eq!(state.lines, Lines::from("\nb"));
    }

    #[test]
    fn test_delete_line() {
        let mut state = test_state();
        state.cursor = Index2::new(2, 3);

        DeleteLine(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(1, 0));
        assert_eq!(state.lines, Lines::from("Hello World!\n"));

        DeleteLine(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 0));
        assert_eq!(state.lines, Lines::from("Hello World!"));

        DeleteLine(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 0));
        assert_eq!(state.lines, Lines::from(""));
    }

    #[test]
    fn test_delete_to_first_char_of_line() {
        let mut state = EditorState::new(Lines::from("  Hello World!"));
        state.cursor = Index2::new(0, 4);

        DeleteToFirstCharOfLine.execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 2));
        assert_eq!(state.lines, Lines::from("  llo World!"));

        state.cursor = Index2::new(0, 2);
        DeleteToFirstCharOfLine.execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 0));
        assert_eq!(state.lines, Lines::from("llo World!"));
    }

    #[test]
    fn test_delete_to_end_of_line() {
        let mut state = test_state();
        state.cursor = Index2::new(0, 3);

        DeleteToEndOfLine.execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 2));
        assert_eq!(state.lines, Lines::from("Hel\n\n123."));
    }

    #[test]
    fn test_delete_selection() {
        let mut state = test_state();
        let st = Index2::new(0, 1);
        let en = Index2::new(2, 0);
        state.selection = Some(Selection::new(st, en));

        DeleteSelection.execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 1));
        assert_eq!(state.lines, Lines::from("H23."));
    }

    #[test]
    fn test_delete_selection_out_of_bounds() {
        let mut state = EditorState::new(Lines::from("123.\nHello World!\n456."));
        let st = Index2::new(0, 5);
        let en = Index2::new(2, 10);
        state.selection = Some(Selection::new(st, en));

        DeleteSelection.execute(&mut state);
        // assert_eq!(state.cursor, Index2::new(0, 1));
        assert_eq!(state.lines, Lines::from("123."));
    }

    #[test]
    fn test_delete_char_forward() {
        let mut state = EditorState::new(Lines::from("Hello World!\nNext line"));
        state.mode = EditorMode::Insert;

        // Delete character 'H'
        state.cursor = Index2::new(0, 0);
        DeleteCharForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 0));
        assert_eq!(state.lines, Lines::from("ello World!\nNext line"));

        // Delete character 'e'
        state.cursor = Index2::new(0, 0);
        DeleteCharForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 0));
        assert_eq!(state.lines, Lines::from("llo World!\nNext line"));

        // Delete character at end of line (newline)
        state.cursor = Index2::new(0, 10);
        DeleteCharForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 10));
        assert_eq!(state.lines, Lines::from("llo World!Next line"));
    }

    fn wrap_state(text: &str, width: u16) -> EditorState {
        use ratatui_core::layout::Rect;
        let mut state = EditorState::new(Lines::from(text));
        state.view.screen_area = Rect { x: 0, y: 0, width, height: 20 };
        state
    }

    #[test]
    fn test_delete_line_wrap_single_visual_row() {
        // "Hello\nWorld" at width 20: each line is a single visual row.
        // dd on line 0 should delete the entire logical line.
        let mut state = wrap_state("Hello\nWorld", 20);
        state.cursor = Index2::new(0, 2);
        DeleteLine(1).execute(&mut state);
        assert_eq!(state.lines, Lines::from("World"));
        assert_eq!(state.cursor, Index2::new(0, 0));
    }

    #[test]
    fn test_delete_line_wrap_multi_visual_row_first() {
        // "0123456789" at width 4 → visual rows: ["0123", "4567", "89"].
        // Cursor on first visual row (col 1) → drain [0, 4) → line becomes "456789".
        let mut state = wrap_state("0123456789", 4);
        state.cursor = Index2::new(0, 1);
        DeleteLine(1).execute(&mut state);
        assert_eq!(state.lines, Lines::from("456789"));
        assert_eq!(state.cursor.row, 0);
        assert_eq!(state.cursor.col, 0);
    }

    #[test]
    fn test_delete_line_wrap_multi_visual_row_middle() {
        // Cursor on second visual row (col 5) → drain [4, 8) → line becomes "012389".
        let mut state = wrap_state("0123456789", 4);
        state.cursor = Index2::new(0, 5);
        DeleteLine(1).execute(&mut state);
        assert_eq!(state.lines, Lines::from("012389"));
        assert_eq!(state.cursor.col, 4);
    }

    #[test]
    fn test_delete_line_wrap_multi_visual_row_last() {
        // Cursor on last visual row (col 8) → drain [8, 10) → line becomes "01234567".
        let mut state = wrap_state("0123456789", 4);
        state.cursor = Index2::new(0, 8);
        DeleteLine(1).execute(&mut state);
        assert_eq!(state.lines, Lines::from("01234567"));
        assert_eq!(state.cursor.col, 7);
    }

    #[test]
    fn test_delete_to_end_of_line_wrap() {
        // "0123456789" at width 4: cursor at col 5 (second visual row).
        // D deletes [5, 8) → "01234" + "89" = "0123489".
        let mut state = wrap_state("0123456789", 4);
        state.cursor = Index2::new(0, 5);
        DeleteToEndOfLine.execute(&mut state);
        assert_eq!(state.lines, Lines::from("0123489"));
        assert_eq!(state.cursor.col, 4);
    }

    #[test]
    fn test_delete_to_end_of_line_no_wrap_unchanged() {
        let mut state = EditorState::new(Lines::from("Hello World!"));
        state.view.wrap = false;
        state.cursor = Index2::new(0, 3);
        DeleteToEndOfLine.execute(&mut state);
        assert_eq!(state.lines, Lines::from("Hel"));
        assert_eq!(state.cursor.col, 2);
    }

    #[test]
    fn test_delete_to_first_char_wrap() {
        // "  abcd  efgh" at width 8: visual rows ["  abcd  ", "efgh"].
        // Cursor at col 9 on second visual row "efgh" → col_start=8, first non-ws=8.
        // col <= first_char, so anchor = col_start = 8. Nothing deleted.
        let mut state = wrap_state("  abcd  efgh", 8);
        state.cursor = Index2::new(0, 9);
        DeleteToFirstCharOfLine.execute(&mut state);
        // cursor at first non-ws of visual row which is col 8; cursor was at 9 > 8
        // anchor = first_char=8, delete [8..9] → "  abcd  fgh"
        assert_eq!(state.cursor.col, 8);
        assert_eq!(state.lines, Lines::from("  abcd  fgh"));
    }

    #[test]
    fn test_delete_char_forward_at_end() {
        let mut state = EditorState::new(Lines::from("Hello\nWorld"));
        state.mode = EditorMode::Insert;

        // Cursor at end of first line should delete newline
        state.cursor = Index2::new(0, 5);
        DeleteCharForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 5));
        assert_eq!(state.lines, Lines::from("HelloWorld"));

        // Cursor at end of last line should do nothing
        state.cursor = Index2::new(0, 10);
        DeleteCharForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 10));
        assert_eq!(state.lines, Lines::from("HelloWorld"));
    }
}
