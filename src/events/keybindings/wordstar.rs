use super::super::key::input::KeyCode;
use crate::actions::cpaste::{CopySelection, PasteOverSelection};
use crate::actions::delete::{DeleteCharForward, DeleteLine, DeleteToEndOfLine};
use crate::actions::motion::{MoveHalfPageDown, MoveHalfPageUp, MoveToFirstRow, MoveToLastRow};
use crate::actions::search::StartSearch;
use crate::actions::{
    Action, Chainable, DeleteChar, DeleteSelection, FindNext, FindPrevious, LineBreak,
    MoveBackward, MoveDown, MoveForward, MoveToEndOfLine, MoveToStartOfLine, MoveUp,
    MoveWordBackward, MoveWordForward, MoveWordForwardToEndOfWord, Paste, RemoveCharFromSearch,
    SelectCurrentSearch, StopSearch, SwitchMode, Undo,
};
use crate::events::{KeyEventRegister, KeyInput};
use crate::EditorMode;
use std::collections::HashMap;

use EditorMode::{Insert, Search, Visual};

#[allow(clippy::too_many_lines)]
pub fn key_bindings() -> HashMap<KeyEventRegister, Action> {
    let mut key_map = get_insert_mode_key_bindings();
    key_map.extend(get_visual_mode_key_bindings());
    key_map.extend(get_search_mode_key_bindings());
    key_map
}

/// Register a binding for both lowercase and uppercase Ctrl+key variants.
fn insert_ctrl_both_cases(
    map: &mut HashMap<KeyEventRegister, Action>,
    mode: EditorMode,
    c: char,
    action: Action,
) {
    let lower = c.to_ascii_lowercase();
    let upper = c.to_ascii_uppercase();
    let reg = |ch: char| match mode {
        Insert => KeyEventRegister::i(vec![KeyInput::ctrl(ch)]),
        Visual => KeyEventRegister::v(vec![KeyInput::ctrl(ch)]),
        Search => KeyEventRegister::s(vec![KeyInput::ctrl(ch)]),
        _ => KeyEventRegister::n(vec![KeyInput::ctrl(ch)]),
    };
    map.insert(reg(lower), action.clone());
    if lower != upper {
        map.insert(reg(upper), action);
    }
}

/// Register a two-key Ctrl sequence (e.g. Ctrl+Q then Ctrl+S) for all case
/// combinations of the two characters.
fn insert_ctrl_seq_both_cases(
    map: &mut HashMap<KeyEventRegister, Action>,
    mode: EditorMode,
    c1: char,
    c2: char,
    action: Action,
) {
    let l1 = c1.to_ascii_lowercase();
    let u1 = c1.to_ascii_uppercase();
    let l2 = c2.to_ascii_lowercase();
    let u2 = c2.to_ascii_uppercase();
    let reg = |ch1: char, ch2: char| match mode {
        Insert => KeyEventRegister::i(vec![KeyInput::ctrl(ch1), KeyInput::ctrl(ch2)]),
        Visual => KeyEventRegister::v(vec![KeyInput::ctrl(ch1), KeyInput::ctrl(ch2)]),
        Search => KeyEventRegister::s(vec![KeyInput::ctrl(ch1), KeyInput::ctrl(ch2)]),
        _ => KeyEventRegister::n(vec![KeyInput::ctrl(ch1), KeyInput::ctrl(ch2)]),
    };
    map.insert(reg(l1, l2), action.clone());
    if l1 != u1 {
        map.insert(reg(u1, l2), action.clone());
    }
    if l2 != u2 {
        map.insert(reg(l1, u2), action.clone());
    }
    if l1 != u1 && l2 != u2 {
        map.insert(reg(u1, u2), action);
    }
}

#[allow(clippy::too_many_lines)]
fn get_insert_mode_key_bindings() -> HashMap<KeyEventRegister, Action> {
    let mut map = HashMap::new();

    // =====================================================================
    // Essential WordStar Navigation (The Diamond)
    // =====================================================================

    // Ctrl+S → Move left one character
    insert_ctrl_both_cases(&mut map, Insert, 's', MoveBackward(1).into());
    // Ctrl+D → Move right one character
    insert_ctrl_both_cases(&mut map, Insert, 'd', MoveForward(1).into());
    // Ctrl+E → Move up one line
    insert_ctrl_both_cases(&mut map, Insert, 'e', MoveUp(1).into());
    // Ctrl+X → Move down one line
    insert_ctrl_both_cases(&mut map, Insert, 'x', MoveDown(1).into());
    // Ctrl+A → Move left one word
    insert_ctrl_both_cases(&mut map, Insert, 'a', MoveWordBackward(1).into());
    // Ctrl+F → Move right one word
    insert_ctrl_both_cases(&mut map, Insert, 'f', MoveWordForward(1).into());
    // Ctrl+R → Scroll up a page
    insert_ctrl_both_cases(&mut map, Insert, 'r', MoveHalfPageUp().into());
    // Ctrl+C → Scroll down a page
    insert_ctrl_both_cases(&mut map, Insert, 'c', MoveHalfPageDown().into());

    // =====================================================================
    // Quick Menu Commands (Ctrl+Q then Ctrl+Key)
    // =====================================================================

    // Ctrl+Q, Ctrl+S → Move to start of line
    insert_ctrl_seq_both_cases(&mut map, Insert, 'q', 's', MoveToStartOfLine().into());
    // Ctrl+Q, Ctrl+D → Move to end of line
    insert_ctrl_seq_both_cases(&mut map, Insert, 'q', 'd', MoveToEndOfLine().into());
    // Ctrl+Q, Ctrl+E → Move to top of screen / first row
    insert_ctrl_seq_both_cases(&mut map, Insert, 'q', 'e', MoveToFirstRow().into());
    // Ctrl+Q, Ctrl+X → Move to bottom of screen / last row
    insert_ctrl_seq_both_cases(&mut map, Insert, 'q', 'x', MoveToLastRow().into());
    // Ctrl+Q, Ctrl+R → Move to beginning of file
    insert_ctrl_seq_both_cases(
        &mut map,
        Insert,
        'q',
        'r',
        MoveToFirstRow().chain(MoveToStartOfLine()).into(),
    );
    // Ctrl+Q, Ctrl+C → Move to end of file
    insert_ctrl_seq_both_cases(
        &mut map,
        Insert,
        'q',
        'c',
        MoveToLastRow().chain(MoveToEndOfLine()).into(),
    );
    // Ctrl+Q, Ctrl+Y → Delete to end of line
    insert_ctrl_seq_both_cases(&mut map, Insert, 'q', 'y', DeleteToEndOfLine.into());
    // Ctrl+Q, Ctrl+F → Search
    insert_ctrl_seq_both_cases(
        &mut map,
        Insert,
        'q',
        'f',
        StartSearch.chain(SwitchMode(Search)).into(),
    );

    // =====================================================================
    // Block and File Operations (Ctrl+K then Ctrl+Key)
    // =====================================================================

    // Ctrl+K, Ctrl+B → Mark beginning of block (enter Visual mode)
    insert_ctrl_seq_both_cases(&mut map, Insert, 'k', 'b', SwitchMode(Visual).into());
    // Ctrl+K, Ctrl+V → Paste block
    insert_ctrl_seq_both_cases(&mut map, Insert, 'k', 'v', Paste.into());

    // =====================================================================
    // Editing Commands
    // =====================================================================

    // Ctrl+Y → Delete entire line
    insert_ctrl_both_cases(&mut map, Insert, 'y', DeleteLine(1).into());
    // Ctrl+T → Delete word to the right
    insert_ctrl_both_cases(
        &mut map,
        Insert,
        't',
        SwitchMode(Visual)
            .chain(MoveWordForwardToEndOfWord(1))
            .chain(DeleteSelection)
            .chain(SwitchMode(Insert))
            .into(),
    );
    // Ctrl+H → Backspace (delete character backward)
    insert_ctrl_both_cases(&mut map, Insert, 'h', DeleteChar(1).into());
    // Ctrl+G → Delete character forward
    insert_ctrl_both_cases(&mut map, Insert, 'g', DeleteCharForward(1).into());
    // Ctrl+U → Undo
    insert_ctrl_both_cases(&mut map, Insert, 'u', Undo.into());

    // =====================================================================
    // Standard keys
    // =====================================================================

    map.insert(
        KeyEventRegister::i(vec![KeyInput::new(KeyCode::Backspace)]),
        DeleteChar(1).into(),
    );
    map.insert(
        KeyEventRegister::i(vec![KeyInput::new(KeyCode::Delete)]),
        DeleteCharForward(1).into(),
    );
    map.insert(
        KeyEventRegister::i(vec![KeyInput::new(KeyCode::Enter)]),
        LineBreak(1).into(),
    );

    // Arrow key navigation (fallbacks for the diamond)
    map.insert(
        KeyEventRegister::i(vec![KeyInput::new(KeyCode::Right)]),
        MoveForward(1).into(),
    );
    map.insert(
        KeyEventRegister::i(vec![KeyInput::new(KeyCode::Left)]),
        MoveBackward(1).into(),
    );
    map.insert(
        KeyEventRegister::i(vec![KeyInput::new(KeyCode::Up)]),
        MoveUp(1).into(),
    );
    map.insert(
        KeyEventRegister::i(vec![KeyInput::new(KeyCode::Down)]),
        MoveDown(1).into(),
    );
    map.insert(
        KeyEventRegister::i(vec![KeyInput::new(KeyCode::Home)]),
        MoveToStartOfLine().into(),
    );
    map.insert(
        KeyEventRegister::i(vec![KeyInput::new(KeyCode::End)]),
        MoveToEndOfLine().into(),
    );
    map.insert(
        KeyEventRegister::i(vec![KeyInput::new(KeyCode::PageUp)]),
        MoveHalfPageUp().into(),
    );
    map.insert(
        KeyEventRegister::i(vec![KeyInput::new(KeyCode::PageDown)]),
        MoveHalfPageDown().into(),
    );

    map
}

fn get_visual_mode_key_bindings() -> HashMap<KeyEventRegister, Action> {
    let mut map = HashMap::new();

    // =====================================================================
    // Navigation in Visual mode (The Diamond)
    // =====================================================================

    insert_ctrl_both_cases(&mut map, Visual, 's', MoveBackward(1).into());
    insert_ctrl_both_cases(&mut map, Visual, 'd', MoveForward(1).into());
    insert_ctrl_both_cases(&mut map, Visual, 'e', MoveUp(1).into());
    insert_ctrl_both_cases(&mut map, Visual, 'x', MoveDown(1).into());
    insert_ctrl_both_cases(&mut map, Visual, 'a', MoveWordBackward(1).into());
    insert_ctrl_both_cases(&mut map, Visual, 'f', MoveWordForward(1).chain(MoveBackward(1)).into());
    insert_ctrl_both_cases(&mut map, Visual, 'r', MoveHalfPageUp().into());
    insert_ctrl_both_cases(&mut map, Visual, 'c', MoveHalfPageDown().into());

    // Quick Menu in Visual mode
    insert_ctrl_seq_both_cases(&mut map, Visual, 'q', 's', MoveToStartOfLine().into());
    insert_ctrl_seq_both_cases(&mut map, Visual, 'q', 'd', MoveToEndOfLine().into());
    insert_ctrl_seq_both_cases(&mut map, Visual, 'q', 'e', MoveToFirstRow().into());
    insert_ctrl_seq_both_cases(&mut map, Visual, 'q', 'x', MoveToLastRow().into());
    insert_ctrl_seq_both_cases(
        &mut map,
        Visual,
        'q',
        'r',
        MoveToFirstRow().chain(MoveToStartOfLine()).into(),
    );
    insert_ctrl_seq_both_cases(
        &mut map,
        Visual,
        'q',
        'c',
        MoveToLastRow().chain(MoveToEndOfLine()).into(),
    );

    // =====================================================================
    // Block operations in Visual mode (Ctrl+K prefix)
    // =====================================================================

    // Ctrl+K, Ctrl+K → Copy selection (mark end of block)
    insert_ctrl_seq_both_cases(
        &mut map,
        Visual,
        'k',
        'k',
        CopySelection.chain(SwitchMode(Insert)).into(),
    );
    // Ctrl+K, Ctrl+C → Copy block
    insert_ctrl_seq_both_cases(
        &mut map,
        Visual,
        'k',
        'c',
        CopySelection.chain(SwitchMode(Insert)).into(),
    );
    // Ctrl+K, Ctrl+V → Paste over selection (move block)
    insert_ctrl_seq_both_cases(
        &mut map,
        Visual,
        'k',
        'v',
        PasteOverSelection.chain(SwitchMode(Insert)).into(),
    );
    // Ctrl+K, Ctrl+Y → Delete block
    insert_ctrl_seq_both_cases(
        &mut map,
        Visual,
        'k',
        'y',
        DeleteSelection.chain(SwitchMode(Insert)).into(),
    );

    // =====================================================================
    // Escape / cancel selection
    // =====================================================================

    map.insert(
        KeyEventRegister::v(vec![KeyInput::new(KeyCode::Esc)]),
        SwitchMode(EditorMode::Normal)
            .chain(SwitchMode(Insert))
            .into(),
    );

    // Arrow keys in Visual mode
    map.insert(
        KeyEventRegister::v(vec![KeyInput::new(KeyCode::Right)]),
        MoveForward(1).into(),
    );
    map.insert(
        KeyEventRegister::v(vec![KeyInput::new(KeyCode::Left)]),
        MoveBackward(1).into(),
    );
    map.insert(
        KeyEventRegister::v(vec![KeyInput::new(KeyCode::Up)]),
        MoveUp(1).into(),
    );
    map.insert(
        KeyEventRegister::v(vec![KeyInput::new(KeyCode::Down)]),
        MoveDown(1).into(),
    );
    map.insert(
        KeyEventRegister::v(vec![KeyInput::new(KeyCode::Home)]),
        MoveToStartOfLine().into(),
    );
    map.insert(
        KeyEventRegister::v(vec![KeyInput::new(KeyCode::End)]),
        MoveToEndOfLine().into(),
    );

    map
}

fn get_search_mode_key_bindings() -> HashMap<KeyEventRegister, Action> {
    let mut map = HashMap::new();

    map.insert(
        KeyEventRegister::s(vec![KeyInput::new(KeyCode::Enter)]),
        SelectCurrentSearch.chain(SwitchMode(Insert)).into(),
    );
    map.insert(
        KeyEventRegister::s(vec![KeyInput::new(KeyCode::Esc)]),
        StopSearch.chain(SwitchMode(Insert)).into(),
    );
    map.insert(
        KeyEventRegister::s(vec![KeyInput::new(KeyCode::Backspace)]),
        RemoveCharFromSearch.into(),
    );
    // Ctrl+S in search mode → find next
    insert_ctrl_both_cases(&mut map, Search, 's', FindNext.into());
    // Ctrl+R in search mode → find previous
    insert_ctrl_both_cases(&mut map, Search, 'r', FindPrevious.into());
    // Arrow keys for search navigation
    map.insert(
        KeyEventRegister::s(vec![KeyInput::new(KeyCode::Right)]),
        FindNext.into(),
    );
    map.insert(
        KeyEventRegister::s(vec![KeyInput::new(KeyCode::Left)]),
        FindPrevious.into(),
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

    #[test]
    fn test_diamond_keys_present() {
        let map = key_bindings();
        // Ctrl+S (move left) should be in Insert mode
        let key = KeyEventRegister::i(vec![KeyInput::ctrl('s')]);
        assert!(
            map.contains_key(&key),
            "Ctrl+S should be bound in Insert mode"
        );
        // Ctrl+D (move right) should be in Insert mode
        let key = KeyEventRegister::i(vec![KeyInput::ctrl('d')]);
        assert!(
            map.contains_key(&key),
            "Ctrl+D should be bound in Insert mode"
        );
        // Ctrl+E (move up) should be in Insert mode
        let key = KeyEventRegister::i(vec![KeyInput::ctrl('e')]);
        assert!(
            map.contains_key(&key),
            "Ctrl+E should be bound in Insert mode"
        );
        // Ctrl+X (move down) should be in Insert mode
        let key = KeyEventRegister::i(vec![KeyInput::ctrl('x')]);
        assert!(
            map.contains_key(&key),
            "Ctrl+X should be bound in Insert mode"
        );
    }

    #[test]
    fn test_quick_menu_keys_present() {
        let map = key_bindings();
        // Ctrl+Q, Ctrl+S (start of line)
        let key = KeyEventRegister::i(vec![KeyInput::ctrl('q'), KeyInput::ctrl('s')]);
        assert!(
            map.contains_key(&key),
            "Ctrl+Q Ctrl+S should be bound in Insert mode"
        );
        // Ctrl+Q, Ctrl+R (beginning of file)
        let key = KeyEventRegister::i(vec![KeyInput::ctrl('q'), KeyInput::ctrl('r')]);
        assert!(
            map.contains_key(&key),
            "Ctrl+Q Ctrl+R should be bound in Insert mode"
        );
    }

    #[test]
    fn test_block_operations_present() {
        let map = key_bindings();
        // Ctrl+K, Ctrl+B (mark block start) in Insert mode
        let key = KeyEventRegister::i(vec![KeyInput::ctrl('k'), KeyInput::ctrl('b')]);
        assert!(
            map.contains_key(&key),
            "Ctrl+K Ctrl+B should be bound in Insert mode"
        );
        // Ctrl+K, Ctrl+Y (delete block) in Visual mode
        let key = KeyEventRegister::v(vec![KeyInput::ctrl('k'), KeyInput::ctrl('y')]);
        assert!(
            map.contains_key(&key),
            "Ctrl+K Ctrl+Y should be bound in Visual mode"
        );
    }

    #[test]
    fn test_uppercase_variants_present() {
        let map = key_bindings();
        // Uppercase Ctrl+S
        let key = KeyEventRegister::i(vec![KeyInput::ctrl('S')]);
        assert!(
            map.contains_key(&key),
            "Ctrl+S (uppercase) should be bound in Insert mode"
        );
        // Mixed case Ctrl+Q, Ctrl+S
        let key = KeyEventRegister::i(vec![KeyInput::ctrl('Q'), KeyInput::ctrl('s')]);
        assert!(
            map.contains_key(&key),
            "Ctrl+Q (upper) Ctrl+s (lower) should be bound"
        );
    }
}
