/// Word Wrap Demo
///
/// This example demonstrates visual-line-aware cursor movement and viewport
/// scrolling when word wrap is enabled.
///
/// Run with: cargo run --example word_wrap_demo
///
/// Controls:
///   - Arrow keys (Up/Down) move by visual (screen) line, not logical paragraph
///   - Mouse wheel scrolls by visual lines
///   - Ctrl+D / Ctrl+U: half-page down/up (visual lines)
///   - Tab: toggle between wrap ON and wrap OFF to compare behavior
///   - Ctrl+C: quit
use edtui::{EditorEventHandler, EditorState, EditorTheme, EditorView, Lines};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Constraint, Layout},
    prelude::*,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Widget},
    DefaultTerminal,
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal) -> Result<(), Box<dyn Error>> {
    let mut app = App::new();
    loop {
        terminal.draw(|frame| frame.render_widget(&mut app, frame.area()))?;

        let event = event::read()?;
        // On Windows, crossterm emits both Press and Release key events.
        // Filter to only handle Press to avoid double inputs.
        if let Event::Key(key) = &event {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
                break;
            }
            if key.code == KeyCode::Tab {
                app.wrap_enabled = !app.wrap_enabled;
                continue;
            }
        }
        app.event_handler.on_event(event, &mut app.state);
    }
    Ok(())
}

struct App {
    state: EditorState,
    event_handler: EditorEventHandler,
    wrap_enabled: bool,
}

impl App {
    fn new() -> Self {
        let content = Lines::from(DEMO_TEXT);
        let mut state = EditorState::new(content);
        state.mode = edtui::EditorMode::Insert;

        Self {
            state,
            event_handler: EditorEventHandler::default(),
            wrap_enabled: true,
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [header, main] =
            Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(area);

        // Header / status bar
        let wrap_status = if self.wrap_enabled { "ON" } else { "OFF" };
        let header_text = format!(
            " Word Wrap Demo  |  Wrap: {} (Tab to toggle)  |  \u{2191}\u{2193}: visual line  |  Ctrl+C: quit ",
            wrap_status
        );
        let header_block = Block::new()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray));
        Paragraph::new(header_text)
            .style(Style::default().bold())
            .block(header_block)
            .render(header, buf);

        // Editor
        let editor_block = Block::bordered()
            .title(format!(" Editor (wrap {}) ", wrap_status))
            .border_style(Style::default().fg(Color::Cyan));

        EditorView::new(&mut self.state)
            .theme(EditorTheme::default().block(editor_block))
            .wrap(self.wrap_enabled)
            .render(main, buf);
    }
}

const DEMO_TEXT: &str = "\
Visual Line Movement Demo

Try pressing Up/Down arrow keys. With wrap ON, the cursor moves one screen line at a time through this long paragraph, instead of jumping to the next logical line. This makes editing long paragraphs in a word processor feel natural and predictable. The cursor preserves its horizontal position across visual lines, just like in any modern text editor.

Short line above, long paragraph below.

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.

Another short line.

Scrolling also works by visual line now. Use Ctrl+D and Ctrl+U to half-page scroll. With wrap ON the viewport scrolls smoothly through wrapped text. The mouse wheel also scrolls by visual lines rather than jumping entire paragraphs.

The quick brown fox jumped over the lazy dog. The quick brown fox jumped over the lazy dog. The quick brown fox jumped over the lazy dog. The quick brown fox jumped over the lazy dog. The quick brown fox jumped over the lazy dog.

Toggle wrap with Tab to compare the old behavior (wrap OFF uses logical lines) versus the new visual-line behavior (wrap ON).

End of demo text.";
