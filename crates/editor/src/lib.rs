/// Cursor position inside a buffer.
/// row and col are zero-based.
#[derive(Debug, Clone, PartialEq)]
pub struct Cursor {
    pub row: usize,
    pub col: usize,
}

impl Default for Cursor {
    fn default() -> Self {
        Self { row: 0, col: 0 }
    }
}

/// A single open file buffer.
/// Internal representation: Vec<String>, one String per line.
/// All mutations are pure methods - return Self or mutate &mut self.
/// No IO, no side effects.
#[derive(Debug, Clone, PartialEq)]
pub struct Buffer {
    lines: Vec<String>,
    pub cursor: Cursor,
}

impl Buffer {
    /// Create an empty buffer with one empty line and cursor at (0, 0).
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor: Cursor::default(),
        }
    }

    /// Create a buffer from a string, splitting on newlines.
    /// Cursor starts at (0, 0).
    pub fn from_str(content: &str) -> Self {
        let lines: Vec<String> = if content.is_empty() {
            vec![String::new()]
        } else {
            content.lines().map(|l| l.to_string()).collect()
        };
        Self {
            lines,
            cursor: Cursor::default(),
        }
    }

    /// Read-only access to the lines.
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// Current line length (number of chars in the cursor row).
    fn current_line_len(&self) -> usize {
        self.lines[self.cursor.row].len()
    }

    /// Clamp col to the length of the given row.
    fn clamp_col(&mut self, row: usize) {
        let max_col = self.lines[row].len();
        if self.cursor.col > max_col {
            self.cursor.col = max_col;
        }
    }

    // ------------------------------------------------------------------
    // Cursor movement
    // ------------------------------------------------------------------

    /// Move cursor right. Stops at end of line (no wrap).
    pub fn move_right(&mut self) {
        if self.cursor.col < self.current_line_len() {
            self.cursor.col += 1;
        }
    }

    /// Move cursor left. Stops at beginning of line (no wrap).
    pub fn move_left(&mut self) {
        if self.cursor.col > 0 {
            self.cursor.col -= 1;
        }
    }

    /// Move cursor down. Stops at last line. Clamps col to line length.
    pub fn move_down(&mut self) {
        if self.cursor.row + 1 < self.lines.len() {
            self.cursor.row += 1;
            self.clamp_col(self.cursor.row);
        }
    }

    /// Move cursor up. Stops at first line. Clamps col to line length.
    pub fn move_up(&mut self) {
        if self.cursor.row > 0 {
            self.cursor.row -= 1;
            self.clamp_col(self.cursor.row);
        }
    }

    // ------------------------------------------------------------------
    // Editing operations
    // ------------------------------------------------------------------

    /// Insert a character at the current cursor position.
    /// Cursor advances one column after insert.
    pub fn insert_char(&mut self, ch: char) {
        let col = self.cursor.col;
        self.lines[self.cursor.row].insert(col, ch);
        self.cursor.col += 1;
    }

    /// Delete the character immediately before the cursor (backspace).
    /// Does nothing when at the beginning of line (col == 0).
    pub fn delete_char(&mut self) {
        if self.cursor.col == 0 {
            return;
        }
        let col = self.cursor.col;
        self.lines[self.cursor.row].remove(col - 1);
        self.cursor.col -= 1;
    }

    /// Split the current line at the cursor position (Enter / newline).
    /// Text from cursor to end of line moves to a new line below.
    /// Cursor moves to (row + 1, 0).
    pub fn split_line(&mut self) {
        let col = self.cursor.col;
        let row = self.cursor.row;
        let tail = self.lines[row].split_off(col);
        self.lines.insert(row + 1, tail);
        self.cursor.row += 1;
        self.cursor.col = 0;
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests - written BEFORE implementation (TDD)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // 1. test_empty_buffer
    #[test]
    fn test_empty_buffer() {
        let buf = Buffer::new();
        assert_eq!(buf.lines(), &[""]);
        assert_eq!(buf.cursor, Cursor { row: 0, col: 0 });
    }

    // 2. test_from_str
    #[test]
    fn test_from_str() {
        let buf = Buffer::from_str("hello");
        assert_eq!(buf.lines(), &["hello"]);
        assert_eq!(buf.cursor, Cursor { row: 0, col: 0 });
    }

    // 2b. from_str multiline
    #[test]
    fn test_from_str_multiline() {
        let buf = Buffer::from_str("hello\nworld");
        assert_eq!(buf.lines(), &["hello", "world"]);
        assert_eq!(buf.cursor, Cursor { row: 0, col: 0 });
    }

    // 3. test_cursor_move_right
    #[test]
    fn test_cursor_move_right() {
        let mut buf = Buffer::from_str("hello");
        buf.move_right();
        assert_eq!(buf.cursor.col, 1);
    }

    // 4. test_cursor_move_right_eol
    #[test]
    fn test_cursor_move_right_eol() {
        let mut buf = Buffer::from_str("hi");
        buf.cursor.col = 2; // at end of line
        buf.move_right();
        assert_eq!(buf.cursor.col, 2); // no change
    }

    // 5. test_cursor_move_left
    #[test]
    fn test_cursor_move_left() {
        let mut buf = Buffer::from_str("hello");
        buf.cursor.col = 3;
        buf.move_left();
        assert_eq!(buf.cursor.col, 2);
    }

    // 6. test_cursor_move_left_bol
    #[test]
    fn test_cursor_move_left_bol() {
        let mut buf = Buffer::from_str("hello");
        buf.cursor.col = 0;
        buf.move_left();
        assert_eq!(buf.cursor.col, 0); // no change
    }

    // 7. test_cursor_move_down
    #[test]
    fn test_cursor_move_down() {
        let mut buf = Buffer::from_str("hello\nworld");
        buf.move_down();
        assert_eq!(buf.cursor.row, 1);
    }

    // 7b. move_down at last line - no change
    #[test]
    fn test_cursor_move_down_last_line() {
        let mut buf = Buffer::from_str("hello");
        buf.move_down();
        assert_eq!(buf.cursor.row, 0); // no change
    }

    // 8. test_cursor_move_up
    #[test]
    fn test_cursor_move_up() {
        let mut buf = Buffer::from_str("hello\nworld");
        buf.cursor.row = 1;
        buf.move_up();
        assert_eq!(buf.cursor.row, 0);
    }

    // 8b. move_up at first line - no change
    #[test]
    fn test_cursor_move_up_first_line() {
        let mut buf = Buffer::from_str("hello");
        buf.move_up();
        assert_eq!(buf.cursor.row, 0); // no change
    }

    // 9. test_cursor_clamp_col
    #[test]
    fn test_cursor_clamp_col() {
        let mut buf = Buffer::from_str("hello\nhi");
        buf.cursor.col = 4; // valid on "hello" (len=5)
        buf.move_down();    // "hi" has len=2
        assert_eq!(buf.cursor.row, 1);
        assert_eq!(buf.cursor.col, 2); // clamped to len of "hi"
    }

    // 10. test_insert_char at beginning
    #[test]
    fn test_insert_char() {
        let mut buf = Buffer::from_str("hello");
        buf.cursor.col = 0;
        buf.insert_char('x');
        assert_eq!(buf.lines()[0], "xhello");
        assert_eq!(buf.cursor.col, 1);
    }

    // 11. test_insert_char_mid
    #[test]
    fn test_insert_char_mid() {
        let mut buf = Buffer::from_str("hello");
        buf.cursor.col = 2;
        buf.insert_char('x');
        assert_eq!(buf.lines()[0], "hexllo");
        assert_eq!(buf.cursor.col, 3);
    }

    // 12. test_delete_char_backspace
    #[test]
    fn test_delete_char_backspace() {
        let mut buf = Buffer::from_str("hello");
        buf.cursor.col = 1;
        buf.delete_char();
        assert_eq!(buf.lines()[0], "ello");
        assert_eq!(buf.cursor.col, 0);
    }

    // 13. test_delete_at_bol
    #[test]
    fn test_delete_at_bol() {
        let mut buf = Buffer::from_str("hello");
        buf.cursor.col = 0;
        buf.delete_char();
        assert_eq!(buf.lines()[0], "hello"); // unchanged
        assert_eq!(buf.cursor.col, 0);
    }

    // 14. test_line_split_enter
    #[test]
    fn test_line_split_enter() {
        let mut buf = Buffer::from_str("hello");
        buf.cursor.col = 2;
        buf.split_line();
        assert_eq!(buf.lines(), &["he", "llo"]);
        assert_eq!(buf.cursor, Cursor { row: 1, col: 0 });
    }

    // 15. test_line_split_at_eol
    #[test]
    fn test_line_split_at_eol() {
        let mut buf = Buffer::from_str("hello");
        buf.cursor.col = 5; // end of "hello"
        buf.split_line();
        assert_eq!(buf.lines(), &["hello", ""]);
        assert_eq!(buf.cursor, Cursor { row: 1, col: 0 });
    }

    // 15b. split at beginning of line
    #[test]
    fn test_line_split_at_bol() {
        let mut buf = Buffer::from_str("hello");
        buf.cursor.col = 0;
        buf.split_line();
        assert_eq!(buf.lines(), &["", "hello"]);
        assert_eq!(buf.cursor, Cursor { row: 1, col: 0 });
    }

    // 16. test_buffer_lines_accessor
    #[test]
    fn test_buffer_lines_accessor() {
        let buf = Buffer::from_str("abc\ndef");
        let lines = buf.lines();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "abc");
        assert_eq!(lines[1], "def");
    }
}
