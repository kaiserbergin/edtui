//! The editors state
pub mod mode;
mod search;
pub mod selection;
mod undo;
pub(crate) mod view;

use self::search::SearchState;
use self::view::{cursor_display_col, ViewState};
use self::{mode::EditorMode, selection::Selection, undo::Stack};
use crate::actions::Execute;
use crate::clipboard::{Clipboard, ClipboardTrait};
use crate::helper::max_col;
use crate::{Index2, Lines};

/// Represents the state of an editor.
#[derive(Clone, Debug)]
pub struct EditorState {
    /// The text in the editor.
    pub lines: Lines,

    /// The current cursor position in the editor.
    pub cursor: Index2,

    /// The mode of the editor (insert, visual or normal mode).
    pub mode: EditorMode,

    /// Represents the selection in the editor, if any.
    pub selection: Option<Selection>,

    /// Internal view state of the editor.
    pub(crate) view: ViewState,

    /// State holding the search results in search mode.
    pub(crate) search: SearchState,

    /// Stack for undo operations.
    pub(crate) undo: Stack,

    /// Stack for redo operations.
    pub(crate) redo: Stack,

    /// Clipboard for yank and paste operations.
    pub(crate) clip: Clipboard,

    /// The desired display (visual/screen) column to aim for when moving
    /// vertically. Updated by horizontal movements; used by up/down to
    /// preserve the same screen position across lines.
    pub(crate) desired_display_col: usize,

    /// Flag indicating a system editor was requested.
    #[cfg(feature = "system-editor")]
    pub(crate) system_edit_requested: bool,
}

impl Default for EditorState {
    /// Creates a default `EditorState` with no text.
    fn default() -> Self {
        EditorState::new(Lines::default())
    }
}

impl EditorState {
    /// Creates a new editor state.
    ///
    /// # Example
    ///
    /// ```
    /// use edtui::{EditorState, Lines};
    ///
    /// let state = EditorState::new(Lines::from("First line\nSecond Line"));
    /// ```
    #[must_use]
    pub fn new(lines: Lines) -> EditorState {
        EditorState {
            lines,
            cursor: Index2::new(0, 0),
            mode: EditorMode::Normal,
            selection: None,
            view: ViewState::default(),
            search: SearchState::default(),
            undo: Stack::new(),
            redo: Stack::new(),
            clip: Clipboard::default(),
            desired_display_col: 0,
            #[cfg(feature = "system-editor")]
            system_edit_requested: false,
        }
    }

    /// Execute an action on the editor state
    /// # Example
    ///
    /// ```
    /// use edtui::{EditorState, Lines};
    /// use edtui::actions::DeleteLine;
    ///
    /// let mut state = EditorState::new(Lines::from("Hello wold!"));
    /// state.execute(DeleteLine(1))
    /// ```
    pub fn execute(&mut self, mut action: impl Execute) {
        action.execute(self);
    }

    /// Set a custom clipboard.
    pub fn set_clipboard(&mut self, clipboard: impl ClipboardTrait + 'static) {
        self.clip = Clipboard::new(clipboard);
    }

    /// Returns the current search pattern.
    #[must_use]
    pub fn search_pattern(&self) -> String {
        self.search.pattern.clone()
    }

    /// Clears selection and search state (e.g. when switching keybinding mode).
    pub fn clear_selection_and_search(&mut self) {
        self.selection = None;
        self.search.clear();
    }

    /// Clamps the column of the cursor if the cursor is out of bounds.
    /// In normal or visual mode, clamps on `col = len() - 1`, in insert
    /// mode on `col = len()`.
    pub(crate) fn clamp_column(&mut self) {
        let max_col = max_col(&self.lines, &self.cursor, self.mode);
        self.cursor.col = self.cursor.col.min(max_col);
    }

    /// Updates `desired_display_col` from the current cursor position.
    /// Call this after any horizontal movement so that the next up/down
    /// aims at the same visual column.
    pub(crate) fn update_desired_display_col(&mut self) {
        let width = self.view.screen_area.width as usize;
        let tab_width = self.view.tab_width;
        // When width is 0 (e.g. before the first render) use a very large
        // width so the display column equals the absolute character width
        // from the start of the line.
        let effective_width = if width > 0 { width } else { usize::MAX };
        self.desired_display_col = cursor_display_col(
            &self.lines,
            self.cursor.row,
            self.cursor.col,
            effective_width,
            tab_width,
        );
    }
}
