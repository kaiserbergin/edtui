use app::{App, AppContext};
use edtui::{EditorState, Lines, MsWordEditorEventHandler};
use std::error::Error;
use term::Term;
mod app;
mod term;
mod theme;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    let mut term = Term::new()?;
    let mut app = App {
        context: AppContext::new(),
        should_quit: false,
    };
    app.run(&mut term)
}

impl AppContext {
    pub fn new() -> Self {
        let mut state = EditorState::new(Lines::from(
            "EdTUI with Windows / MS Word-style keybindings.

Selection (Windows behavior):
- Shift+Arrow: start selection in Insert, extend in Visual
- Arrow without Shift in Visual: clear selection, back to Insert
- Type a character in Visual: replace selection with character

Navigation:
- Arrows: move
- Ctrl+Left/Right: word
- Ctrl+Up/Down: half page
- Ctrl+Alt+Arrows: start/end of line or buffer
- Home/End, PageUp/PageDown

Editing:
- Backspace/Delete: delete char
- Ctrl+Backspace/Delete: delete word
- Ctrl+Z/Y: Undo/Redo
- Ctrl+V: Paste

Search: Ctrl+F to search, Enter to select, Esc to cancel.
Quit: Ctrl+C
",
        ));
        state.mode = edtui::EditorMode::Insert;

        Self {
            state,
            event_handler: MsWordEditorEventHandler::default(),
        }
    }
}
