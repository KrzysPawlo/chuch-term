use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};
use crate::editor::{EditorState, LineNumberMode};

const ACCENT: Color = Color::Rgb(176, 196, 200);  // #b0c4c8
const DIM: Color = Color::Rgb(90, 90, 90);        // #5a5a5a
const BG: Color = Color::Rgb(10, 10, 10);         // #0a0a0a

/// Calculate the gutter width needed to display line numbers.
pub fn gutter_width(line_count: usize) -> u16 {
    let digits = line_count.to_string().len() as u16;
    (digits + 1).max(3)
}

/// Line numbers gutter widget.
pub struct LineNumbersGutter<'a> {
    pub state: &'a EditorState,
}

impl<'a> Widget for LineNumbersGutter<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let offset = self.state.viewport.offset_row;
        let line_count = self.state.buffer.line_count();
        let current_row = self.state.cursor.row;

        for screen_row in 0..area.height {
            let buf_row = offset + screen_row as usize;
            let y = area.top() + screen_row;

            // Clear the gutter row
            for x in area.left()..area.right() {
                buf[(x, y)].set_char(' ').set_bg(BG);
            }

            if buf_row >= line_count {
                continue;
            }

            let (num_str, is_current) = match self.state.line_number_mode {
                LineNumberMode::Off => continue,
                LineNumberMode::Absolute => {
                    let n = buf_row + 1;
                    (format!("{n}"), buf_row == current_row)
                }
                LineNumberMode::Relative => {
                    let dist = if buf_row == current_row {
                        buf_row + 1 // show absolute on current line
                    } else {
                        buf_row.abs_diff(current_row)
                    };
                    (format!("{dist}"), buf_row == current_row)
                }
            };

            let style = if is_current {
                Style::default().fg(ACCENT).bg(BG)
            } else {
                Style::default().fg(DIM).bg(BG)
            };

            // Right-align the number in the gutter (leave 1 space on the right)
            let num_len = num_str.len() as u16;
            let gutter_inner = area.width.saturating_sub(1); // 1 space right margin
            let x_start = if num_len < gutter_inner {
                area.left() + gutter_inner - num_len
            } else {
                area.left()
            };

            let mut x = x_start;
            for ch in num_str.chars() {
                if x >= area.right().saturating_sub(1) {
                    break;
                }
                buf[(x, y)].set_char(ch).set_style(style);
                x += 1;
            }
        }
    }
}
