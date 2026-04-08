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

/// Save-as bar rendered in the hints area when EditorMode::SaveAs is active.
pub struct SaveAsBar<'a> {
    pub state: &'a EditorState,
}

impl<'a> Widget for SaveAsBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }
        let y = area.top();
        for x in area.left()..area.right() {
            buf[(x, y)].set_bg(BG).set_char(' ');
        }

        let prompt = format!(" Save as: {}_", self.state.saveas_input);
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
