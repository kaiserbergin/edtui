use super::super::key::input::KeyCode;
use crate::actions::cpaste::PasteOverSelection;
use crate::actions::delete::{
    DeleteCharForward, DeleteToEndOfLine, DeleteToFirstCharOfLine,
};
use crate::actions::motion::{MoveHalfPageDown, MoveToFirstRow, MoveToLastRow};
use crate::actions::search::StartSearch;
#[cfg(feature = "system-editor")]
use crate::actions::OpenSystemEditor;
use crate::actions::{
    Action, AppendNewline, Chainable, ChangeAroundWord, ChangeInnerBetween, ChangeInnerWord,
    ChangeSelection, ChangeWord, CopyLine, CopySelection, DeleteChar, DeleteLine, DeleteSelection,
    FindFirst, FindNext, FindPrevious, InsertNewline, JoinLineWithLineBelow, LineBreak, MoveBackward,
    MoveDown, MoveForward, MoveHalfPageUp, MoveToEndOfLine, MoveToFirst, MoveToMatchinBracket,
    MoveToStartOfLine, MoveUp, MoveWordBackward, MoveWordForward, MoveWordForwardToEndOfWord,
    Paste, Redo, RemoveChar, RemoveCharFromSearch, SelectInnerBetween, SelectInnerWord, SelectLine,
    StopSearch, SwitchMode, Undo,
};
use crate::events::{KeyEventRegister, KeyInput};
use crate::EditorMode;
use std::collections::HashMap;

#[allow(clippy::too_many_lines)]
pub fn key_bindings() -> HashMap<KeyEventRegister, Action> {
    #[allow(unused_mut)]
    let mut map = HashMap::from([
        // Go into normal mode
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Esc)]),
            SwitchMode(EditorMode::Normal).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new('j'), KeyInput::new('j')]),
            SwitchMode(EditorMode::Normal).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new(KeyCode::Esc)]),
            SwitchMode(EditorMode::Normal).into(),
        ),
        // Go into insert mode
        (
            KeyEventRegister::n(vec![KeyInput::new('i')]),
            SwitchMode(EditorMode::Insert).into(),
        ),
        // Go into visual mode
        (
            KeyEventRegister::n(vec![KeyInput::new('v')]),
            SwitchMode(EditorMode::Visual).into(),
        ),
        // Goes into search mode and starts of a new search.
        (
            KeyEventRegister::n(vec![KeyInput::new('/')]),
            StartSearch.chain(SwitchMode(EditorMode::Search)).into(),
        ),
        // Trigger initial search
        (
            KeyEventRegister::s(vec![KeyInput::new(KeyCode::Enter)]),
            FindFirst.chain(SwitchMode(EditorMode::Normal)).into(),
        ),
        // Find next
        (
            KeyEventRegister::n(vec![KeyInput::new('n')]),
            FindNext.into(),
        ),
        // Find previous
        (
            KeyEventRegister::n(vec![KeyInput::shift('N')]),
            FindPrevious.into(),
        ),
        // Clear search
        (
            KeyEventRegister::s(vec![KeyInput::new(KeyCode::Esc)]),
            StopSearch.chain(SwitchMode(EditorMode::Normal)).into(),
        ),
        // Delete last character from search
        (
            KeyEventRegister::s(vec![KeyInput::new(KeyCode::Backspace)]),
            RemoveCharFromSearch.into(),
        ),
        // Go into insert mode and move one char forward
        (
            KeyEventRegister::n(vec![KeyInput::new('a')]),
            SwitchMode(EditorMode::Insert).chain(MoveForward(1)).into(),
        ),
        // Move cursor forward
        (
            KeyEventRegister::n(vec![KeyInput::new('l')]),
            MoveForward(1).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('l')]),
            MoveForward(1).into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::new(KeyCode::Right)]),
            MoveForward(1).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new(KeyCode::Right)]),
            MoveForward(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Right)]),
            MoveForward(1).into(),
        ),
        // Move cursor backward
        (
            KeyEventRegister::n(vec![KeyInput::new('h')]),
            MoveBackward(1).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('h')]),
            MoveBackward(1).into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::new(KeyCode::Left)]),
            MoveBackward(1).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new(KeyCode::Left)]),
            MoveBackward(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Left)]),
            MoveBackward(1).into(),
        ),
        // Move cursor up
        (
            KeyEventRegister::n(vec![KeyInput::new('k')]),
            MoveUp(1).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('k')]),
            MoveUp(1).into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::new(KeyCode::Up)]),
            MoveUp(1).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new(KeyCode::Up)]),
            MoveUp(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Up)]),
            MoveUp(1).into(),
        ),
        // Move cursor down
        (
            KeyEventRegister::n(vec![KeyInput::new('j')]),
            MoveDown(1).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('j')]),
            MoveDown(1).into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::new(KeyCode::Down)]),
            MoveDown(1).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new(KeyCode::Down)]),
            MoveDown(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Down)]),
            MoveDown(1).into(),
        ),
        // Move one word forward/backward
        (
            KeyEventRegister::n(vec![KeyInput::new('w')]),
            MoveWordForward(1).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('w')]),
            MoveWordForward(1).into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::new('e')]),
            MoveWordForwardToEndOfWord(1).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('e')]),
            MoveWordForwardToEndOfWord(1).into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::new('b')]),
            MoveWordBackward(1).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('b')]),
            MoveWordBackward(1).into(),
        ),
        // Move cursor to start/first/last position
        (
            KeyEventRegister::n(vec![KeyInput::new('0')]),
            MoveToStartOfLine().into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::new('_')]),
            MoveToFirst().into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::shift('$')]),
            MoveToEndOfLine().into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::shift('^')]),
            MoveToFirst().into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('0')]),
            MoveToStartOfLine().into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('_')]),
            MoveToFirst().into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::shift('$')]),
            MoveToEndOfLine().into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::shift('^')]),
            MoveToFirst().into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::ctrl('d')]),
            MoveHalfPageDown().into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::ctrl('d')]),
            MoveHalfPageDown().into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::ctrl('u')]),
            MoveHalfPageUp().into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::ctrl('u')]),
            MoveHalfPageUp().into(),
        ),
        // `Home` and `End` go to first/last position in a line
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Home)]),
            MoveToStartOfLine().into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::new(KeyCode::Home)]),
            MoveToStartOfLine().into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new(KeyCode::Home)]),
            MoveToStartOfLine().into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::End)]),
            MoveToEndOfLine().into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::new(KeyCode::End)]),
            MoveToEndOfLine().into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new(KeyCode::End)]),
            MoveToEndOfLine().into(),
        ),
        // `Ctrl+u` deltes from cursor to first non-whitespace character in insert mode
        (
            KeyEventRegister::i(vec![KeyInput::ctrl('u')]),
            DeleteToFirstCharOfLine.into(),
        ),
        // Move cursor to start/first/last position and enter insert mode
        (
            KeyEventRegister::n(vec![KeyInput::shift('I')]),
            SwitchMode(EditorMode::Insert).chain(MoveToFirst()).into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::shift('A')]),
            SwitchMode(EditorMode::Insert)
                .chain(MoveToEndOfLine())
                .chain(MoveForward(1))
                .into(),
        ),
        // Move cursor to start/last row in the buffer
        (
            KeyEventRegister::n(vec![KeyInput::new('g'), KeyInput::new('g')]),
            MoveToFirstRow().into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('g'), KeyInput::new('g')]),
            MoveToFirstRow().into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::shift('G')]),
            MoveToLastRow().into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::shift('G')]),
            MoveToLastRow().into(),
        ),
        // Move cursor to the next opening/closing bracket.
        (
            KeyEventRegister::n(vec![KeyInput::new('%')]),
            MoveToMatchinBracket().into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('%')]),
            MoveToMatchinBracket().into(),
        ),
        // Append/insert new line and switch into insert mode
        (
            KeyEventRegister::n(vec![KeyInput::new('o')]),
            SwitchMode(EditorMode::Insert)
                .chain(AppendNewline(1))
                .into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::shift('O')]),
            SwitchMode(EditorMode::Insert)
                .chain(InsertNewline(1))
                .into(),
        ),
        // Insert a line break
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Enter)]),
            LineBreak(1).into(),
        ),
        // Remove the current character
        (
            KeyEventRegister::n(vec![KeyInput::new('x')]),
            RemoveChar(1).into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::new(KeyCode::Delete)]),
            RemoveChar(1).into(),
        ),
        // Delete the previous character
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Backspace)]),
            DeleteChar(1).into(),
        ),
        // Delete the next character
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Delete)]),
            DeleteCharForward(1).into(),
        ),
        // Delete the current line
        (
            KeyEventRegister::n(vec![KeyInput::new('d'), KeyInput::new('d')]),
            DeleteLine(1).into(),
        ),
        // Delete from the cursor to the end of the line
        (
            KeyEventRegister::n(vec![KeyInput::shift('D')]),
            DeleteToEndOfLine.into(),
        ),
        // Change to end of line (delete to end, enter insert mode)
        (
            KeyEventRegister::n(vec![KeyInput::shift('C')]),
            SwitchMode(EditorMode::Insert).chain(DeleteToEndOfLine).into(),
        ),
        // Delete the current selection
        (
            KeyEventRegister::v(vec![KeyInput::new('d')]),
            DeleteSelection.chain(SwitchMode(EditorMode::Normal)).into(),
        ),
        // Delete word (from cursor to next word) / delete around word
        (
            KeyEventRegister::n(vec![KeyInput::new('d'), KeyInput::new('w')]),
            ChangeWord(1).into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::new('d'), KeyInput::new('a'), KeyInput::new('w')]),
            ChangeAroundWord.into(),
        ),
        // Join the current line with the line below
        (
            KeyEventRegister::n(vec![KeyInput::shift('J')]),
            JoinLineWithLineBelow.into(),
        ),
        // Select inner word between delimiters
        (
            KeyEventRegister::v(vec![KeyInput::new('i'), KeyInput::new('w')]),
            SelectInnerWord.into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('i'), KeyInput::new('"')]),
            SelectInnerBetween::new('"', '"').into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('i'), KeyInput::new('\'')]),
            SelectInnerBetween::new('\'', '\'').into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('i'), KeyInput::new('(')]),
            SelectInnerBetween::new('(', ')').into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('i'), KeyInput::new(')')]),
            SelectInnerBetween::new('(', ')').into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('i'), KeyInput::new('{')]),
            SelectInnerBetween::new('{', '}').into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('i'), KeyInput::new('}')]),
            SelectInnerBetween::new('{', '}').into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('i'), KeyInput::new('[')]),
            SelectInnerBetween::new('[', ']').into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('i'), KeyInput::new(']')]),
            SelectInnerBetween::new('[', ']').into(),
        ),
        // Delete inner word between delimiters
        (
            KeyEventRegister::n(vec![
                KeyInput::new('d'),
                KeyInput::new('i'),
                KeyInput::new('w'),
            ]),
            ChangeInnerWord.into(),
        ),
        (
            KeyEventRegister::n(vec![
                KeyInput::new('d'),
                KeyInput::new('i'),
                KeyInput::new('"'),
            ]),
            ChangeInnerBetween::new('"', '"').into(),
        ),
        (
            KeyEventRegister::n(vec![
                KeyInput::new('d'),
                KeyInput::new('i'),
                KeyInput::new('\''),
            ]),
            ChangeInnerBetween::new('\'', '\'').into(),
        ),
        (
            KeyEventRegister::n(vec![
                KeyInput::new('d'),
                KeyInput::new('i'),
                KeyInput::new('('),
            ]),
            ChangeInnerBetween::new('(', ')').into(),
        ),
        (
            KeyEventRegister::n(vec![
                KeyInput::new('d'),
                KeyInput::new('i'),
                KeyInput::new(')'),
            ]),
            ChangeInnerBetween::new('(', ')').into(),
        ),
        (
            KeyEventRegister::n(vec![
                KeyInput::new('d'),
                KeyInput::new('i'),
                KeyInput::new('{'),
            ]),
            ChangeInnerBetween::new('{', '}').into(),
        ),
        (
            KeyEventRegister::n(vec![
                KeyInput::new('d'),
                KeyInput::new('i'),
                KeyInput::new('}'),
            ]),
            ChangeInnerBetween::new('{', '}').into(),
        ),
        (
            KeyEventRegister::n(vec![
                KeyInput::new('d'),
                KeyInput::new('i'),
                KeyInput::new('['),
            ]),
            ChangeInnerBetween::new('[', ']').into(),
        ),
        (
            KeyEventRegister::n(vec![
                KeyInput::new('d'),
                KeyInput::new('i'),
                KeyInput::new(']'),
            ]),
            ChangeInnerBetween::new('[', ']').into(),
        ),
        // Change word (from cursor to next word) / change around word
        (
            KeyEventRegister::n(vec![KeyInput::new('c'), KeyInput::new('w')]),
            SwitchMode(EditorMode::Insert).chain(ChangeWord(1)).into(),
        ),
        (
            KeyEventRegister::n(vec![
                KeyInput::new('c'),
                KeyInput::new('a'),
                KeyInput::new('w'),
            ]),
            SwitchMode(EditorMode::Insert).chain(ChangeAroundWord).into(),
        ),
        // Change inner word between delimiters
        (
            KeyEventRegister::n(vec![
                KeyInput::new('c'),
                KeyInput::new('i'),
                KeyInput::new('w'),
            ]),
            SwitchMode(EditorMode::Insert).chain(ChangeInnerWord).into(),
        ),
        (
            KeyEventRegister::n(vec![
                KeyInput::new('c'),
                KeyInput::new('i'),
                KeyInput::new('"'),
            ]),
            SwitchMode(EditorMode::Insert)
                .chain(ChangeInnerBetween::new('"', '"'))
                .into(),
        ),
        (
            KeyEventRegister::n(vec![
                KeyInput::new('c'),
                KeyInput::new('i'),
                KeyInput::new('\''),
            ]),
            SwitchMode(EditorMode::Insert)
                .chain(ChangeInnerBetween::new('\'', '\''))
                .into(),
        ),
        (
            KeyEventRegister::n(vec![
                KeyInput::new('c'),
                KeyInput::new('i'),
                KeyInput::new('('),
            ]),
            SwitchMode(EditorMode::Insert)
                .chain(ChangeInnerBetween::new('(', ')'))
                .into(),
        ),
        (
            KeyEventRegister::n(vec![
                KeyInput::new('c'),
                KeyInput::new('i'),
                KeyInput::new(')'),
            ]),
            SwitchMode(EditorMode::Insert)
                .chain(ChangeInnerBetween::new('(', ')'))
                .into(),
        ),
        (
            KeyEventRegister::n(vec![
                KeyInput::new('c'),
                KeyInput::new('i'),
                KeyInput::new('{'),
            ]),
            SwitchMode(EditorMode::Insert)
                .chain(ChangeInnerBetween::new('{', '}'))
                .into(),
        ),
        (
            KeyEventRegister::n(vec![
                KeyInput::new('c'),
                KeyInput::new('i'),
                KeyInput::new('}'),
            ]),
            SwitchMode(EditorMode::Insert)
                .chain(ChangeInnerBetween::new('{', '}'))
                .into(),
        ),
        (
            KeyEventRegister::n(vec![
                KeyInput::new('c'),
                KeyInput::new('i'),
                KeyInput::new('['),
            ]),
            SwitchMode(EditorMode::Insert)
                .chain(ChangeInnerBetween::new('[', ']'))
                .into(),
        ),
        (
            KeyEventRegister::n(vec![
                KeyInput::new('c'),
                KeyInput::new('i'),
                KeyInput::new(']'),
            ]),
            SwitchMode(EditorMode::Insert)
                .chain(ChangeInnerBetween::new('[', ']'))
                .into(),
        ),
        // Change selection
        (
            KeyEventRegister::v(vec![KeyInput::new('c')]),
            SwitchMode(EditorMode::Insert).chain(ChangeSelection).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('x')]),
            ChangeSelection.chain(SwitchMode(EditorMode::Normal)).into(),
        ),
        // Select  the line
        (
            KeyEventRegister::n(vec![KeyInput::shift('V')]),
            SelectLine.into(),
        ),
        // Undo
        (KeyEventRegister::n(vec![KeyInput::new('u')]), Undo.into()),
        // Redo
        (KeyEventRegister::n(vec![KeyInput::ctrl('r')]), Redo.into()),
        // Copy
        (
            KeyEventRegister::v(vec![KeyInput::new('y')]),
            CopySelection.chain(SwitchMode(EditorMode::Normal)).into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::new('y'), KeyInput::new('y')]),
            CopyLine.into(),
        ),
        // Paste
        (KeyEventRegister::n(vec![KeyInput::new('p')]), Paste.into()),
        (
            KeyEventRegister::v(vec![KeyInput::new('p')]),
            PasteOverSelection
                .chain(SwitchMode(EditorMode::Normal))
                .into(),
        ),
    ]);

    // Open system editor (Ctrl+e in normal mode)
    #[cfg(feature = "system-editor")]
    map.insert(
        KeyEventRegister::n(vec![KeyInput::ctrl('e')]),
        OpenSystemEditor.into(),
    );

    map
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
