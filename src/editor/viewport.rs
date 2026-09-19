use super::cursor::Cursor;

/// Tracks which row/column is at the top-left of the visible area.
#[derive(Debug, Clone, Copy, Default)]
pub struct Viewport {
    pub offset_row: usize,
    pub offset_col: usize,
}

impl Viewport {
    pub fn new() -> Self {
        Self {
            offset_row: 0,
            offset_col: 0,
        }
    }

    /// Adjust the viewport so that `cursor` is visible within `viewport_height` rows.
    pub fn scroll_to_cursor(&mut self, cursor: &Cursor, viewport_height: usize) {
        let h = viewport_height.max(1);
        if cursor.row < self.offset_row {
            self.offset_row = cursor.row;
        } else if cursor.row >= self.offset_row + h {
            self.offset_row = cursor.row.saturating_sub(h - 1);
        }
    }

    /// Adjust the horizontal viewport offset so the cursor's display column
    /// stays visible within `width` columns. `cursor_display_col` is the
    /// cursor's absolute display column within its line (not screen-relative).
    pub fn scroll_to_cursor_horizontal(&mut self, cursor_display_col: usize, width: usize) {
        let w = width.max(1);
        if cursor_display_col < self.offset_col {
            self.offset_col = cursor_display_col;
        } else if cursor_display_col >= self.offset_col + w {
            self.offset_col = cursor_display_col.saturating_sub(w - 1);
        }
    }
}

// ──────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrolls_down_when_cursor_below() {
        let mut vp = Viewport::new();
        let cursor = Cursor { row: 25, col: 0 };
        vp.scroll_to_cursor(&cursor, 24);
        assert_eq!(vp.offset_row, 2); // 25 - 24 + 1 = 2
    }

    #[test]
    fn scrolls_up_when_cursor_above() {
        let mut vp = Viewport {
            offset_row: 10,
            offset_col: 0,
        };
        let cursor = Cursor { row: 5, col: 0 };
        vp.scroll_to_cursor(&cursor, 24);
        assert_eq!(vp.offset_row, 5);
    }

    #[test]
    fn no_scroll_when_cursor_in_view() {
        let mut vp = Viewport::new();
        let cursor = Cursor { row: 10, col: 0 };
        vp.scroll_to_cursor(&cursor, 24);
        assert_eq!(vp.offset_row, 0);
    }

    #[test]
    fn scrolls_right_when_cursor_display_col_beyond_view() {
        let mut vp = Viewport::new();
        vp.scroll_to_cursor_horizontal(100, 40);
        assert_eq!(vp.offset_col, 61); // 100 - 40 + 1 = 61
    }

    #[test]
    fn scrolls_left_when_cursor_display_col_before_offset() {
        let mut vp = Viewport {
            offset_row: 0,
            offset_col: 50,
        };
        vp.scroll_to_cursor_horizontal(10, 40);
        assert_eq!(vp.offset_col, 10);
    }

    #[test]
    fn no_horizontal_scroll_when_cursor_col_in_view() {
        let mut vp = Viewport::new();
        vp.scroll_to_cursor_horizontal(20, 40);
        assert_eq!(vp.offset_col, 0);
    }
}
