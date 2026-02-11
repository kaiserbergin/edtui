use crate::actions::{DeleteSelection, Execute, SwitchMode};
use crate::events::keybindings::ms_word::is_movement_key;
use crate::events::key::input::KeyCode;
use crate::events::{EditorEventHandler, Event, KeyInput};
use crate::EditorMode;
use crate::EditorState;

/// Event handler that wraps MS Word key bindings with Windows-style selection behavior:
/// - Shift + movement in Insert enters Visual mode
/// - Movement without Shift in Visual clears selection and returns to Insert
/// - Typing a character (no Ctrl/Alt) in Visual deletes selection and inserts the character
#[derive(Clone, Debug)]
pub struct MsWordEditorEventHandler {
    inner: EditorEventHandler,
}

impl Default for MsWordEditorEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl MsWordEditorEventHandler {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: EditorEventHandler::ms_word_mode(),
        }
    }

    /// Handles key and mouse events with Windows-style selection rules applied first.
    pub fn on_event<T>(&mut self, event: T, state: &mut EditorState)
    where
        T: Into<Event>,
    {
        let event = event.into();
        if let Event::Key(key_input) = &event {
            self.apply_windows_selection_rules(key_input, state);
        }
        self.inner.on_event(event, state);
    }

    /// Handles key events with Windows-style selection rules applied first.
    pub fn on_key_event<T>(&mut self, event: T, state: &mut EditorState)
    where
        T: Into<KeyInput>,
    {
        let key_input = event.into();
        self.apply_windows_selection_rules(&key_input, state);
        self.inner.on_key_event(key_input, state);
    }

    fn apply_windows_selection_rules(&self, key_input: &KeyInput, state: &mut EditorState) {
        // Shift + movement in Insert → enter Visual
        if state.mode == EditorMode::Insert
            && key_input.modifiers.has_shift()
            && is_movement_key(key_input)
        {
            SwitchMode(EditorMode::Visual).execute(state);
            return;
        }

        // Movement without Shift in Visual → clear selection and return to Insert
        if state.mode == EditorMode::Visual
            && !key_input.modifiers.has_shift()
            && is_movement_key(key_input)
        {
            state.selection = None;
            SwitchMode(EditorMode::Insert).execute(state);
            return;
        }

        // In Visual, character key (no Ctrl/Alt) → delete selection and go to Insert
        if state.mode == EditorMode::Visual
            && !key_input.modifiers.has_control()
            && !key_input.modifiers.has_alt()
            && matches!(key_input.key, KeyCode::Char(_))
        {
            DeleteSelection.execute(state);
            SwitchMode(EditorMode::Insert).execute(state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::key::input::KeyCode;
    use crate::state::selection::Selection;
    use crate::{Index2, Lines};

    #[test]
    fn shift_movement_in_insert_enters_visual() {
        let mut state = EditorState::new(Lines::from("hello"));
        state.mode = EditorMode::Insert;

        let mut handler = MsWordEditorEventHandler::new();
        handler.on_key_event(KeyInput::shift(KeyCode::Right), &mut state);

        assert_eq!(state.mode, EditorMode::Visual);
    }

    #[test]
    fn movement_without_shift_in_visual_clears_and_enters_insert() {
        let mut state = EditorState::new(Lines::from("hello"));
        state.mode = EditorMode::Visual;
        state.selection = Some(Selection::new(Index2::new(0, 0), Index2::new(0, 2)));

        let mut handler = MsWordEditorEventHandler::new();
        handler.on_key_event(KeyInput::new(KeyCode::Right), &mut state);

        assert_eq!(state.mode, EditorMode::Insert);
        assert!(state.selection.is_none());
    }

    #[test]
    fn char_in_visual_deletes_selection_and_inserts() {
        let mut state = EditorState::new(Lines::from("hello"));
        state.mode = EditorMode::Visual;
        state.selection = Some(Selection::new(Index2::new(0, 1), Index2::new(0, 4))); // "ell" selected

        let mut handler = MsWordEditorEventHandler::new();
        handler.on_key_event(KeyInput::new('x'), &mut state);

        assert_eq!(state.mode, EditorMode::Insert);
        // After DeleteSelection + SwitchMode(Insert), the 'x' is processed by inner handler.
        // Selection (0,1)-(0,4) removes a range leaving "h"; cursor at 1; insert 'x' gives "hx".
        assert_eq!(state.lines.to_string(), "hx");
    }
}
