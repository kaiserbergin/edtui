use std::cmp::min;

use jagged::{index::RowIndex, Index2};

use crate::{
    clipboard::ClipboardTrait,
    helper::{append_str, insert_str, max_row},
    state::view::{col_to_visual_row_in_wrapped, visual_row_col_range},
    view::line_wrapper::LineWrapper,
    EditorState,
};

#[cfg(test)]
use crate::EditorMode;

use super::{delete::delete_selection, Execute};

#[derive(Clone, Debug)]
pub struct Paste;

impl Execute for Paste {
    fn execute(&mut self, state: &mut EditorState) {
        let s = state.clip.get_text();
        if s.is_empty() {
            return;
        }

        state.capture();
        state.clamp_column();

        let s = if let Some(stripped) = s.strip_prefix('\n') {
            state.cursor = Index2::new(min(max_row(state), state.cursor.row + 1), 0);
            state.lines.insert(RowIndex::new(state.cursor.row), vec![]);
            stripped
        } else {
            state.clamp_column();
            &s
        };

        append_str(&mut state.lines, &mut state.cursor, s);
    }
}

#[derive(Clone, Debug)]
pub struct PasteOverSelection;

impl Execute for PasteOverSelection {
    fn execute(&mut self, state: &mut EditorState) {
        if let Some(selection) = state.selection.take() {
            state.capture();
            state.clamp_column();
            let _ = delete_selection(state, &selection);
            insert_str(&mut state.lines, &mut state.cursor, &state.clip.get_text());
        }
    }
}

#[derive(Clone, Debug)]
pub struct CopySelection;

impl Execute for CopySelection {
    fn execute(&mut self, state: &mut EditorState) {
        if let Some(s) = &state.selection {
            state.clip.set_text(s.copy_from(&state.lines).into());
            state.selection = None;
        }
    }
}

#[derive(Clone, Debug)]
pub struct CopyLine;

impl Execute for CopyLine {
    fn execute(&mut self, state: &mut EditorState) {
        let wrap = state.view.wrap && state.view.screen_area.width > 0;
        if let Some(line) = state.lines.get(RowIndex::new(state.cursor.row)) {
            if wrap {
                let width = state.view.screen_area.width as usize;
                let tab_width = state.view.tab_width;
                let wrapped = LineWrapper::wrap_line(line, width, tab_width);
                if wrapped.len() <= 1 {
                    let text = String::from('\n') + &line.iter().collect::<String>();
                    state.clip.set_text(text);
                } else {
                    let visual_row = col_to_visual_row_in_wrapped(&wrapped, state.cursor.col);
                    let (col_start, col_end) = visual_row_col_range(&wrapped, visual_row);
                    let end = col_end.min(line.len());
                    let text: String = line[col_start..end].iter().collect();
                    state.clip.set_text(text);
                }
            } else {
                let text = String::from('\n') + &line.iter().collect::<String>();
                state.clip.set_text(text);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::actions::Undo;
    use crate::clipboard::InternalClipboard;
    use crate::state::selection::Selection;
    use crate::Index2;
    use crate::Lines;

    use super::*;
    fn test_state() -> EditorState {
        let mut state = EditorState::new(Lines::from("Hello World!\n\n123."));
        state.set_clipboard(InternalClipboard::default());
        state
    }

    #[test]
    fn test_copy_paste() {
        let mut state = test_state();
        let selection = Selection::new(Index2::new(0, 0), Index2::new(0, 2));
        state.selection = Some(selection);

        CopySelection.execute(&mut state);
        Paste.execute(&mut state);

        assert_eq!(state.cursor, Index2::new(0, 3));
        assert_eq!(state.lines, Lines::from("HHelello World!\n\n123."));
    }

    #[test]
    fn test_paste_with_newline_into_empty_buffer() {
        let mut state = EditorState::default();
        state.set_clipboard(InternalClipboard::default());
        state.clip.set_text("\ntext".to_string());

        Paste.execute(&mut state);

        assert_eq!(state.cursor, Index2::new(0, 3));
        assert_eq!(state.lines, Lines::from("text"));
    }

    fn wrap_state(text: &str, width: u16) -> EditorState {
        use ratatui_core::layout::Rect;
        let mut state = EditorState::new(Lines::from(text));
        state.set_clipboard(InternalClipboard::default());
        state.view.screen_area = Rect { x: 0, y: 0, width, height: 20 };
        state
    }

    #[test]
    fn test_copy_line_wrap_single_visual_row() {
        // "Hello" at width 20: single visual row → full-line copy with \n prefix.
        let mut state = wrap_state("Hello", 20);
        state.cursor = Index2::new(0, 2);
        CopyLine.execute(&mut state);
        assert_eq!(state.clip.get_text(), "\nHello");
    }

    #[test]
    fn test_copy_line_wrap_multi_first_row() {
        // "0123456789" at width 4 → ["0123", "4567", "89"].
        // Cursor at col 1 (first visual row) → copies "0123" (no \n prefix).
        let mut state = wrap_state("0123456789", 4);
        state.cursor = Index2::new(0, 1);
        CopyLine.execute(&mut state);
        assert_eq!(state.clip.get_text(), "0123");
    }

    #[test]
    fn test_copy_line_wrap_multi_second_row() {
        // Cursor at col 5 (second visual row) → copies "4567".
        let mut state = wrap_state("0123456789", 4);
        state.cursor = Index2::new(0, 5);
        CopyLine.execute(&mut state);
        assert_eq!(state.clip.get_text(), "4567");
    }

    #[test]
    fn test_copy_line_wrap_multi_last_row() {
        // Cursor at col 9 (last visual row "89") → copies "89".
        let mut state = wrap_state("0123456789", 4);
        state.cursor = Index2::new(0, 9);
        CopyLine.execute(&mut state);
        assert_eq!(state.clip.get_text(), "89");
    }

    #[test]
    fn test_copy_line_wrap_inline_paste() {
        // Visual-row yank (no \n prefix) → Paste inserts inline after cursor, not as new line.
        let mut state = wrap_state("0123456789\nXXX", 4);
        state.cursor = Index2::new(0, 1); // first visual row → copies "0123"
        CopyLine.execute(&mut state);
        // Move to line 1 and paste: inline paste inserts "0123" after cursor char at col 0.
        state.cursor = Index2::new(1, 0);
        Paste.execute(&mut state);
        // 'p' pastes after cursor: 'X' + "0123" + "XX" = "X0123XX"
        assert_eq!(state.lines, Lines::from("0123456789\nX0123XX"));
    }

    #[test]
    fn test_paste_over_selection() {
        let mut state = test_state();
        state.selection = Some(Selection::new(Index2::new(0, 6), Index2::new(0, 10)));
        state.clip.set_text(String::from("Earth"));
        state.mode = EditorMode::Visual;

        PasteOverSelection.execute(&mut state);

        assert_eq!(state.lines, Lines::from("Hello Earth!\n\n123."));
        assert_eq!(state.cursor, Index2::new(0, 10));
        assert_eq!(state.mode, EditorMode::Visual);

        Undo.execute(&mut state);

        assert_eq!(state.lines, Lines::from("Hello World!\n\n123."));
        assert_eq!(state.mode, EditorMode::Visual);
    }
}
