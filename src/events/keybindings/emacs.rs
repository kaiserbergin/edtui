use super::super::key::input::KeyCode;
use crate::actions::delete::{DeleteCharForward, DeleteToEndOfLine, DeleteToFirstCharOfLine};
use crate::actions::motion::{MoveHalfPageDown, MoveHalfPageUp, MoveToFirstRow, MoveToLastRow};
use crate::actions::search::StartSearch;
#[cfg(feature = "system-editor")]
use crate::actions::OpenSystemEditor;
use crate::actions::{
    Action, Chainable, DeleteChar, DeleteSelection, FindNext, FindPrevious, LineBreak,
    MoveBackward, MoveDown, MoveForward, MoveToEndOfLine, MoveToStartOfLine, MoveUp,
    MoveWordBackward, MoveWordForward, MoveWordForwardToEndOfWord, Paste, Redo, RemoveCharFromSearch,
    SelectCurrentSearch, StopSearch, SwitchMode, Undo,
};
use crate::events::{KeyEventRegister, KeyInput};
use crate::EditorMode;
use std::collections::HashMap;

#[allow(clippy::too_many_lines)]
pub fn key_bindings() -> HashMap<KeyEventRegister, Action> {
    HashMap::from([
        (
            KeyEventRegister::i(vec![KeyInput::ctrl('s')]),
            StartSearch.chain(SwitchMode(EditorMode::Search)).into(),
        ),
        (
            KeyEventRegister::s(vec![KeyInput::ctrl('s')]),
            FindNext.into(),
        ),
        (
            KeyEventRegister::s(vec![KeyInput::ctrl('r')]),
            FindPrevious.into(),
        ),
        (
            KeyEventRegister::s(vec![KeyInput::new(KeyCode::Enter)]),
            SelectCurrentSearch
                .chain(SwitchMode(EditorMode::Insert))
                .into(),
        ),
        (
            KeyEventRegister::s(vec![KeyInput::ctrl('g')]),
            StopSearch.chain(SwitchMode(EditorMode::Insert)).into(),
        ),
        (
            KeyEventRegister::s(vec![KeyInput::new(KeyCode::Backspace)]),
            RemoveCharFromSearch.into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl('f')]),
            MoveForward(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Right)]),
            MoveForward(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl('b')]),
            MoveBackward(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Left)]),
            MoveBackward(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl('p')]),
            MoveUp(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Up)]),
            MoveUp(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl('n')]),
            MoveDown(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Down)]),
            MoveDown(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::alt('f')]),
            MoveWordForward(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::alt('b')]),
            MoveWordBackward(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl('v')]),
            MoveHalfPageDown().into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::alt('v')]),
            MoveHalfPageUp().into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::alt('<')]),
            MoveToFirstRow().into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::alt('>')]),
            MoveToLastRow().into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl('a')]),
            MoveToStartOfLine().into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Home)]),
            MoveToStartOfLine().into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::End)]),
            MoveToEndOfLine().into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl('e')]),
            MoveToEndOfLine().into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::alt('u')]),
            DeleteToFirstCharOfLine.into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl('k')]),
            DeleteToEndOfLine.into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl('o')]),
            LineBreak(1)
                .chain(MoveUp(1))
                .chain(MoveToEndOfLine())
                .into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Enter)]),
            LineBreak(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl('j')]),
            LineBreak(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Backspace)]),
            DeleteChar(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl('h')]),
            DeleteChar(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Backspace)]),
            DeleteCharForward(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl('d')]),
            DeleteCharForward(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::alt('d')]),
            SwitchMode(EditorMode::Visual)
                .chain(MoveWordForwardToEndOfWord(1))
                .chain(DeleteSelection)
                .chain(SwitchMode(EditorMode::Insert))
                .into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::alt(KeyCode::Backspace)]),
            SwitchMode(EditorMode::Visual)
                .chain(MoveWordBackward(1))
                .chain(DeleteSelection)
                .chain(SwitchMode(EditorMode::Insert))
                .into(),
        ),
        (KeyEventRegister::i(vec![KeyInput::ctrl('u')]), Undo.into()),
        (KeyEventRegister::i(vec![KeyInput::ctrl('r')]), Redo.into()),
        (KeyEventRegister::i(vec![KeyInput::ctrl('y')]), Paste.into()),
        #[cfg(feature = "system-editor")]
        (
            KeyEventRegister::i(vec![KeyInput::alt('e')]),
            OpenSystemEditor.into(),
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_bindings_non_empty() {
        let map = key_bindings();
        assert!(!map.is_empty());
    }
}
