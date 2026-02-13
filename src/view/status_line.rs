use super::theme::{DARK_GRAY, WHITE};
use ratatui_core::layout::{Constraint, HorizontalAlignment, Layout};
use ratatui_core::{buffer::Buffer, layout::Rect, style::Style, text::Span, widgets::Widget};
use ratatui_widgets::block::Block;

/// An optional status line for Editor.
#[derive(Debug, Clone)]
pub struct EditorStatusLine {
    /// Displays the current editor mode in the status line.
    mode: String,
    /// The current search buffer. Shown only in search mode.
    search: Option<String>,
    /// An optional label displayed alongside the mode (e.g. keybinding name).
    label: Option<String>,
    /// The style for the label of the status line
    style_label: Option<Style>,
    /// The style for the mode of the status line
    style_mode: Option<Style>,
    /// The style for the search of the status line
    style_search: Option<Style>,
    /// The style for the line itself
    style_line: Style,
    /// Horizontal alignment of the status bar
    alignment: HorizontalAlignment,
    /// Whether to show the mode text (e.g. Normal/Insert/Visual).
    show_mode: bool,
    /// When true, the status line is only shown in search mode (one row with search only).
    show_only_in_search_mode: bool,
}

impl Default for EditorStatusLine {
    /// Creates a new instance of [`EditorStatusLine`].
    ///
    /// This constructor initializes with default style.
    fn default() -> Self {
        Self {
            mode: String::new(),
            search: None,
            label: None,
            style_label: None,
            style_mode: Some(Style::default().fg(WHITE).bg(DARK_GRAY).bold()),
            style_search: Some(Style::default().fg(WHITE).bg(DARK_GRAY)),
            style_line: Style::default().fg(WHITE).bg(DARK_GRAY),
            alignment: HorizontalAlignment::Left,
            show_mode: true,
            show_only_in_search_mode: false,
        }
    }
}

impl EditorStatusLine {
    /// Overwrite the style for the status lines content.
    ///
    /// This method allows you to customize the appearance of the
    /// status lines content.
    #[deprecated(
        since = "0.10.4",
        note = "Please use `style_mode` and `style_search` instead"
    )]
    #[must_use]
    pub fn style_text(mut self, style: Style) -> Self {
        self.style_mode = Some(style);
        self.style_search = Some(style);
        self
    }

    /// Overwrite the style for the status lines mode.
    ///
    /// This method allows you to customize the appearance of the
    /// status lines mode.
    #[must_use]
    pub fn style_mode(mut self, style: impl Into<Option<Style>>) -> Self {
        self.style_mode = style.into();
        self
    }

    /// Overwrite the style for the status lines search.
    ///
    /// This method allows you to customize the appearance of the
    /// status lines search.
    #[must_use]
    pub fn style_search(mut self, style: impl Into<Option<Style>>) -> Self {
        self.style_search = style.into();
        self
    }

    /// Overwrite the style for the status lines.
    ///
    /// This method allows you to customize the appearance of the
    /// status line.
    #[must_use]
    pub fn style_line(mut self, style: Style) -> Self {
        self.style_line = style;
        self
    }

    /// Overwrite the mode content for the status line.
    ///
    /// This method is used internally to dynamically set the editors mode.
    #[must_use]
    pub fn mode<S: Into<String>>(mut self, mode: S) -> Self {
        self.mode = mode.into();
        self
    }

    /// Overwrite the search content for the status line.
    ///
    /// This method is used internally to dynamically set the editors mode.
    #[must_use]
    pub fn search<S: Into<String>>(mut self, search: Option<S>) -> Self {
        self.search = search.map(Into::into);
        self
    }

    #[deprecated(
        since = "0.10.4",
        note = "Please use `alignment(HorizontalAlignment::Left)` or `alignment(HorizontalAlignment::Right)` instead"
    )]
    pub fn align_left(self, align_left: bool) -> Self {
        let alignment = match align_left {
            true => HorizontalAlignment::Left,
            false => HorizontalAlignment::Right,
        };
        self.alignment(alignment)
    }

    /// Set the alignment for the status line content.
    #[must_use]
    pub fn alignment(mut self, alignment: HorizontalAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Set a label to display in the status line (e.g. keybinding name).
    #[must_use]
    pub fn label<S: Into<String>>(mut self, label: S) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Overwrite the style for the status line label.
    #[must_use]
    pub fn style_label(mut self, style: impl Into<Option<Style>>) -> Self {
        self.style_label = style.into();
        self
    }

    /// Set whether to show the mode text (Normal/Insert/Visual).
    /// When false, the mode is hidden (useful for non-Vim keybindings).
    #[must_use]
    pub fn show_mode(mut self, show: bool) -> Self {
        self.show_mode = show;
        self
    }

    /// When true, the status line row is only reserved and shown when the editor is in search mode.
    /// Use with `show_mode(false)` to show only the search pattern line (e.g. `/pattern`) in the editor pane.
    #[must_use]
    pub fn show_only_in_search_mode(mut self, show: bool) -> Self {
        self.show_only_in_search_mode = show;
        self
    }

    /// Returns whether the status line is only shown in search mode.
    #[must_use]
    pub(crate) fn is_show_only_in_search_mode(&self) -> bool {
        self.show_only_in_search_mode
    }
}

impl Widget for EditorStatusLine {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Build the primary display text: "Label | Mode", "Label", or "Mode"
        let display_text = match (&self.label, self.show_mode) {
            (Some(label), true) if !self.mode.is_empty() => format!(" {} | {} ", label, self.mode),
            (Some(label), _) => format!(" {} ", label),
            (None, true) => format!("{:^10}", self.mode),
            (None, false) => String::new(),
        };

        let display_width = display_text.len() as u16;

        let constraints = match self.alignment {
            HorizontalAlignment::Left => {
                vec![Constraint::Length(display_width), Constraint::Min(1)]
            }
            HorizontalAlignment::Center => vec![
                Constraint::Min(1),
                Constraint::Length(display_width),
                Constraint::Min(1),
            ],
            HorizontalAlignment::Right => {
                vec![Constraint::Min(1), Constraint::Length(display_width)]
            }
        };

        let layout = Layout::horizontal(constraints).split(area);

        let search_text = match self.search {
            None => String::new(),
            Some(search) => format!(" Find: {search}"),
        };

        let display_style = self
            .style_label
            .or(self.style_mode)
            .unwrap_or(self.style_line);
        let display_span = Span::raw(display_text).style(display_style);
        let search_span =
            Span::raw(search_text).style(self.style_search.unwrap_or(self.style_line));

        let line_block = Block::new().style(self.style_line);

        line_block.render(area, buf);

        match self.alignment {
            HorizontalAlignment::Left => {
                display_span.render(layout[0], buf);
                search_span.render(layout[1], buf);
            }
            HorizontalAlignment::Center => {
                display_span.render(layout[1], buf);
                search_span.render(layout[2], buf);
            }
            HorizontalAlignment::Right => {
                display_span.render(layout[1], buf);
                search_span.render(layout[0], buf);
            }
        }
    }
}
