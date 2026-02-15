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

    /// Wraps a line of characters into visual lines that fit within `max_width`.
    /// Uses word-boundary-aware splitting that matches `wrap_spans` so that
    /// cursor movement and rendering agree on where visual lines break.
    pub(crate) fn wrap_line(line: &[char], max_width: usize, tab_width: usize) -> Vec<Vec<char>> {
        if line.is_empty() || max_width == 0 {
            return vec![];
        }

        let mut result = Vec::new();
        let mut start = 0;

        while start < line.len() {
            let segment = &line[start..];

            // Find how many characters fit within max_width.
            let mut width = 0;
            let mut fit_count = 0;
            for &ch in segment {
                let cw = char_width(ch, tab_width);
                if width + cw > max_width {
                    break;
                }
                width += cw;
                fit_count += 1;
            }

            // All remaining characters fit — take them all.
            if fit_count >= segment.len() {
                result.push(segment.to_vec());
                break;
            }

            // Need to split. At least 1 character must be taken.
            if fit_count == 0 {
                result.push(vec![segment[0]]);
                start += 1;
                continue;
            }

            // Try word-boundary split (same logic as get_split_at_word).
            let split_at = Self::get_split_at_word_chars(segment, fit_count);

            result.push(line[start..start + split_at].to_vec());
            start += split_at;
        }

        result
    }

    /// Word-boundary-aware split for a slice of chars.
    /// Given that `char_index` characters fit on the line, adjusts the split
    /// point backward to the nearest whitespace boundary.
    fn get_split_at_word_chars(chars: &[char], char_index: usize) -> usize {
        if char_index == 0 {
            return 0;
        }
        if char_index >= chars.len() {
            return char_index;
        }

        // If the character just before the split is whitespace, split here.
        if chars[char_index - 1].is_ascii_whitespace() {
            return char_index;
        }

        // Search backward for whitespace.
        for j in (0..char_index.saturating_sub(1)).rev() {
            if chars[j].is_ascii_whitespace() {
                // Split after the space (space stays with first part).
                return j + 1;
            }
        }

        // No whitespace found — force-split at the original position.
        char_index
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

    #[test]
    fn test_wrap_line_matches_wrap_spans_word_boundary() {
        // wrap_line and wrap_spans must agree on where lines break so that
        // cursor movement and rendering are in sync.
        let text = "Hello World Test";
        let chars: Vec<char> = text.chars().collect();
        let wrapped_chars = LineWrapper::wrap_line(&chars, 7, 4);

        // wrap_spans produces ["Hello ", "World ", "Test"]
        let spans = vec![Span::raw(text)];
        let wrapped_spans = LineWrapper::wrap_spans(spans, 7, 4);

        // Same number of visual lines.
        assert_eq!(wrapped_chars.len(), wrapped_spans.len());

        // Same character content per visual line.
        let chars_strs: Vec<String> = wrapped_chars
            .iter()
            .map(|v| v.iter().collect::<String>())
            .collect();
        let span_strs: Vec<String> = wrapped_spans
            .iter()
            .map(|line| line.iter().map(|s| s.content.as_ref()).collect::<String>())
            .collect();
        assert_eq!(chars_strs, span_strs);
    }

    #[test]
    fn test_wrap_line_long_word() {
        let chars: Vec<char> = "Supercalifragilisticexpialidocious".chars().collect();
        let wrapped = LineWrapper::wrap_line(&chars, 10, 4);

        assert!(wrapped.len() > 1);
        assert_eq!(
            wrapped[0].iter().collect::<String>(),
            "Supercalif"
        );
    }

    #[test]
    fn test_wrap_line_empty() {
        let wrapped = LineWrapper::wrap_line(&[], 10, 4);
        assert!(wrapped.is_empty());
    }
}
