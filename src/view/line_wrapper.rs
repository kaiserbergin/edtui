use crate::helper::{char_width, span_width, split_str_at};
use ratatui_core::text::Span;
use std::borrow::Cow;

#[derive(Default)]
pub(crate) struct LineWrapper;

impl LineWrapper {
    /// Splits a given line width into multiple smaller widths, ensuring each width
    /// is no larger than the specified maximum width.
    pub(crate) fn determine_split(line_width: usize, max_width: usize) -> Vec<usize> {
        if line_width == 0 {
            return vec![0];
        }

        let mut remaining_width = line_width;
        let mut split_widths = Vec::new();

        while remaining_width > 0 {
            let current_chunk = std::cmp::min(remaining_width, max_width);
            split_widths.push(current_chunk);
            remaining_width = remaining_width.saturating_sub(max_width);
        }

        split_widths
    }

    pub(crate) fn wrap_line(line: &[char], max_width: usize, tab_width: usize) -> Vec<Vec<char>> {
        let mut lines = Vec::new();
        let mut line_width = 0;
        let mut current_line = Vec::new();

        for &ch in line {
            let char_width = char_width(ch, tab_width);

            if line_width + char_width > max_width {
                lines.push(current_line.clone());
                current_line.clear();
                line_width = 0;
            }

            current_line.push(ch);
            line_width += char_width;
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        lines
    }

    pub(crate) fn wrap_spans(
        spans: Vec<Span<'_>>,
        max_width: usize,
        tab_width: usize,
    ) -> Vec<Vec<Span<'_>>> {
        let mut wrapped_lines = Vec::new();
        let mut current_line = Vec::new();
        let mut current_line_width = 0;

        for span in spans {
            // If adding this span exceeds the max width, handle wrapping
            if current_line_width + span_width(&span, tab_width) > max_width {
                let mut remaining_span = span.clone();
                let mut split_at = max_width - current_line_width;

                while span_width(&remaining_span, tab_width) > split_at {
                    let (fitting_part, rest) =
                        Self::split_span_at(remaining_span, split_at, tab_width);
                    current_line.push(fitting_part.clone());
                    wrapped_lines.push(current_line.clone());

                    // Prepare for the next line
                    current_line.clear();
                    remaining_span = rest;
                    split_at = max_width;
                }

                // Add remaining part to the current line
                current_line_width = span_width(&remaining_span, tab_width);
                current_line.push(remaining_span);
            } else {
                // No wrapping needed, just add the span
                current_line_width += span_width(&span, tab_width);
                current_line.push(span);
            }
        }

        // Add any remaining content as the last line
        if !current_line.is_empty() {
            wrapped_lines.push(current_line);
        }

        wrapped_lines
    }

    /// Adjusts the split index to split at word boundaries when possible.
    /// If the character at the split point is not whitespace, searches backward
    /// for the nearest whitespace character. Returns the adjusted split index
    /// (position after the space), or the original index if no whitespace is found.
    fn get_split_at_word(s: &str, char_index: usize) -> usize {
        if char_index == 0 {
            return 0;
        }

        let chars: Vec<char> = s.chars().collect();
        if char_index >= chars.len() {
            return char_index;
        }

        // Check if the character just before the split is whitespace
        let last_fitting_char = chars[char_index - 1];
        if last_fitting_char.is_ascii_whitespace() {
            return char_index;
        }

        // Search backward for whitespace
        for j in (0..char_index - 1).rev() {
            if chars[j].is_ascii_whitespace() {
                // Split after the space (keep space with first part)
                return j + 1;
            }
        }

        // No whitespace found, word is too long - split at original position
        char_index
    }

    fn split_str_at(s: Cow<'_, str>, split_at: usize, tab_width: usize) -> (String, String) {
        let mut current_width = 0;
        for (i, ch) in s.chars().enumerate() {
            current_width += char_width(ch, tab_width);
            if current_width > split_at {
                let adjusted_i = Self::get_split_at_word(s.as_ref(), i);
                let (a, b) = split_str_at(s, adjusted_i);
                return (a.to_string(), b.to_string());
            }
        }

        (s.to_string(), String::new())
    }

    fn split_span_at(span: Span, split_at: usize, tab_width: usize) -> (Span, Span) {
        let (a, b) = Self::split_str_at(span.content, split_at, tab_width);
        (Span::styled(a, span.style), Span::styled(b, span.style))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_spans() {
        let spans = vec![Span::raw("Hello"), Span::raw("World")];
        let wrapped_spans = LineWrapper::wrap_spans(spans, 3, 0);

        assert_eq!(wrapped_spans[0], vec![Span::raw("Hel")]);
        assert_eq!(wrapped_spans[1], vec![Span::raw("lo"), Span::raw("W")]);
        assert_eq!(wrapped_spans[2], vec![Span::raw("orl")]);
        assert_eq!(wrapped_spans[3], vec![Span::raw("d")]);
    }

    #[test]
    fn test_wrap_spans_with_emoji() {
        let spans = vec![Span::raw("Hell🙂!")];
        let wrapped_spans = LineWrapper::wrap_spans(spans, 4, 0);

        assert_eq!(wrapped_spans[0], vec![Span::raw("Hell")]);
        assert_eq!(wrapped_spans[1], vec![Span::raw("🙂!")]);
    }

    #[test]
    fn test_split_span_at_with_emoji() {
        let span = Span::raw("🙂!");
        let (left, right) = LineWrapper::split_span_at(span, 2, 0);

        assert_eq!(left, Span::raw("🙂"));
        assert_eq!(right, Span::raw("!"));
    }

    #[test]
    fn test_line_wrapper_determine_split() {
        let line_widths = LineWrapper::determine_split(5, 3);

        assert_eq!(line_widths[0], 3);
        assert_eq!(line_widths[1], 2);

        let line_widths = LineWrapper::determine_split(6, 3);

        assert_eq!(line_widths[0], 3);
        assert_eq!(line_widths[1], 3);
    }

    #[test]
    fn test_get_split_at_word_basic() {
        // "Hello World" - splitting in middle of "World" should adjust to after "Hello "
        let text = "Hello World";
        let split_at = 8; // In middle of "World"
        let adjusted = LineWrapper::get_split_at_word(text, split_at);
        assert_eq!(adjusted, 6); // After "Hello " (space included)
        
        let (first, second) = (text.chars().take(adjusted).collect::<String>(),
                               text.chars().skip(adjusted).collect::<String>());
        assert_eq!(first, "Hello ");
        assert_eq!(second, "World");
    }

    #[test]
    fn test_get_split_at_word_no_space() {
        // "HelloWorld" - no space, should return original index
        let text = "HelloWorld";
        let split_at = 7;
        let adjusted = LineWrapper::get_split_at_word(text, split_at);
        assert_eq!(adjusted, 7); // No adjustment, word too long
    }

    #[test]
    fn test_get_split_at_word_at_space() {
        // "Hello World" - split exactly after space
        let text = "Hello World";
        let split_at = 6; // Right after "Hello "
        let adjusted = LineWrapper::get_split_at_word(text, split_at);
        assert_eq!(adjusted, 6); // No change needed, already at good position
    }

    #[test]
    fn test_get_split_at_word_edge_cases() {
        // Empty string
        let text = "";
        let adjusted = LineWrapper::get_split_at_word(text, 0);
        assert_eq!(adjusted, 0);

        // Single character
        let text = "A";
        let adjusted = LineWrapper::get_split_at_word(text, 1);
        assert_eq!(adjusted, 1);

        // Split at position 0
        let text = "Hello";
        let adjusted = LineWrapper::get_split_at_word(text, 0);
        assert_eq!(adjusted, 0);

        // Multiple spaces
        let text = "Hello   World";
        let split_at = 10; // In middle of "World"
        let adjusted = LineWrapper::get_split_at_word(text, split_at);
        assert_eq!(adjusted, 8); // After last space
    }

    #[test]
    fn test_wrap_spans_with_words() {
        // Test word-aware wrapping with real spans
        let spans = vec![Span::raw("Hello World Test")];
        let wrapped_spans = LineWrapper::wrap_spans(spans, 7, 4);

        // Should split as "Hello " (width 6) and "World " (width 6) and "Test" (width 4)
        // With word awareness, "Hello " should be on first line
        assert_eq!(wrapped_spans[0][0].content, "Hello ");
        assert_eq!(wrapped_spans[1][0].content, "World ");
        assert_eq!(wrapped_spans[2][0].content, "Test");
    }

    #[test]
    fn test_wrap_spans_long_word() {
        // Test that long words still get split when necessary
        let spans = vec![Span::raw("Supercalifragilisticexpialidocious")];
        let wrapped_spans = LineWrapper::wrap_spans(spans, 10, 4);

        // Should force-split the long word
        assert!(wrapped_spans.len() > 1);
        assert_eq!(wrapped_spans[0][0].content, "Supercalif");
    }
}
