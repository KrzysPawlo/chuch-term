use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use crate::editor::EditorState;

const BG: Color = Color::Rgb(18, 18, 18);
const ACCENT: Color = Color::Rgb(176, 196, 200);
const DIM: Color = Color::Rgb(90, 90, 90);

/// Go-to-line bar rendered in the hints area.
pub struct GotoBar<'a> {
    pub state: &'a EditorState,
}

impl<'a> Widget for GotoBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }
        let y = area.top();
        for x in area.left()..area.right() {
            buf[(x, y)].set_bg(BG).set_char(' ');
        }

        let line_count = self.state.buffer.line_count();
        let prompt = format!(
            " Go to line: {}_   [1\u{2013}{}]",
            self.state.goto_input, line_count
        );
        let hint = "   Enter Confirm  \u{00b7}  Esc Cancel";

        let accent_style = Style::default()
            .fg(ACCENT)
            .bg(BG)
            .add_modifier(Modifier::BOLD);
        let dim_style = Style::default().fg(DIM).bg(BG);

        let mut x = area.left();
        let max_x = area.right();

        for ch in prompt.chars() {
            if x >= max_x {
                break;
            }
            buf[(x, y)].set_char(ch).set_style(accent_style);
            x += 1;
        }
        for ch in hint.chars() {
            if x >= max_x {
                break;
            }
            buf[(x, y)].set_char(ch).set_style(dim_style);
            x += 1;
        }
    }
}
