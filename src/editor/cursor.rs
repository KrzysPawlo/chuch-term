use super::buffer::TextBuffer;

/// Cursor position in the buffer.
/// `col` is a byte offset into the line string (not a character or display index).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Cursor {
    pub row: usize,
    pub col: usize,
}

impl Cursor {
    pub fn new() -> Self {
        Self { row: 0, col: 0 }
    }

    /// Clamp row and col to valid buffer positions.
    pub fn clamp(&mut self, buf: &TextBuffer) {
        (self.row, self.col) = buf.clamp_position(self.row, self.col);
    }

    pub fn move_up(&mut self, buf: &TextBuffer) {
        if self.row > 0 {
            self.row -= 1;
            self.clamp(buf);
        }
    }

    pub fn move_down(&mut self, buf: &TextBuffer) {
        if self.row + 1 < buf.line_count() {
            self.row += 1;
            self.clamp(buf);
        }
    }

    pub fn move_left(&mut self, buf: &TextBuffer) {
        self.clamp(buf);
        if self.col > 0 {
            // Step back one char boundary.
            self.col -= 1;
            let line = buf.line(self.row);
            while self.col > 0 && !line.is_char_boundary(self.col) {
                self.col -= 1;
            }
        } else if self.row > 0 {
            self.row -= 1;
            self.col = buf.line(self.row).len();
        }
    }

    pub fn move_right(&mut self, buf: &TextBuffer) {
        self.clamp(buf);
        let line = buf.line(self.row);
        let col = self.col;
        if col < line.len() {
            // Step forward one char.
            if let Some(ch) = line[col..].chars().next() {
                self.col = col + ch.len_utf8();
            } else {
                self.col = line.len();
            }
        } else if self.row + 1 < buf.line_count() {
            self.row += 1;
            self.col = 0;
        } else {
            self.col = line.len();
        }
    }

    pub fn home(&mut self) {
        self.col = 0;
    }

    pub fn end(&mut self, buf: &TextBuffer) {
        self.clamp(buf);
        self.col = buf.line(self.row).len();
    }

    pub fn page_up(&mut self, buf: &TextBuffer, viewport_height: usize) {
        let rows = viewport_height.max(1);
        self.row = self.row.saturating_sub(rows);
        self.clamp(buf);
    }

    pub fn page_down(&mut self, buf: &TextBuffer, viewport_height: usize) {
        let rows = viewport_height.max(1);
        let max_row = buf.line_count().saturating_sub(1);
        self.row = (self.row + rows).min(max_row);
        self.clamp(buf);
    }
}

// ──────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::buffer::TextBuffer;

    fn buf(lines: &[&str]) -> TextBuffer {
        TextBuffer {
            lines: lines.iter().map(|s| s.to_string()).collect(),
            dirty: false,
            file_path: None,
        }
    }

    #[test]
    fn move_right_wraps_to_next_line() {
        let b = buf(&["hi", "there"]);
        let mut c = Cursor { row: 0, col: 2 };
        c.move_right(&b);
        assert_eq!(c, Cursor { row: 1, col: 0 });
    }

    #[test]
    fn move_left_wraps_to_prev_line() {
        let b = buf(&["hi", "there"]);
        let mut c = Cursor { row: 1, col: 0 };
        c.move_left(&b);
        assert_eq!(c, Cursor { row: 0, col: 2 });
    }

    #[test]
    fn home_and_end() {
        let b = buf(&["hello"]);
        let mut c = Cursor { row: 0, col: 3 };
        c.home();
        assert_eq!(c.col, 0);
        c.end(&b);
        assert_eq!(c.col, 5);
    }

    #[test]
    fn clamp_to_shorter_line() {
        let b = buf(&["hi", "x"]);
        let mut c = Cursor { row: 1, col: 10 };
        c.clamp(&b);
        assert_eq!(c.col, 1);
    }

    #[test]
    fn page_up_clamps_to_zero() {
        let b = buf(&["a", "b", "c"]);
        let mut c = Cursor { row: 1, col: 0 };
        c.page_up(&b, 20);
        assert_eq!(c.row, 0);
    }

    #[test]
    fn page_down_clamps_to_last_line() {
        let b = buf(&["a", "b", "c"]);
        let mut c = Cursor { row: 0, col: 0 };
        c.page_down(&b, 20);
        assert_eq!(c.row, 2);
    }

    #[test]
    fn move_right_clamps_misaligned_utf8_offset() {
        let b = buf(&["zaż"]);
        let mut c = Cursor { row: 0, col: 3 };
        c.move_right(&b);
        assert_eq!(c, Cursor { row: 0, col: 4 });
    }
}
