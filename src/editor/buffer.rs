use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

const UTF8_BOM: &[u8; 3] = b"\xEF\xBB\xBF";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineEnding {
    #[default]
    None,
    Lf,
    Crlf,
}

impl LineEnding {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Lf => "\n",
            Self::Crlf => "\r\n",
        }
    }

    pub fn serialized_len(self) -> usize {
        self.as_str().len()
    }

    fn insertion_default(self) -> Self {
        match self {
            Self::None => Self::Lf,
            other => other,
        }
    }
}

/// Text buffer with exact serialization metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextBuffer {
    pub lines: Vec<String>,
    pub line_endings: Vec<LineEnding>,
    pub dirty: bool,
    pub file_path: Option<PathBuf>,
    pub has_utf8_bom: bool,
    pub insertion_line_ending: LineEnding,
}

impl TextBuffer {
    /// Create an empty buffer (single empty line, no file path).
    pub fn new_empty() -> Self {
        Self::from_lines_with_metadata(vec![String::new()], false, LineEnding::Lf, None)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn from_lines(lines: Vec<String>) -> Self {
        Self::from_lines_with_metadata(lines, false, LineEnding::Lf, None)
    }

    fn from_lines_with_metadata(
        mut lines: Vec<String>,
        has_utf8_bom: bool,
        insertion_line_ending: LineEnding,
        line_endings: Option<Vec<LineEnding>>,
    ) -> Self {
        if lines.is_empty() {
            lines.push(String::new());
        }

        let mut buffer = Self {
            line_endings: line_endings.unwrap_or_else(|| default_line_endings(lines.len())),
            lines,
            dirty: false,
            file_path: None,
            has_utf8_bom,
            insertion_line_ending: insertion_line_ending.insertion_default(),
        };
        buffer.normalize_metadata();
        buffer
    }

    /// Load a file into a buffer.
    pub fn from_file(path: &Path) -> Result<Self> {
        let bytes = std::fs::read(path)
            .with_context(|| format!("Cannot open file: {}", path.display()))?;
        let (has_utf8_bom, payload) = if bytes.starts_with(UTF8_BOM) {
            (true, &bytes[UTF8_BOM.len()..])
        } else {
            (false, bytes.as_slice())
        };
        let content = std::str::from_utf8(payload)
            .with_context(|| format!("Unsupported encoding for {}: only UTF-8 is supported", path.display()))?;
        let (lines, line_endings, insertion_line_ending) = parse_document(content);
        let mut buffer = Self::from_lines_with_metadata(
            lines,
            has_utf8_bom,
            insertion_line_ending,
            Some(line_endings),
        );
        buffer.file_path = Some(path.to_path_buf());
        Ok(buffer)
    }

    /// Number of lines in the buffer.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Reference to a line by index. Panics if out of bounds — callers must clamp.
    pub fn line(&self, row: usize) -> &str {
        &self.lines[row]
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn line_ending(&self, row: usize) -> LineEnding {
        self.line_endings[row]
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn set_lines_for_testing(&mut self, lines: Vec<String>) {
        self.lines = lines;
        self.line_endings = default_line_endings(self.lines.len());
        self.insertion_line_ending = LineEnding::Lf;
        self.has_utf8_bom = false;
        self.normalize_metadata();
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

    pub fn serialized_text(&self) -> String {
        let mut out = String::with_capacity(
            self.lines.iter().map(String::len).sum::<usize>()
                + self
                    .line_endings
                    .iter()
                    .map(|ending| ending.serialized_len())
                    .sum::<usize>(),
        );
        for (line, ending) in self.lines.iter().zip(self.line_endings.iter().copied()) {
            out.push_str(line);
            out.push_str(ending.as_str());
        }
        out
    }

    pub fn serialized_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        if self.has_utf8_bom {
            bytes.extend_from_slice(UTF8_BOM);
        }
        bytes.extend_from_slice(self.serialized_text().as_bytes());
        bytes
    }

    /// Clamp a byte offset to the nearest valid grapheme boundary in a line.
    pub fn clamp_column(&self, row: usize, col: usize) -> usize {
        let row = row.min(self.lines.len().saturating_sub(1));
        clamp_grapheme_boundary(self.line(row), col)
    }

    /// Clamp a buffer position to a valid row and grapheme boundary.
    pub fn clamp_position(&self, row: usize, col: usize) -> (usize, usize) {
        let row = row.min(self.lines.len().saturating_sub(1));
        (row, self.clamp_column(row, col))
    }

    /// Return the buffer position after inserting `text` at `start`.
    pub fn position_after(start: (usize, usize), text: &str) -> (usize, usize) {
        let parts: Vec<&str> = text.split('\n').collect();
        if parts.len() == 1 {
            (start.0, start.1 + text.len())
        } else {
            (start.0 + parts.len() - 1, parts.last().map_or(0, |part| part.len()))
        }
    }

    pub fn dominant_line_ending(&self) -> LineEnding {
        self.insertion_line_ending.insertion_default()
    }

    // ──────────────────────────────────────────────────────────────────
    // Mutation
    // ──────────────────────────────────────────────────────────────────

    /// Insert a character at (row, col) where col is a byte offset.
    /// Returns the new cursor col after insertion.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn insert_char(&mut self, row: usize, col: usize, ch: char) -> usize {
        let line = &mut self.lines[row];
        let col = clamp_grapheme_boundary(line, col);
        line.insert(col, ch);
        self.mark_dirty();
        col + ch.len_utf8()
    }

    /// Delete the grapheme immediately before (row, col).
    /// If col == 0 and row > 0, joins this line with the previous one.
    /// Returns the new (row, col) after deletion.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn delete_char_before(&mut self, row: usize, col: usize) -> (usize, usize) {
        let col = self.clamp_column(row, col);
        if col > 0 {
            let line = &mut self.lines[row];
            let new_col = prev_grapheme_boundary(line, col);
            line.drain(new_col..col);
            self.mark_dirty();
            (row, new_col)
        } else if row > 0 {
            let current = self.lines.remove(row);
            let current_ending = self.line_endings.remove(row);
            let prev_len = self.lines[row - 1].len();
            self.lines[row - 1].push_str(&current);
            self.line_endings[row - 1] = current_ending;
            self.mark_dirty();
            (row - 1, prev_len)
        } else {
            (row, col)
        }
    }

    /// Delete the grapheme at (row, col).
    /// If col is at end of line and there is a next line, joins the lines.
    /// Returns the new (row, col) — the cursor does not move.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn delete_char_at(&mut self, row: usize, col: usize) -> (usize, usize) {
        let col = self.clamp_column(row, col);
        let line_len = self.lines[row].len();
        if col < line_len {
            let end = next_grapheme_boundary(self.lines[row].as_str(), col);
            if end > col {
                self.lines[row].drain(col..end);
                self.mark_dirty();
            }
            (row, col)
        } else if row + 1 < self.lines.len() {
            let next = self.lines.remove(row + 1);
            let next_ending = self.line_endings.remove(row + 1);
            self.lines[row].push_str(&next);
            self.line_endings[row] = next_ending;
            self.mark_dirty();
            (row, col)
        } else {
            (row, col)
        }
    }

    /// Split the line at (row, col) and insert a new line below.
    /// Returns the new cursor position (row+1, 0).
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn insert_newline(&mut self, row: usize, col: usize) -> (usize, usize) {
        let col = self.clamp_column(row, col);
        let remainder = self.lines[row].split_off(col);
        let trailing_ending = self.line_endings[row];
        self.line_endings[row] = self.dominant_line_ending();
        self.lines.insert(row + 1, remainder);
        self.line_endings.insert(row + 1, trailing_ending);
        self.mark_dirty();
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
            out.push_str(&self.lines[row]);
        }
        out.push('\n');
        let last_line = &self.lines[end_row];
        let e = end_col.min(last_line.len());
        out.push_str(&last_line[..e]);
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
                self.mark_dirty();
            }
            return;
        }

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
        let trailing_ending = self.line_endings[end_row];

        self.lines.drain((start_row + 1)..=end_row);
        self.line_endings.drain((start_row + 1)..=end_row);
        self.lines[start_row] = prefix + &suffix;
        self.line_endings[start_row] = trailing_ending;
        self.mark_dirty();
    }

    /// Insert `text` at (row, col). Handles embedded newlines.
    pub fn insert_text_at(&mut self, row: usize, col: usize, text: &str) {
        if row >= self.lines.len() {
            return;
        }
        let col = self.clamp_column(row, col);
        if !text.contains('\n') {
            self.lines[row].insert_str(col, text);
            self.mark_dirty();
            return;
        }

        let suffix = self.lines[row].split_off(col);
        let trailing_ending = self.line_endings[row];
        let parts: Vec<&str> = text.split('\n').collect();
        self.lines[row].push_str(parts.first().copied().unwrap_or_default());
        self.line_endings[row] = self.dominant_line_ending();

        let insert_ending = self.dominant_line_ending();
        let mut insert_at = row + 1;
        for part in parts.iter().skip(1).take(parts.len().saturating_sub(2)) {
            self.lines.insert(insert_at, (*part).to_string());
            self.line_endings.insert(insert_at, insert_ending);
            insert_at += 1;
        }

        let last_part = parts.last().copied().unwrap_or_default();
        self.lines.insert(insert_at, format!("{last_part}{suffix}"));
        self.line_endings.insert(insert_at, trailing_ending);
        self.mark_dirty();
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
        let content = self.serialized_bytes();
        let tmp_path = temp_save_path(path);
        std::fs::write(&tmp_path, &content)
            .with_context(|| format!("Cannot write tmp file: {}", tmp_path.display()))?;
        std::fs::rename(&tmp_path, path)
            .with_context(|| format!("Cannot rename tmp to: {}", path.display()))?;
        self.dirty = false;
        Ok(())
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
        self.normalize_metadata();
    }

    fn normalize_metadata(&mut self) {
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }

        match self.line_endings.len().cmp(&self.lines.len()) {
            std::cmp::Ordering::Less => {
                self.line_endings
                    .extend(default_line_endings(self.lines.len() - self.line_endings.len()));
            }
            std::cmp::Ordering::Greater => self.line_endings.truncate(self.lines.len()),
            std::cmp::Ordering::Equal => {}
        }

        if let Some(last) = self.line_endings.last_mut()
            && self.lines.len() == 1
            && self.lines[0].is_empty()
        {
            *last = LineEnding::None;
        }

        self.insertion_line_ending = dominant_line_ending(&self.line_endings);
    }
}

fn default_line_endings(line_count: usize) -> Vec<LineEnding> {
    match line_count {
        0 => vec![LineEnding::None],
        1 => vec![LineEnding::None],
        n => {
            let mut endings = vec![LineEnding::Lf; n];
            if let Some(last) = endings.last_mut() {
                *last = LineEnding::None;
            }
            endings
        }
    }
}

fn parse_document(content: &str) -> (Vec<String>, Vec<LineEnding>, LineEnding) {
    let mut lines = Vec::new();
    let mut endings = Vec::new();
    let bytes = content.as_bytes();
    let mut start = 0usize;
    let mut idx = 0usize;

    while idx < bytes.len() {
        match bytes[idx] {
            b'\n' => {
                lines.push(content[start..idx].to_string());
                endings.push(LineEnding::Lf);
                idx += 1;
                start = idx;
            }
            b'\r' if idx + 1 < bytes.len() && bytes[idx + 1] == b'\n' => {
                lines.push(content[start..idx].to_string());
                endings.push(LineEnding::Crlf);
                idx += 2;
                start = idx;
            }
            _ => idx += 1,
        }
    }

    lines.push(content[start..].to_string());
    endings.push(LineEnding::None);

    if lines.is_empty() {
        lines.push(String::new());
        endings.push(LineEnding::None);
    }

    let insertion = dominant_line_ending(&endings);
    (lines, endings, insertion)
}

fn dominant_line_ending(line_endings: &[LineEnding]) -> LineEnding {
    let mut lf = 0usize;
    let mut crlf = 0usize;
    for ending in line_endings {
        match ending {
            LineEnding::Lf => lf += 1,
            LineEnding::Crlf => crlf += 1,
            LineEnding::None => {}
        }
    }

    if crlf > lf {
        LineEnding::Crlf
    } else if lf > 0 {
        LineEnding::Lf
    } else if crlf > 0 {
        LineEnding::Crlf
    } else {
        LineEnding::Lf
    }
}

// ──────────────────────────────────────────────────────────────────────
// Grapheme helpers
// ──────────────────────────────────────────────────────────────────────

pub(crate) fn prev_grapheme_boundary(s: &str, pos: usize) -> usize {
    let target = clamp_grapheme_boundary(s, pos);
    let mut previous = 0usize;
    for (idx, _) in s.grapheme_indices(true) {
        if idx >= target {
            break;
        }
        previous = idx;
    }
    previous
}

pub(crate) fn next_grapheme_boundary(s: &str, pos: usize) -> usize {
    let target = clamp_grapheme_boundary(s, pos);
    if target >= s.len() {
        return s.len();
    }
    for (idx, grapheme) in s.grapheme_indices(true) {
        if idx == target {
            return idx + grapheme.len();
        }
        if idx > target {
            return idx;
        }
    }
    s.len()
}

pub(crate) fn clamp_grapheme_boundary(s: &str, pos: usize) -> usize {
    let target = pos.min(s.len());
    let mut boundary = 0usize;
    for (idx, _) in s.grapheme_indices(true) {
        if idx > target {
            break;
        }
        boundary = idx;
    }
    if target == s.len() {
        s.len()
    } else {
        boundary
    }
}

pub(crate) fn grapheme_display_width(grapheme: &str) -> usize {
    grapheme.width().max(1)
}

pub(crate) fn display_col_for_byte(s: &str, target: usize) -> usize {
    let target = clamp_grapheme_boundary(s, target);
    let mut display = 0usize;
    for (idx, grapheme) in s.grapheme_indices(true) {
        if idx >= target {
            break;
        }
        display += grapheme_display_width(grapheme);
    }
    display
}

pub(crate) fn byte_for_display_col(s: &str, target_col: usize) -> usize {
    let mut display = 0usize;
    for (idx, grapheme) in s.grapheme_indices(true) {
        let width = grapheme_display_width(grapheme);
        if target_col < display + width {
            let midpoint = display + width.div_ceil(2);
            return if target_col >= midpoint {
                idx + grapheme.len()
            } else {
                idx
            };
        }
        display += width;
    }
    s.len()
}

pub(crate) fn grapheme_slice(s: &str, start: usize, end: usize) -> &str {
    let start = clamp_grapheme_boundary(s, start);
    let end = clamp_grapheme_boundary(s, end).max(start);
    &s[start..end]
}

fn temp_save_path(path: &Path) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();

    match (path.parent(), path.file_name()) {
        (Some(parent), Some(name)) => {
            let tmp_name = format!(
                ".{}.{}.{}.tmp",
                name.to_string_lossy(),
                std::process::id(),
                unique
            );
            parent.join(tmp_name)
        }
        _ => path.with_extension(format!("{}.tmp", unique)),
    }
}

// ──────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn buf(lines: &[&str]) -> TextBuffer {
        TextBuffer::from_lines(lines.iter().map(|line| (*line).to_string()).collect())
    }

    #[test]
    fn new_empty_has_one_line() {
        let b = TextBuffer::new_empty();
        assert_eq!(b.line_count(), 1);
        assert_eq!(b.line(0), "");
        assert!(!b.dirty);
        assert_eq!(b.line_ending(0), LineEnding::None);
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
    fn delete_before_joins_lines_and_preserves_trailing_ending() {
        let mut b = buf(&["foo", "bar"]);
        let (r, c) = b.delete_char_before(1, 0);
        assert_eq!(b.line_count(), 1);
        assert_eq!(b.line(0), "foobar");
        assert_eq!((r, c), (0, 3));
        assert_eq!(b.line_ending(0), LineEnding::None);
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
        assert_eq!(b.line_ending(0), LineEnding::Lf);
        assert_eq!(b.line_ending(1), LineEnding::None);
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
    fn insert_char_clamps_to_grapheme_boundary() {
        let mut b = buf(&["e\u{301}x"]);
        let new_col = b.insert_char(0, 1, 'X');
        assert_eq!(b.line(0), "Xe\u{301}x");
        assert_eq!(new_col, 1);
    }

    #[test]
    fn delete_at_clamps_to_grapheme_boundary() {
        let mut b = buf(&["e\u{301}x"]);
        let (r, c) = b.delete_char_at(0, 1);
        assert_eq!(b.line(0), "x");
        assert_eq!((r, c), (0, 0));
    }

    #[test]
    fn text_in_range_clamps_misaligned_boundaries() {
        let b = buf(&["zażółć"]);
        assert_eq!(b.text_in_range((0, 3), (0, 8)), "żół");
    }

    #[test]
    fn clamp_position_snaps_to_valid_grapheme_boundary() {
        let b = buf(&["e\u{301}x"]);
        assert_eq!(b.clamp_position(0, 1), (0, 0));
        assert_eq!(b.clamp_position(0, "e\u{301}x".len()), (0, "e\u{301}x".len()));
    }

    #[test]
    fn display_name_no_path() {
        let b = TextBuffer::new_empty();
        assert_eq!(b.display_name(), "[New File]");
    }

    #[test]
    fn exact_round_trip_preserves_bom_and_mixed_endings() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let path = std::env::temp_dir().join(format!(
            "chuch-term-buffer-roundtrip-{}-{}-mixed.txt",
            std::process::id(),
            unique
        ));
        let original = b"\xEF\xBB\xBFalpha\r\nbeta\ngamma";
        std::fs::write(&path, original).expect("write fixture");

        let mut buffer = TextBuffer::from_file(&path).expect("load");
        assert_eq!(buffer.serialized_bytes(), original);

        buffer.save().expect("save");
        let saved = std::fs::read(&path).expect("read saved");
        assert_eq!(saved, original);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn save_preserves_missing_trailing_newline() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let path = std::env::temp_dir().join(format!(
            "chuch-term-buffer-roundtrip-{}-{}-plain.txt",
            std::process::id(),
            unique
        ));
        std::fs::write(&path, b"alpha").expect("write fixture");

        let mut buffer = TextBuffer::from_file(&path).expect("load");
        buffer.save().expect("save");

        assert_eq!(std::fs::read(&path).expect("read"), b"alpha");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn insert_text_uses_dominant_line_ending() {
        let mut buffer = TextBuffer::from_lines_with_metadata(
            vec!["alpha".to_string(), "omega".to_string()],
            false,
            LineEnding::Crlf,
            Some(vec![LineEnding::Crlf, LineEnding::None]),
        );
        buffer.insert_text_at(0, 5, "\nnext");
        assert_eq!(buffer.line_ending(0), LineEnding::Crlf);
        assert_eq!(buffer.serialized_text(), "alpha\r\nnext\r\nomega");
    }

    #[test]
    fn temp_save_path_for_tmp_file_is_distinct() {
        let original = Path::new("/tmp/example.tmp");
        let temp = temp_save_path(original);

        assert_ne!(temp, original);
    }

    #[test]
    fn byte_for_display_col_handles_graphemes() {
        let line = "A👨‍👩‍👧‍👦B";
        assert_eq!(byte_for_display_col(line, 0), 0);
        assert_eq!(byte_for_display_col(line, 1), 1);
        assert_eq!(display_col_for_byte(line, 1), 1);
        assert_eq!(display_col_for_byte(line, line.len()), 4);
    }
}
