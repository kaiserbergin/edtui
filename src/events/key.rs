pub(crate) mod deprecated;
pub(crate) mod input;

use crate::actions::{Action, AppendCharToSearch, Execute, InsertChar};
use crate::events::keybindings;
use crate::events::KeyInput;
use crate::{EditorMode, EditorState};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct KeyEventHandler {
    lookup: Vec<KeyInput>,
    register: HashMap<KeyEventRegister, Action>,
    capture_on_insert: bool,
}

impl Default for KeyEventHandler {
    fn default() -> Self {
        Self::vim_mode()
    }
}

impl KeyEventHandler {
    /// Creates a new `KeyEventHandler`.
    #[must_use]
    pub fn new(register: HashMap<KeyEventRegister, Action>, capture_on_insert: bool) -> Self {
        Self {
            lookup: Vec::new(),
            register,
            capture_on_insert,
        }
    }

    /// Creates a new `KeyEventHandler` with vim keybindings.
    #[must_use]
    pub fn vim_mode() -> Self {
        Self::new(keybindings::vim::key_bindings(), false)
    }

    /// Creates a new `KeyEventHandler` with emacs keybindings.
    #[must_use]
    pub fn emacs_mode() -> Self {
        Self::new(keybindings::emacs::key_bindings(), true)
    }

    /// Creates a new `KeyEventHandler` with MS Word-style keybindings.
    #[must_use]
    pub fn ms_word_mode() -> Self {
        Self::new(keybindings::ms_word::key_bindings(), true)
    }

    /// Creates a new `KeyEventHandler` with WordStar-style keybindings.
    #[must_use]
    pub fn wordstar_mode() -> Self {
        Self::new(keybindings::wordstar::key_bindings(), true)
    }

    /// Insert a new callback to the registry
    pub fn insert<T>(&mut self, key: KeyEventRegister, action: T)
    where
        T: Into<Action>,
    {
        self.register.insert(key, action.into());
    }

    /// Extents the register with the contents of an iterator
    pub fn extend<T, U>(&mut self, iter: T)
    where
        U: Into<Action>,
        T: IntoIterator<Item = (KeyEventRegister, U)>,
    {
        self.register
            .extend(iter.into_iter().map(|(k, v)| (k, v.into())));
    }

    /// Remove a callback from the registry
    pub fn remove(&mut self, key: &KeyEventRegister) {
        self.register.remove(key);
    }

    /// Returns an action for a specific register key, if present.
    /// Returns an action only if there is an exact match. If there
    /// are multiple matches or an inexact match, the specified key
    /// is appended to the lookup vector.
    /// If there is an exact match or if none of the keys in the registry
    /// starts with the current sequence, the lookup sequence is reset.
    /// Supports numeric prefixes (e.g. 26w, d15w, c100w) by stripping
    /// a contiguous digit sequence and looking up the base binding.
    #[must_use]
    fn get(&mut self, c: KeyInput, mode: EditorMode) -> Option<Action> {
        self.lookup.push(c);
        let lookup = &self.lookup;
        let mode = mode;

        // Try to parse as (prefix)(digits)(suffix) for countable bindings
        for i in 0..lookup.len() {
            for j in (i + 1..lookup.len()).rev() {
                let digit_slice = &lookup[i..j];
                if digit_slice.is_empty() || !digit_slice.iter().all(key_input_is_digit) {
                    continue;
                }
                let suffix = &lookup[j..];
                if suffix.is_empty() {
                    continue;
                }
                // Don't treat a suffix that starts with a digit as the command (e.g. "10" should
                // wait for "w"/"j" rather than matching "0" = start of line).
                if suffix.first().map_or(false, key_input_is_digit) {
                    continue;
                }
                let base_keys: Vec<KeyInput> = lookup[0..i]
                    .iter()
                    .chain(suffix.iter())
                    .cloned()
                    .collect();
                let base_key = KeyEventRegister::new(base_keys, mode);
                if let Some(action) = self.register.get(&base_key) {
                    let n = parse_digit_prefix(digit_slice);
                    if n > 0 {
                        self.lookup.clear();
                        return Some(action.clone().with_count(n));
                    }
                }
            }
        }

        // Incomplete count: all digits, or [d]/[c] followed by only digits — wait for more input
        if lookup.iter().all(key_input_is_digit) {
            return None;
        }
        if lookup.len() >= 2 {
            let (prefix, rest) = lookup.split_at(1);
            if (prefix[0].key == input::KeyCode::Char('d')
                || prefix[0].key == input::KeyCode::Char('c'))
                && rest.iter().all(key_input_is_digit)
            {
                return None;
            }
        }

        let key = KeyEventRegister::new(self.lookup.clone(), mode);
        match self
            .register
            .keys()
            .filter(|k| k.mode == key.mode && k.keys.starts_with(&key.keys))
            .count()
        {
            0 => {
                self.lookup.clear();
                None
            }
            1 => self.register.get(&key).map(|action| {
                self.lookup.clear();
                action.clone()
            }),
            _ => None,
        }
    }

    /// Returns true if any insert-mode binding in the register has a key sequence
    /// that starts with the given key (used to decide whether to buffer in insert mode).
    fn has_insert_sequence_starting_with(&self, key: &KeyInput) -> bool {
        self.register.keys().any(|k| {
            k.mode == EditorMode::Insert && !k.keys.is_empty() && k.keys[0] == *key
        })
    }

    /// Inserts all buffered keys in the lookup as characters and clears the lookup.
    /// Used when an insert-mode sequence does not match or is cancelled (e.g. by Esc).
    fn flush_insert_lookup(&mut self, state: &mut EditorState) {
        for key in self.lookup.drain(..) {
            if let input::KeyCode::Char(c) = key.key {
                if self.capture_on_insert {
                    state.capture();
                }
                InsertChar(c).execute(state);
            } else if matches!(key.key, input::KeyCode::Tab) {
                if self.capture_on_insert {
                    state.capture();
                }
                InsertChar('\t').execute(state);
            }
        }
    }
}

fn key_input_is_digit(k: &KeyInput) -> bool {
    matches!(k.key, input::KeyCode::Char(c) if c.is_ascii_digit())
}

fn parse_digit_prefix(digits: &[KeyInput]) -> usize {
    let s: String = digits
        .iter()
        .filter_map(|k| {
            if let input::KeyCode::Char(c) = k.key {
                Some(c)
            } else {
                None
            }
        })
        .collect();
    s.parse::<usize>().unwrap_or(0).max(1)
}


#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct KeyInputSequence(Vec<KeyInput>);

impl KeyInputSequence {
    pub fn new(keys: Vec<KeyInput>) -> Self {
        KeyInputSequence(keys)
    }
}

impl From<Vec<KeyInput>> for KeyInputSequence {
    fn from(keys: Vec<KeyInput>) -> Self {
        KeyInputSequence(keys)
    }
}

#[allow(deprecated)]
impl From<Vec<deprecated::KeyEvent>> for KeyInputSequence {
    fn from(events: Vec<deprecated::KeyEvent>) -> Self {
        KeyInputSequence(events.into_iter().map(|event| event.into()).collect())
    }
}

impl From<KeyInputSequence> for Vec<KeyInput> {
    fn from(seq: KeyInputSequence) -> Self {
        seq.0
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct KeyEventRegister {
    keys: Vec<KeyInput>,
    mode: EditorMode,
}

type RegisterCB = fn(&mut EditorState);

#[derive(Clone, Debug)]
struct RegisterVal(pub fn(&mut EditorState));

impl KeyEventRegister {
    pub fn new<T>(key: T, mode: EditorMode) -> Self
    where
        T: Into<KeyInputSequence>,
    {
        Self {
            keys: key.into().into(),
            mode,
        }
    }

    pub fn n<T>(key: T) -> Self
    where
        T: Into<KeyInputSequence>,
    {
        Self::new(key, EditorMode::Normal)
    }

    pub fn v<T>(key: T) -> Self
    where
        T: Into<KeyInputSequence>,
    {
        Self::new(key, EditorMode::Visual)
    }

    pub fn i<T>(key: T) -> Self
    where
        T: Into<KeyInputSequence>,
    {
        Self::new(key, EditorMode::Insert)
    }

    pub fn s<T>(key: T) -> Self
    where
        T: Into<KeyInputSequence>,
    {
        Self::new(key, EditorMode::Search)
    }
}

impl KeyEventHandler {
    pub(crate) fn on_event<T>(&mut self, key: T, state: &mut EditorState)
    where
        T: Into<KeyInput> + Copy + std::fmt::Debug,
    {
        let mode = state.mode;
        let key_input = key.into();

        // Insert mode: support key sequences (e.g. jj -> Normal) and flush on non-char keys
        if mode == EditorMode::Insert {
            let is_char_or_tab = matches!(key_input.key, input::KeyCode::Char(_) | input::KeyCode::Tab)
                && (key_input.modifiers == input::Modifiers::NONE
                    || key_input.modifiers == input::Modifiers::SHIFT);

            if !self.lookup.is_empty() && !is_char_or_tab {
                self.flush_insert_lookup(state);
            }

            if let input::KeyCode::Char(c) = key_input.key {
                if key_input.modifiers == input::Modifiers::NONE
                    || key_input.modifiers == input::Modifiers::SHIFT
                {
                    if self.has_insert_sequence_starting_with(&key_input) || !self.lookup.is_empty() {
                        let saved = self.lookup.clone();
                        if let Some(mut action) = self.get(key_input, EditorMode::Insert) {
                            action.execute(state);
                            return;
                        }
                        // get() returned None: either no match (lookup cleared) or partial match (still waiting)
                        if self.lookup.is_empty() {
                            // No match - insert buffered keys and current character
                            for k in &saved {
                                if let input::KeyCode::Char(ch) = k.key {
                                    if self.capture_on_insert {
                                        state.capture();
                                    }
                                    InsertChar(ch).execute(state);
                                }
                            }
                            if self.capture_on_insert {
                                state.capture();
                            }
                            InsertChar(c).execute(state);
                        }
                        // else: partial match (e.g. first 'j' of "jj") - don't insert, wait for next key
                        return;
                    }
                    if self.capture_on_insert {
                        state.capture();
                    }
                    InsertChar(c).execute(state);
                    return;
                }
            }

            if matches!(key_input.key, input::KeyCode::Tab)
                && key_input.modifiers == input::Modifiers::NONE
            {
                if self.capture_on_insert {
                    state.capture();
                }
                InsertChar('\t').execute(state);
                return;
            }
        }

        // Always add characters to search in search mode
        if mode == EditorMode::Search {
            if let input::KeyCode::Char(c) = key_input.key {
                if key_input.modifiers == input::Modifiers::NONE {
                    AppendCharToSearch(c).execute(state);
                    return;
                }
            }
        }

        // Else lookup an action from the register
        if let Some(mut action) = self.get(key_input, mode) {
            action.execute(state);
        }
    }
}

#[cfg(test)]
mod tests {
    #[allow(deprecated)]
    use super::deprecated::KeyEvent;
    use super::input::KeyCode;
    use super::*;

    #[test]
    #[allow(deprecated)]
    fn test_key_event_register_with_key_event() {
        let register = KeyEventRegister::n(vec![KeyEvent::Ctrl('a'), KeyEvent::Char('b')]);
        assert_eq!(register.mode, EditorMode::Normal);
        assert_eq!(register.keys.len(), 2);

        assert_eq!(register.keys[0], KeyInput::ctrl('a'));
        assert_eq!(register.keys[1], KeyInput::new('b'));
    }

    #[test]
    fn test_key_event_register_with_key_input() {
        let register = KeyEventRegister::i(vec![KeyInput::ctrl('a'), KeyInput::new('b')]);
        assert_eq!(register.mode, EditorMode::Insert);
        assert_eq!(register.keys.len(), 2);

        assert_eq!(register.keys[0], KeyInput::ctrl('a'));
        assert_eq!(register.keys[1], KeyInput::new('b'));
    }

    #[test]
    fn test_key_event_register_with_crossterm() {
        use crossterm::event::{KeyCode as CTKeyCode, KeyEvent as CTKeyEvent, KeyModifiers};

        let ct_key_event = CTKeyEvent::new(CTKeyCode::Char('a'), KeyModifiers::CONTROL);
        let key_input: KeyInput = ct_key_event.into();

        let register = KeyEventRegister::v(vec![key_input, KeyInput::new(CTKeyCode::Enter)]);
        assert_eq!(register.mode, EditorMode::Visual);
        assert_eq!(register.keys.len(), 2);

        assert_eq!(register.keys[0], KeyInput::ctrl('a'));
        assert_eq!(register.keys[1], KeyInput::new(CTKeyCode::Enter));
    }

    #[test]
    fn test_insert_hello_world() {
        use crate::EditorState;

        let mut state = EditorState::default();
        state.mode = EditorMode::Insert;

        let mut handler = KeyEventHandler::default();

        let inputs = vec![
            KeyInput::shift('H'),
            KeyInput::new('e'),
            KeyInput::new('l'),
            KeyInput::new('l'),
            KeyInput::new('o'),
            KeyInput::new(' '),
            KeyInput::shift('W'),
            KeyInput::new('o'),
            KeyInput::new('r'),
            KeyInput::new('l'),
            KeyInput::new('d'),
            KeyInput::shift('!'),
            KeyInput::new(KeyCode::Enter),
            KeyInput::shift('H'),
            KeyInput::new('i'),
            KeyInput::shift('!'),
        ];

        for input in inputs {
            handler.on_event(input, &mut state);
        }

        assert_eq!(state.lines.to_string(), String::from("Hello World!\nHi!"));
    }

    #[test]
    fn test_vim_emacs_ms_word_wordstar_mode_construct_and_handle_key() {
        use crate::EditorState;

        let mut state = EditorState::default();
        state.mode = crate::EditorMode::Insert;

        let mut vim = KeyEventHandler::vim_mode();
        vim.on_event(KeyInput::new('a'), &mut state);
        assert!(state.lines.to_string().len() >= 1);

        let mut state2 = EditorState::default();
        state2.mode = crate::EditorMode::Insert;
        let mut emacs = KeyEventHandler::emacs_mode();
        emacs.on_event(KeyInput::new(KeyCode::Right), &mut state2);

        let mut state3 = EditorState::default();
        state3.mode = crate::EditorMode::Insert;
        let mut ms_word = KeyEventHandler::ms_word_mode();
        ms_word.on_event(KeyInput::new(KeyCode::Down), &mut state3);

        let mut state4 = EditorState::default();
        state4.mode = crate::EditorMode::Insert;
        let mut wordstar = KeyEventHandler::wordstar_mode();
        wordstar.on_event(KeyInput::new(KeyCode::Right), &mut state4);
    }

    // --- jj insert-mode escape and $ / ^ bindings ---

    #[test]
    fn test_vim_jj_in_insert_switches_to_normal_without_inserting() {
        use crate::{EditorState, Index2, Lines};

        let mut state = EditorState::new(Lines::from("ab"));
        state.mode = EditorMode::Insert;
        state.cursor = Index2::new(0, 2);

        let mut handler = KeyEventHandler::vim_mode();
        handler.on_event(KeyInput::new('j'), &mut state);
        assert_eq!(state.mode, EditorMode::Insert);
        assert_eq!(state.lines.to_string(), "ab");

        handler.on_event(KeyInput::new('j'), &mut state);
        assert_eq!(state.mode, EditorMode::Normal);
        assert_eq!(state.lines.to_string(), "ab");
    }

    #[test]
    fn test_vim_j_then_other_char_in_insert_inserts_both() {
        use crate::EditorState;

        let mut state = EditorState::default();
        state.mode = EditorMode::Insert;

        let mut handler = KeyEventHandler::vim_mode();
        handler.on_event(KeyInput::new('j'), &mut state);
        handler.on_event(KeyInput::new('k'), &mut state);

        assert_eq!(state.lines.to_string(), "jk");
        assert_eq!(state.mode, EditorMode::Insert);
    }

    #[test]
    fn test_vim_j_then_esc_in_insert_inserts_j_then_normal() {
        use crate::EditorState;

        let mut state = EditorState::default();
        state.mode = EditorMode::Insert;

        let mut handler = KeyEventHandler::vim_mode();
        handler.on_event(KeyInput::new('j'), &mut state);
        handler.on_event(KeyInput::new(KeyCode::Esc), &mut state);

        assert_eq!(state.lines.to_string(), "j");
        assert_eq!(state.mode, EditorMode::Normal);
    }

    #[test]
    fn test_vim_single_j_then_space_in_insert_inserts_j_space() {
        use crate::EditorState;

        let mut state = EditorState::default();
        state.mode = EditorMode::Insert;

        let mut handler = KeyEventHandler::vim_mode();
        handler.on_event(KeyInput::new('j'), &mut state);
        handler.on_event(KeyInput::new(' '), &mut state);

        assert_eq!(state.lines.to_string(), "j ");
        assert_eq!(state.mode, EditorMode::Insert);
    }

    #[test]
    fn test_vim_dollar_normal_mode_moves_to_end_of_line() {
        use crate::{EditorState, Index2, Lines};

        let mut state = EditorState::new(Lines::from("abc"));
        state.mode = EditorMode::Normal;
        state.cursor = Index2::new(0, 0);

        let mut handler = KeyEventHandler::vim_mode();
        handler.on_event(KeyInput::shift('$'), &mut state);

        assert_eq!(state.cursor, Index2::new(0, 2));
        assert_eq!(state.mode, EditorMode::Normal);
    }

    #[test]
    fn test_vim_caret_normal_mode_moves_to_first_non_whitespace() {
        use crate::{EditorState, Index2, Lines};

        let mut state = EditorState::new(Lines::from("  xy"));
        state.mode = EditorMode::Normal;
        state.cursor = Index2::new(0, 3);

        let mut handler = KeyEventHandler::vim_mode();
        handler.on_event(KeyInput::shift('^'), &mut state);

        assert_eq!(state.cursor, Index2::new(0, 2));
        assert_eq!(state.mode, EditorMode::Normal);
    }

    #[test]
    fn test_vim_dollar_visual_mode_moves_to_end_of_line() {
        use crate::{EditorState, Index2, Lines};

        let mut state = EditorState::new(Lines::from("hello"));
        state.mode = EditorMode::Visual;
        state.cursor = Index2::new(0, 1);

        let mut handler = KeyEventHandler::vim_mode();
        handler.on_event(KeyInput::shift('$'), &mut state);

        // Visual mode uses insert-style max_col: end of "hello" is col 5 (one past last char)
        assert_eq!(state.cursor, Index2::new(0, 5));
    }

    #[test]
    fn test_vim_caret_visual_mode_moves_to_first_non_whitespace() {
        use crate::{EditorState, Index2, Lines};

        let mut state = EditorState::new(Lines::from("   ab"));
        state.mode = EditorMode::Visual;
        state.cursor = Index2::new(0, 4);

        let mut handler = KeyEventHandler::vim_mode();
        handler.on_event(KeyInput::shift('^'), &mut state);

        assert_eq!(state.cursor, Index2::new(0, 3));
    }
}
