use super::super::key::input::KeyCode;
use crate::actions::cpaste::{CopySelection, PasteOverSelection};
use crate::actions::delete::DeleteCharForward;
use crate::actions::motion::{MoveHalfPageDown, MoveHalfPageUp, MoveToFirstRow, MoveToLastRow};
use crate::actions::search::StartSearch;
use crate::actions::{
    Action, Chainable, DeleteChar, DeleteSelection, FindNext, FindPrevious, LineBreak,
    MoveBackward, MoveDown, MoveForward, MoveToEndOfLine, MoveToStartOfLine, MoveUp,
    MoveWordBackward, MoveWordForward, Paste, Redo, RemoveCharFromSearch, SelectCurrentSearch,
    StopSearch, SwitchMode, Undo,
};
use crate::events::{KeyEventRegister, KeyInput};
use crate::EditorMode;
use crossterm::event::KeyModifiers;
use std::collections::HashMap;

use EditorMode::{Insert, Normal, Search, Visual};

/// Returns true if the key is a movement key (arrows, Home, End, PageUp, PageDown, Tab, BackTab).
pub fn is_movement_key(key: &KeyInput) -> bool {
    match key.key {
        KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => true,
        KeyCode::Home | KeyCode::End => true,
        KeyCode::PageUp | KeyCode::PageDown => true,
        KeyCode::Tab | KeyCode::BackTab => true,
        _ => false,
    }
}

pub fn key_bindings() -> HashMap<KeyEventRegister, Action> {
    let mut key_map = get_insert_mode_key_bindings();
    key_map.extend(get_visual_mode_key_bindings());
    key_map.extend(get_search_mode_key_bindings());
    key_map
}

fn get_visual_mode_key_bindings() -> HashMap<KeyEventRegister, Action> {
    let mut key_map = HashMap::from([
        (
            KeyEventRegister::v(vec![KeyInput::shift(KeyCode::Right)]),
            MoveForward(1).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::shift(KeyCode::Left)]),
            MoveBackward(1).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::shift(KeyCode::Up)]),
            MoveUp(1).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::shift(KeyCode::Down)]),
            MoveDown(1).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::with_modifiers(
                KeyCode::Up,
                KeyModifiers::SHIFT | KeyModifiers::CONTROL,
            )]),
            MoveHalfPageUp().into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::with_modifiers(
                KeyCode::Down,
                KeyModifiers::SHIFT | KeyModifiers::CONTROL,
            )]),
            MoveHalfPageDown().into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::with_modifiers(
                KeyCode::Right,
                KeyModifiers::SHIFT | KeyModifiers::CONTROL,
            )]),
            MoveWordForward(1).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::with_modifiers(
                KeyCode::Left,
                KeyModifiers::SHIFT | KeyModifiers::CONTROL,
            )]),
            MoveWordBackward(1).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::with_modifiers(
                KeyCode::Up,
                KeyModifiers::SHIFT | KeyModifiers::CONTROL | KeyModifiers::ALT,
            )]),
            MoveToFirstRow().chain(MoveToStartOfLine()).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::with_modifiers(
                KeyCode::Down,
                KeyModifiers::ALT | KeyModifiers::CONTROL | KeyModifiers::SHIFT,
            )]),
            MoveToLastRow().chain(MoveToEndOfLine()).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::with_modifiers(
                KeyCode::Left,
                KeyModifiers::ALT | KeyModifiers::CONTROL | KeyModifiers::SHIFT,
            )]),
            MoveToStartOfLine().into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::with_modifiers(
                KeyCode::Right,
                KeyModifiers::ALT | KeyModifiers::CONTROL | KeyModifiers::SHIFT,
            )]),
            MoveToEndOfLine().into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::ctrl(KeyCode::Char('c'))]),
            CopySelection.chain(SwitchMode(Insert)).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::ctrl(KeyCode::Char('C'))]),
            CopySelection.chain(SwitchMode(Insert)).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::ctrl(KeyCode::Char('x'))]),
            DeleteSelection.chain(SwitchMode(Insert)).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::ctrl(KeyCode::Char('X'))]),
            DeleteSelection.chain(SwitchMode(Insert)).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::ctrl(KeyCode::Char('v'))]),
            PasteOverSelection.chain(SwitchMode(Insert)).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::ctrl(KeyCode::Char('V'))]),
            PasteOverSelection.chain(SwitchMode(Insert)).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::ctrl(KeyCode::Char('z'))]),
            SwitchMode(Normal).chain(SwitchMode(Insert)).chain(Undo).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::ctrl(KeyCode::Char('Z'))]),
            SwitchMode(Normal).chain(SwitchMode(Insert)).chain(Undo).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::ctrl(KeyCode::Char('y'))]),
            SwitchMode(Normal).chain(SwitchMode(Insert)).chain(Redo).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::ctrl(KeyCode::Char('Y'))]),
            SwitchMode(Normal).chain(SwitchMode(Insert)).chain(Redo).into(),
        ),
    ]);

    key_map.extend(get_all_modifiers_map(
        Visual,
        KeyCode::Enter,
        DeleteSelection
            .chain(SwitchMode(Insert))
            .chain(LineBreak(1))
            .into(),
    ));
    key_map.extend(get_all_modifiers_map(
        Visual,
        KeyCode::Delete,
        DeleteSelection.chain(SwitchMode(Insert)).into(),
    ));
    key_map.extend(get_all_modifiers_map(
        Visual,
        KeyCode::Backspace,
        DeleteSelection.chain(SwitchMode(Insert)).into(),
    ));
    key_map.extend(get_all_modifiers_map(
        Visual,
        KeyCode::Esc,
        SwitchMode(Normal).chain(SwitchMode(Insert)).into(),
    ));

    key_map
}

fn get_insert_mode_key_bindings() -> HashMap<KeyEventRegister, Action> {
    HashMap::from([
        (
            KeyEventRegister::i(vec![KeyInput::ctrl('f')]),
            StartSearch.chain(SwitchMode(Search)).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl('F')]),
            StartSearch.chain(SwitchMode(Search)).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Backspace)]),
            DeleteChar(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Delete)]),
            DeleteCharForward(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Enter)]),
            LineBreak(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl(KeyCode::Backspace)]),
            MoveBackward(1)
                .chain(SwitchMode(Visual))
                .chain(MoveWordBackward(1))
                .chain(DeleteSelection)
                .chain(SwitchMode(Insert))
                .into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl(KeyCode::Delete)]),
            SwitchMode(Visual)
                .chain(MoveWordForward(1))
                .chain(MoveBackward(1))
                .chain(DeleteSelection)
                .chain(SwitchMode(Insert))
                .into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::with_modifiers(
                KeyCode::Backspace,
                KeyModifiers::ALT | KeyModifiers::CONTROL,
            )]),
            SwitchMode(Visual)
                .chain(MoveToStartOfLine())
                .chain(DeleteSelection)
                .chain(SwitchMode(Insert))
                .into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::with_modifiers(
                KeyCode::Delete,
                KeyModifiers::ALT | KeyModifiers::CONTROL,
            )]),
            SwitchMode(Visual)
                .chain(MoveToEndOfLine())
                .chain(DeleteSelection)
                .chain(SwitchMode(Insert))
                .into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Right)]),
            MoveForward(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Left)]),
            MoveBackward(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Up)]),
            MoveUp(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Down)]),
            MoveDown(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl(KeyCode::Up)]),
            MoveHalfPageUp().into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl(KeyCode::Down)]),
            MoveHalfPageDown().into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl(KeyCode::Right)]),
            MoveWordForward(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl(KeyCode::Left)]),
            MoveWordBackward(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::with_modifiers(
                KeyCode::Up,
                KeyModifiers::ALT | KeyModifiers::CONTROL,
            )]),
            MoveToFirstRow().chain(MoveToStartOfLine()).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::with_modifiers(
                KeyCode::Down,
                KeyModifiers::ALT | KeyModifiers::CONTROL,
            )]),
            MoveToLastRow().chain(MoveToEndOfLine()).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::with_modifiers(
                KeyCode::Left,
                KeyModifiers::ALT | KeyModifiers::CONTROL,
            )]),
            MoveToStartOfLine().into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::with_modifiers(
                KeyCode::Right,
                KeyModifiers::ALT | KeyModifiers::CONTROL,
            )]),
            MoveToEndOfLine().into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl(KeyCode::Char('v'))]),
            Paste.into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl(KeyCode::Char('V'))]),
            Paste.into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl(KeyCode::Char('z'))]),
            Undo.into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl(KeyCode::Char('Z'))]),
            Undo.into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl(KeyCode::Char('y'))]),
            Redo.into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl(KeyCode::Char('Y'))]),
            Redo.into(),
        ),
    ])
}

fn get_search_mode_key_bindings() -> HashMap<KeyEventRegister, Action> {
    HashMap::from([
        (
            KeyEventRegister::s(vec![KeyInput::new(KeyCode::Enter)]),
            SelectCurrentSearch.chain(SwitchMode(Insert)).into(),
        ),
        (
            KeyEventRegister::s(vec![KeyInput::new(KeyCode::Right)]),
            FindNext.into(),
        ),
        (
            KeyEventRegister::s(vec![KeyInput::new(KeyCode::Left)]),
            FindPrevious.into(),
        ),
        (
            KeyEventRegister::s(vec![KeyInput::new(KeyCode::Esc)]),
            StopSearch.chain(SwitchMode(Insert)).into(),
        ),
        (
            KeyEventRegister::s(vec![KeyInput::new(KeyCode::Backspace)]),
            RemoveCharFromSearch.into(),
        ),
    ])
}

pub(crate) fn get_all_modifiers_map(
    editor_mode: EditorMode,
    key_code: KeyCode,
    action: Action,
) -> HashMap<KeyEventRegister, Action> {
    match editor_mode {
        Normal => HashMap::from([
            (KeyEventRegister::n(vec![KeyInput::new(key_code)]), action.clone()),
            (KeyEventRegister::n(vec![KeyInput::ctrl(key_code)]), action.clone()),
            (KeyEventRegister::n(vec![KeyInput::alt(key_code)]), action.clone()),
            (KeyEventRegister::n(vec![KeyInput::shift(key_code)]), action.clone()),
            (
                KeyEventRegister::n(vec![KeyInput::with_modifiers(
                    key_code,
                    KeyModifiers::ALT | KeyModifiers::SHIFT | KeyModifiers::CONTROL,
                )]),
                action.clone(),
            ),
            (
                KeyEventRegister::n(vec![KeyInput::with_modifiers(
                    key_code,
                    KeyModifiers::ALT | KeyModifiers::SHIFT,
                )]),
                action.clone(),
            ),
            (
                KeyEventRegister::n(vec![KeyInput::with_modifiers(
                    key_code,
                    KeyModifiers::ALT | KeyModifiers::CONTROL,
                )]),
                action.clone(),
            ),
            (
                KeyEventRegister::n(vec![KeyInput::with_modifiers(
                    key_code,
                    KeyModifiers::SHIFT | KeyModifiers::CONTROL,
                )]),
                action.clone(),
            ),
        ]),
        Insert => HashMap::from([
            (KeyEventRegister::i(vec![KeyInput::new(key_code)]), action.clone()),
            (KeyEventRegister::i(vec![KeyInput::ctrl(key_code)]), action.clone()),
            (KeyEventRegister::i(vec![KeyInput::alt(key_code)]), action.clone()),
            (KeyEventRegister::i(vec![KeyInput::shift(key_code)]), action.clone()),
            (
                KeyEventRegister::i(vec![KeyInput::with_modifiers(
                    key_code,
                    KeyModifiers::ALT | KeyModifiers::SHIFT | KeyModifiers::CONTROL,
                )]),
                action.clone(),
            ),
            (
                KeyEventRegister::i(vec![KeyInput::with_modifiers(
                    key_code,
                    KeyModifiers::ALT | KeyModifiers::SHIFT,
                )]),
                action.clone(),
            ),
            (
                KeyEventRegister::i(vec![KeyInput::with_modifiers(
                    key_code,
                    KeyModifiers::ALT | KeyModifiers::CONTROL,
                )]),
                action.clone(),
            ),
            (
                KeyEventRegister::i(vec![KeyInput::with_modifiers(
                    key_code,
                    KeyModifiers::SHIFT | KeyModifiers::CONTROL,
                )]),
                action.clone(),
            ),
        ]),
        Visual => HashMap::from([
            (KeyEventRegister::v(vec![KeyInput::new(key_code)]), action.clone()),
            (KeyEventRegister::v(vec![KeyInput::ctrl(key_code)]), action.clone()),
            (KeyEventRegister::v(vec![KeyInput::alt(key_code)]), action.clone()),
            (KeyEventRegister::v(vec![KeyInput::shift(key_code)]), action.clone()),
            (
                KeyEventRegister::v(vec![KeyInput::with_modifiers(
                    key_code,
                    KeyModifiers::ALT | KeyModifiers::SHIFT | KeyModifiers::CONTROL,
                )]),
                action.clone(),
            ),
            (
                KeyEventRegister::v(vec![KeyInput::with_modifiers(
                    key_code,
                    KeyModifiers::ALT | KeyModifiers::SHIFT,
                )]),
                action.clone(),
            ),
            (
                KeyEventRegister::v(vec![KeyInput::with_modifiers(
                    key_code,
                    KeyModifiers::ALT | KeyModifiers::CONTROL,
                )]),
                action.clone(),
            ),
            (
                KeyEventRegister::v(vec![KeyInput::with_modifiers(
                    key_code,
                    KeyModifiers::SHIFT | KeyModifiers::CONTROL,
                )]),
                action.clone(),
            ),
        ]),
        Search => HashMap::from([
            (KeyEventRegister::s(vec![KeyInput::new(key_code)]), action.clone()),
            (KeyEventRegister::s(vec![KeyInput::ctrl(key_code)]), action.clone()),
            (KeyEventRegister::s(vec![KeyInput::alt(key_code)]), action.clone()),
            (KeyEventRegister::s(vec![KeyInput::shift(key_code)]), action.clone()),
            (
                KeyEventRegister::s(vec![KeyInput::with_modifiers(
                    key_code,
                    KeyModifiers::ALT | KeyModifiers::SHIFT | KeyModifiers::CONTROL,
                )]),
                action.clone(),
            ),
            (
                KeyEventRegister::s(vec![KeyInput::with_modifiers(
                    key_code,
                    KeyModifiers::ALT | KeyModifiers::SHIFT,
                )]),
                action.clone(),
            ),
            (
                KeyEventRegister::s(vec![KeyInput::with_modifiers(
                    key_code,
                    KeyModifiers::ALT | KeyModifiers::CONTROL,
                )]),
                action.clone(),
            ),
            (
                KeyEventRegister::s(vec![KeyInput::with_modifiers(
                    key_code,
                    KeyModifiers::SHIFT | KeyModifiers::CONTROL,
                )]),
                action.clone(),
            ),
        ]),
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::key::input::KeyCode;
    use super::*;
    use crate::events::KeyInput;

    #[test]
    fn test_is_movement_key_arrows() {
        assert!(is_movement_key(&KeyInput::new(KeyCode::Up)));
        assert!(is_movement_key(&KeyInput::new(KeyCode::Down)));
        assert!(is_movement_key(&KeyInput::new(KeyCode::Left)));
        assert!(is_movement_key(&KeyInput::new(KeyCode::Right)));
    }

    #[test]
    fn test_is_movement_key_navigation() {
        assert!(is_movement_key(&KeyInput::new(KeyCode::Home)));
        assert!(is_movement_key(&KeyInput::new(KeyCode::End)));
        assert!(is_movement_key(&KeyInput::new(KeyCode::PageUp)));
        assert!(is_movement_key(&KeyInput::new(KeyCode::PageDown)));
        assert!(is_movement_key(&KeyInput::new(KeyCode::Tab)));
        assert!(is_movement_key(&KeyInput::new(KeyCode::BackTab)));
    }

    #[test]
    fn test_is_movement_key_char_false() {
        assert!(!is_movement_key(&KeyInput::new('a')));
        assert!(!is_movement_key(&KeyInput::new(KeyCode::Enter)));
        assert!(!is_movement_key(&KeyInput::new(KeyCode::Esc)));
        assert!(!is_movement_key(&KeyInput::new(KeyCode::Backspace)));
    }

    #[test]
    fn test_key_bindings_non_empty() {
        let map = key_bindings();
        assert!(!map.is_empty());
    }
}
