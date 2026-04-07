use super::cursor::Cursor;

/// Tracks which row is at the top of the visible area.
#[derive(Debug, Clone, Copy, Default)]
pub struct Viewport {
    pub offset_row: usize,
}

impl Viewport {
    pub fn new() -> Self {
        Self { offset_row: 0 }
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
}

// ──────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrolls_down_when_cursor_below() {
        let mut vp = Viewport { offset_row: 0 };
        let cursor = Cursor { row: 25, col: 0 };
        vp.scroll_to_cursor(&cursor, 24);
        assert_eq!(vp.offset_row, 2); // 25 - 24 + 1 = 2
    }

    #[test]
    fn scrolls_up_when_cursor_above() {
        let mut vp = Viewport { offset_row: 10 };
        let cursor = Cursor { row: 5, col: 0 };
        vp.scroll_to_cursor(&cursor, 24);
        assert_eq!(vp.offset_row, 5);
    }

    #[test]
    fn no_scroll_when_cursor_in_view() {
        let mut vp = Viewport { offset_row: 0 };
        let cursor = Cursor { row: 10, col: 0 };
        vp.scroll_to_cursor(&cursor, 24);
        assert_eq!(vp.offset_row, 0);
    }
}
