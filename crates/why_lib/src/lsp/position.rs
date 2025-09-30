use ropey::Rope;
use tower_lsp_server::lsp_types::{Position, Range};

use crate::lexer::Span;

/// Utility for handling position conversions between UTF-8 (used internally)
/// and UTF-16 (required by LSP specification)
#[derive(Debug, Clone)]
pub struct PositionUtils {
    rope: Rope,
}

impl PositionUtils {
    /// Create a new PositionUtils from text content
    pub fn new(text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
        }
    }

    /// Convert a UTF-8 byte offset to an LSP Position (UTF-16 based)
    pub fn offset_to_position(&self, offset: usize) -> Position {
        let line_idx = self.rope.byte_to_line(offset.min(self.rope.len_bytes()));
        let line_start_byte = self.rope.line_to_byte(line_idx);
        let column_byte_offset = offset.saturating_sub(line_start_byte);

        // Get the line content to calculate UTF-16 character position
        let line = self.rope.line(line_idx);
        let line_str = line.to_string();

        // Convert byte offset within line to UTF-16 character offset
        let char_offset = line_str
            .char_indices()
            .take_while(|(byte_idx, _)| *byte_idx < column_byte_offset)
            .map(|(_, ch)| ch.len_utf16())
            .sum::<usize>();

        Position {
            line: line_idx as u32,
            character: char_offset as u32,
        }
    }

    /// Convert an LSP Position (UTF-16 based) to a UTF-8 byte offset
    pub fn position_to_offset(&self, position: Position) -> usize {
        let line_idx = (position.line as usize).min(self.rope.len_lines().saturating_sub(1));
        let line_start_byte = self.rope.line_to_byte(line_idx);

        // Get the line content
        let line = self.rope.line(line_idx);
        let line_str = line.to_string();

        // Convert UTF-16 character offset to UTF-8 byte offset within the line
        let mut utf16_count = 0;
        let mut byte_offset = 0;

        for (byte_idx, ch) in line_str.char_indices() {
            if utf16_count >= position.character as usize {
                break;
            }
            utf16_count += ch.len_utf16();
            byte_offset = byte_idx + ch.len_utf8();
        }

        line_start_byte + byte_offset.min(line_str.len())
    }

    /// Convert a Span to an LSP Range
    pub fn span_to_range(&self, span: &Span) -> Range {
        Range {
            start: Position {
                line: span.start.0 as u32,
                character: span.start.1 as u32,
            },
            end: Position {
                line: span.end.0 as u32,
                character: span.end.1 as u32,
            },
        }
    }

    /// Convert an LSP Range to byte offsets
    pub fn range_to_offsets(&self, range: &Range) -> (usize, usize) {
        (
            self.position_to_offset(range.start),
            self.position_to_offset(range.end),
        )
    }

    /// Get the length of text in bytes
    pub fn len_bytes(&self) -> usize {
        self.rope.len_bytes()
    }

    /// Get the number of lines
    pub fn len_lines(&self) -> usize {
        self.rope.len_lines()
    }

    /// Get a line of text
    pub fn line(&self, line_idx: usize) -> String {
        if line_idx < self.rope.len_lines() {
            self.rope.line(line_idx).to_string()
        } else {
            String::new()
        }
    }

    /// Check if a position is valid within the document
    pub fn is_valid_position(&self, position: Position) -> bool {
        let line_idx = position.line as usize;
        if line_idx >= self.rope.len_lines() {
            return false;
        }

        let line = self.rope.line(line_idx);
        let line_str = line.to_string();
        let max_utf16_chars = line_str.chars().map(|c| c.len_utf16()).sum::<usize>();

        (position.character as usize) <= max_utf16_chars
    }

    /// Get the end position of the document
    pub fn document_end(&self) -> Position {
        if self.rope.len_lines() == 0 {
            return Position { line: 0, character: 0 };
        }

        let last_line_idx = self.rope.len_lines() - 1;
        let last_line = self.rope.line(last_line_idx);
        let last_line_str = last_line.to_string();
        let last_line_utf16_len = last_line_str.chars().map(|c| c.len_utf16()).sum::<usize>();

        Position {
            line: last_line_idx as u32,
            character: last_line_utf16_len as u32,
        }
    }

    /// Create a range that spans the entire document
    pub fn full_document_range(&self) -> Range {
        Range {
            start: Position { line: 0, character: 0 },
            end: self.document_end(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_position_conversion() {
        let text = "hello\nworld\n";
        let utils = PositionUtils::new(text);

        // Test start of document
        assert_eq!(utils.offset_to_position(0), Position { line: 0, character: 0 });

        // Test start of second line
        assert_eq!(utils.offset_to_position(6), Position { line: 1, character: 0 });

        // Test round trip conversion
        let pos = Position { line: 1, character: 3 };
        let offset = utils.position_to_offset(pos);
        assert_eq!(utils.offset_to_position(offset), pos);
    }

    #[test]
    fn test_unicode_handling() {
        let text = "hello 🌍\nworld\n";
        let utils = PositionUtils::new(text);

        // The 🌍 emoji is 4 bytes in UTF-8 but 2 UTF-16 code units
        let pos = Position { line: 0, character: 8 }; // After "hello 🌍"
        let offset = utils.position_to_offset(pos);
        assert_eq!(utils.offset_to_position(offset), pos);
    }

    #[test]
    fn test_document_end() {
        let text = "line1\nline2";
        let utils = PositionUtils::new(text);

        let end = utils.document_end();
        assert_eq!(end.line, 1);
        assert_eq!(end.character, 5); // Length of "line2"
    }

    #[test]
    fn test_empty_document() {
        let utils = PositionUtils::new("");
        assert_eq!(utils.document_end(), Position { line: 0, character: 0 });
        assert_eq!(utils.len_lines(), 1); // Empty document has 1 empty line
    }

    #[test]
    fn test_position_validation() {
        let text = "hello\nworld\n";
        let utils = PositionUtils::new(text);

        // Valid positions
        assert!(utils.is_valid_position(Position { line: 0, character: 0 }));
        assert!(utils.is_valid_position(Position { line: 0, character: 5 }));
        assert!(utils.is_valid_position(Position { line: 1, character: 5 }));

        // Invalid positions
        assert!(!utils.is_valid_position(Position { line: 5, character: 0 })); // Line out of bounds
        assert!(!utils.is_valid_position(Position { line: 0, character: 10 })); // Character out of bounds
    }
}