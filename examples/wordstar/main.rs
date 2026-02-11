use app::{App, AppContext};
use edtui::{EditorEventHandler, EditorState, Lines};
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
            "EdTUI with WordStar-style keybindings.

Navigation (The Diamond):
- Ctrl+S/D: left/right character
- Ctrl+E/X: up/down line
- Ctrl+A/F: left/right word
- Ctrl+R/C: page up/down

Quick Menu (Ctrl+Q prefix):
- Ctrl+Q, Ctrl+S/D: start/end of line
- Ctrl+Q, Ctrl+E/X: top/bottom of screen
- Ctrl+Q, Ctrl+R/C: beginning/end of file
- Ctrl+Q, Ctrl+Y: delete to end of line
- Ctrl+Q, Ctrl+F: search

Block Operations (Ctrl+K prefix):
- Ctrl+K, Ctrl+B: mark block start (enter selection)
- Ctrl+K, Ctrl+K or Ctrl+K, Ctrl+C: copy block
- Ctrl+K, Ctrl+V: paste
- Ctrl+K, Ctrl+Y: delete block

Editing:
- Ctrl+Y: delete line
- Ctrl+T: delete word right
- Ctrl+H: backspace
- Ctrl+G: delete forward
- Ctrl+U: undo

Quit: Esc
",
        ));
        state.mode = edtui::EditorMode::Insert;

        Self {
            state,
            event_handler: EditorEventHandler::wordstar_mode(),
        }
    }
}
