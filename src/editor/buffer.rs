use std::path::{Path, PathBuf};
use anyhow::{Context, Result};

/// Text buffer: stores the document as a Vec of lines (without trailing newlines).
pub struct TextBuffer {
    pub lines: Vec<String>,
    pub dirty: bool,
    pub file_path: Option<PathBuf>,
}

impl TextBuffer {
    /// Create an empty buffer (single empty line, no file path).
    pub fn new_empty() -> Self {
        Self {
            lines: vec![String::new()],
            dirty: false,
            file_path: None,
        }
    }

    /// Load a file into a buffer.
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Cannot open file: {}", path.display()))?;
        let mut lines: Vec<String> = content.split('\n').map(|s| s.to_string()).collect();
        // Remove the trailing empty line that comes from a file ending with \n.
        if lines.last().map(|l| l.is_empty()).unwrap_or(false) && lines.len() > 1 {
            lines.pop();
        }
        if lines.is_empty() {
            lines.push(String::new());
        }
        Ok(Self {
            lines,
            dirty: false,
            file_path: Some(path.to_path_buf()),
        })
    }

    /// Number of lines in the buffer.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Reference to a line by index. Panics if out of bounds — callers must clamp.
    pub fn line(&self, row: usize) -> &str {
        &self.lines[row]
    }

    /// Display name for the status bar.
    pub fn display_name(&self) -> String {
        match &self.file_path {
            Some(p) => p
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| p.to_string_lossy().to_string()),
            None => "[New File]".to_string(),
        }
    }

    /// Return the entire buffer as a single string without forcing a trailing newline.
    pub fn full_text(&self) -> String {
        self.lines.join("\n")
    }

    /// Convert a (row, col) position into an absolute byte offset in `full_text()`.
    pub fn absolute_offset(&self, row: usize, col: usize) -> usize {
        let (clamped_row, clamped_col) = self.clamp_position(row, col);
        let mut offset = 0usize;
        for (idx, line) in self.lines.iter().enumerate().take(clamped_row) {
            offset += line.len() + 1; // account for the newline between lines
            let _ = idx;
        }
        offset + clamped_col
    }

    /// Clamp a byte offset to the nearest valid character boundary in a line.
    pub fn clamp_column(&self, row: usize, col: usize) -> usize {
        let row = row.min(self.lines.len().saturating_sub(1));
        clamp_char_boundary(self.line(row), col)
    }

    /// Clamp a buffer position to a valid row and UTF-8 boundary.
    pub fn clamp_position(&self, row: usize, col: usize) -> (usize, usize) {
        let row = row.min(self.lines.len().saturating_sub(1));
        (row, self.clamp_column(row, col))
    }

    /// Return the buffer position after inserting `text` at `start`.
    pub fn position_after(start: (usize, usize), text: &str) -> (usize, usize) {
        let (mut row, mut col) = start;
        for ch in text.chars() {
            if ch == '\n' {
                row += 1;
                col = 0;
            } else {
                col += ch.len_utf8();
            }
        }
        (row, col)
    }

    // ──────────────────────────────────────────────────────────────────
    // Mutation
    // ──────────────────────────────────────────────────────────────────

    /// Insert a character at (row, col) where col is a byte offset.
    /// Returns the new cursor col after insertion.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn insert_char(&mut self, row: usize, col: usize, ch: char) -> usize {
        let line = &mut self.lines[row];
        let col = clamp_char_boundary(line, col);
        line.insert(col, ch);
        self.dirty = true;
        col + ch.len_utf8()
    }

    /// Delete the character immediately before (row, col).
    /// If col == 0 and row > 0, joins this line with the previous one.
    /// Returns the new (row, col) after deletion.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn delete_char_before(&mut self, row: usize, col: usize) -> (usize, usize) {
        let col = self.clamp_column(row, col);
        if col > 0 {
            let line = &mut self.lines[row];
            // Find the start of the previous char (handle multi-byte UTF-8).
            let new_col = prev_char_boundary(line, col);
            line.drain(new_col..col);
            self.dirty = true;
            (row, new_col)
        } else if row > 0 {
            // Join current line onto the end of the previous line.
            let current = self.lines.remove(row);
            let prev_len = self.lines[row - 1].len();
            self.lines[row - 1].push_str(&current);
            self.dirty = true;
            (row - 1, prev_len)
        } else {
            (row, col) // nothing to delete
        }
    }

    /// Delete the character at (row, col).
    /// If col is at end of line and there is a next line, joins the lines.
    /// Returns the new (row, col) — the cursor does not move.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn delete_char_at(&mut self, row: usize, col: usize) -> (usize, usize) {
        let col = self.clamp_column(row, col);
        let line_len = self.lines[row].len();
        if col < line_len {
            let end = next_char_boundary(self.lines[row].as_str(), col);
            if end > col {
                self.lines[row].drain(col..end);
                self.dirty = true;
            }
            (row, col)
        } else if row + 1 < self.lines.len() {
            let next = self.lines.remove(row + 1);
            self.lines[row].push_str(&next);
            self.dirty = true;
            (row, col)
        } else {
            (row, col) // at end of last line — nothing to do
        }
    }

    /// Split the line at (row, col) and insert a new line below.
    /// Returns the new cursor position (row+1, 0).
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn insert_newline(&mut self, row: usize, col: usize) -> (usize, usize) {
        let col = self.clamp_column(row, col);
        let remainder = self.lines[row].split_off(col);
        self.lines.insert(row + 1, remainder);
        self.dirty = true;
        (row + 1, 0)
    }

    // ──────────────────────────────────────────────────────────────────
    // Range operations
    // ──────────────────────────────────────────────────────────────────

    /// Get the text in the range [start, end) where each is (row, col).
    pub fn text_in_range(&self, start: (usize, usize), end: (usize, usize)) -> String {
        if start.0 >= self.lines.len() {
            return String::new();
        }
        let (start_row, start_col) = self.clamp_position(start.0, start.1);
        let (end_row, end_col) = self.clamp_position(end.0, end.1);
        if start_row == end_row {
            let line = &self.lines[start_row];
            let s = start_col.min(line.len());
            let e = end_col.min(line.len());
            if s >= e {
                return String::new();
            }
            return line[s..e].to_string();
        }
        let mut out = String::new();
        let first_line = &self.lines[start_row];
        let s = start_col.min(first_line.len());
        out.push_str(&first_line[s..]);
        for row in (start_row + 1)..end_row {
            out.push('\n');
            if row < self.lines.len() {
                out.push_str(&self.lines[row]);
            }
        }
        if end_row < self.lines.len() {
            out.push('\n');
            let last_line = &self.lines[end_row];
            let e = end_col.min(last_line.len());
            out.push_str(&last_line[..e]);
        }
        out
    }

    /// Delete the content in the range [start, end).
    pub fn delete_range(&mut self, start: (usize, usize), end: (usize, usize)) {
        if start.0 >= self.lines.len() {
            return;
        }
        let (start_row, start_col) = self.clamp_position(start.0, start.1);
        let (end_row, end_col) = self.clamp_position(end.0, end.1);
        if start_row == end_row {
            let line = &mut self.lines[start_row];
            let s = start_col.min(line.len());
            let e = end_col.min(line.len());
            if s < e {
                line.drain(s..e);
                self.dirty = true;
            }
            return;
        }
        // Multi-line delete: keep prefix of first line + suffix of last line
        let suffix = {
            let last_line = &self.lines[end_row];
            let e = end_col.min(last_line.len());
            last_line[e..].to_string()
        };
        let prefix = {
            let first_line = &self.lines[start_row];
            let s = start_col.min(first_line.len());
            first_line[..s].to_string()
        };
        // Remove rows from start_row+1 to end_row inclusive
        self.lines.drain((start_row + 1)..=(end_row));
        // Merge prefix + suffix back into start_row
        self.lines[start_row] = prefix + &suffix;
        self.dirty = true;
    }

    /// Insert `text` at (row, col). Handles embedded newlines.
    pub fn insert_text_at(&mut self, row: usize, col: usize, text: &str) {
        if row >= self.lines.len() {
            return;
        }
        let col = self.clamp_column(row, col);
        if !text.contains('\n') {
            self.lines[row].insert_str(col, text);
            self.dirty = true;
            return;
        }
        // Split on newlines and splice into buffer
        let suffix = self.lines[row].split_off(col);
        let parts: Vec<&str> = text.split('\n').collect();
        // Append first part to current line
        if let Some(first) = parts.first() {
            self.lines[row].push_str(first);
        }
        // Insert middle and last parts as new lines
        let insert_pos = row + 1;
        for (i, part) in parts.iter().enumerate().skip(1) {
            if i == parts.len() - 1 {
                // Last part: append suffix
                self.lines.insert(insert_pos + i - 1, format!("{part}{suffix}"));
            } else {
                self.lines.insert(insert_pos + i - 1, part.to_string());
            }
        }
        // If text ended with newline (empty last part), suffix goes on its own line
        if parts.last().map(|p| p.is_empty()).unwrap_or(false) && parts.len() > 1 {
            // Already handled above
        }
        self.dirty = true;
    }

    /// Replace the range described by `old_text` at `start` with `new_text`.
    pub fn apply_change(&mut self, start: (usize, usize), old_text: &str, new_text: &str) {
        if old_text == new_text {
            return;
        }

        let end = Self::position_after(start, old_text);
        self.delete_range(start, end);
        if !new_text.is_empty() {
            self.insert_text_at(start.0, start.1, new_text);
        }
    }

    // ──────────────────────────────────────────────────────────────────
    // I/O
    // ──────────────────────────────────────────────────────────────────

    /// Save the buffer to its file path using an atomic tmp → rename.
    pub fn save(&mut self) -> Result<()> {
        let path = self.file_path.as_ref().context("No file path — use save_as")?;
        let content = self.lines.join("\n") + "\n";
        let tmp_path = temp_save_path(path);
        std::fs::write(&tmp_path, &content)
            .with_context(|| format!("Cannot write tmp file: {}", tmp_path.display()))?;
        std::fs::rename(&tmp_path, path)
            .with_context(|| format!("Cannot rename tmp to: {}", path.display()))?;
        self.dirty = false;
        Ok(())
    }
}

// ──────────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────────

/// Find the byte index of the start of the character before `pos` in `s`.
pub(crate) fn prev_char_boundary(s: &str, pos: usize) -> usize {
    let mut idx = clamp_char_boundary(s, pos).saturating_sub(1);
    while idx > 0 && !s.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}

/// Find the byte index of the start of the character after `pos` in `s`.
pub(crate) fn next_char_boundary(s: &str, pos: usize) -> usize {
    let mut idx = (clamp_char_boundary(s, pos) + 1).min(s.len());
    while idx < s.len() && !s.is_char_boundary(idx) {
        idx += 1;
    }
    idx
}

pub(crate) fn clamp_char_boundary(s: &str, pos: usize) -> usize {
    let mut idx = pos.min(s.len());
    while idx > 0 && !s.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}

fn temp_save_path(path: &Path) -> PathBuf {
    match (path.parent(), path.file_name()) {
        (Some(parent), Some(name)) => {
            let mut tmp_name = name.to_os_string();
            tmp_name.push(".tmp");
            parent.join(tmp_name)
        }
        _ => path.with_extension("tmp"),
    }
}

// ──────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn buf(lines: &[&str]) -> TextBuffer {
        TextBuffer {
            lines: lines.iter().map(|s| s.to_string()).collect(),
            dirty: false,
            file_path: None,
        }
    }

    #[test]
    fn new_empty_has_one_line() {
        let b = TextBuffer::new_empty();
        assert_eq!(b.line_count(), 1);
        assert_eq!(b.line(0), "");
        assert!(!b.dirty);
    }

    #[test]
    fn insert_char_middle() {
        let mut b = buf(&["hello"]);
        let new_col = b.insert_char(0, 2, 'X');
        assert_eq!(b.line(0), "heXllo");
        assert_eq!(new_col, 3);
        assert!(b.dirty);
    }

    #[test]
    fn insert_char_at_end() {
        let mut b = buf(&["hi"]);
        let new_col = b.insert_char(0, 2, '!');
        assert_eq!(b.line(0), "hi!");
        assert_eq!(new_col, 3);
    }

    #[test]
    fn delete_before_middle() {
        let mut b = buf(&["hello"]);
        let (r, c) = b.delete_char_before(0, 3);
        assert_eq!(b.line(0), "helo");
        assert_eq!((r, c), (0, 2));
    }

    #[test]
    fn delete_before_joins_lines() {
        let mut b = buf(&["foo", "bar"]);
        let (r, c) = b.delete_char_before(1, 0);
        assert_eq!(b.line_count(), 1);
        assert_eq!(b.line(0), "foobar");
        assert_eq!((r, c), (0, 3));
    }

    #[test]
    fn delete_before_noop_at_start() {
        let mut b = buf(&["x"]);
        let (r, c) = b.delete_char_before(0, 0);
        assert_eq!(b.line(0), "x");
        assert_eq!((r, c), (0, 0));
        assert!(!b.dirty);
    }

    #[test]
    fn delete_at_middle() {
        let mut b = buf(&["hello"]);
        let (r, c) = b.delete_char_at(0, 1);
        assert_eq!(b.line(0), "hllo");
        assert_eq!((r, c), (0, 1));
    }

    #[test]
    fn delete_at_end_joins_lines() {
        let mut b = buf(&["foo", "bar"]);
        let (r, c) = b.delete_char_at(0, 3);
        assert_eq!(b.line_count(), 1);
        assert_eq!(b.line(0), "foobar");
        assert_eq!((r, c), (0, 3));
    }

    #[test]
    fn delete_at_noop_at_last_position() {
        let mut b = buf(&["x"]);
        let (r, c) = b.delete_char_at(0, 1);
        assert_eq!(b.line(0), "x");
        assert_eq!((r, c), (0, 1));
        assert!(!b.dirty);
    }

    #[test]
    fn insert_newline_splits_line() {
        let mut b = buf(&["hello world"]);
        let (r, c) = b.insert_newline(0, 5);
        assert_eq!(b.line_count(), 2);
        assert_eq!(b.line(0), "hello");
        assert_eq!(b.line(1), " world");
        assert_eq!((r, c), (1, 0));
    }

    #[test]
    fn insert_newline_at_start() {
        let mut b = buf(&["text"]);
        let (r, c) = b.insert_newline(0, 0);
        assert_eq!(b.line(0), "");
        assert_eq!(b.line(1), "text");
        assert_eq!((r, c), (1, 0));
    }

    #[test]
    fn insert_newline_at_end() {
        let mut b = buf(&["text"]);
        let (r, c) = b.insert_newline(0, 4);
        assert_eq!(b.line(0), "text");
        assert_eq!(b.line(1), "");
        assert_eq!((r, c), (1, 0));
    }

    #[test]
    fn insert_char_clamps_to_utf8_boundary() {
        let mut b = buf(&["zaż"]);
        let new_col = b.insert_char(0, 3, 'X');
        assert_eq!(b.line(0), "zaXż");
        assert_eq!(new_col, 3);
    }

    #[test]
    fn delete_at_clamps_to_utf8_boundary() {
        let mut b = buf(&["zaż"]);
        let (r, c) = b.delete_char_at(0, 3);
        assert_eq!(b.line(0), "za");
        assert_eq!((r, c), (0, 2));
    }

    #[test]
    fn text_in_range_clamps_misaligned_boundaries() {
        let b = buf(&["zażółć"]);
        assert_eq!(b.text_in_range((0, 3), (0, 8)), "żół");
    }

    #[test]
    fn clamp_position_snaps_to_valid_utf8_boundary() {
        let b = buf(&["zażółć"]);
        assert_eq!(b.clamp_position(0, 3), (0, 2));
        assert_eq!(b.clamp_position(0, 11), (0, "zażółć".len()));
    }

    #[test]
    fn display_name_no_path() {
        let b = TextBuffer::new_empty();
        assert_eq!(b.display_name(), "[New File]");
    }

    #[test]
    fn temp_save_path_for_tmp_file_is_distinct() {
        let original = Path::new("/tmp/example.tmp");
        let temp = temp_save_path(original);

        assert_ne!(temp, original);
        assert_eq!(temp.file_name().and_then(|name| name.to_str()), Some("example.tmp.tmp"));
    }
}
