use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use crate::editor::EditorState;


/// Go-to-line bar rendered in the hints area.
pub struct GotoBar<'a> {
    pub state: &'a EditorState,
}

impl<'a> Widget for GotoBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }
        let (r, g, b) = self.state.config.theme.bg_bar_rgb();
        let bg = Color::Rgb(r, g, b);
        let (r, g, b) = self.state.config.theme.accent_rgb();
        let accent = Color::Rgb(r, g, b);
        let (r, g, b) = self.state.config.theme.dim_rgb();
        let dim = Color::Rgb(r, g, b);

        let y = area.top();
        for x in area.left()..area.right() {
            buf[(x, y)].set_bg(bg).set_fg(dim).set_char(' ');
        }

        let line_count = self.state.buffer.line_count();
        let prompt = format!(
            " Go to line: {}_   [1\u{2013}{}]",
            self.state.goto_input, line_count
        );
        let hint = "   Enter Confirm  \u{00b7}  Esc Cancel";

        let accent_style = Style::default()
            .fg(accent)
            .bg(bg)
            .add_modifier(Modifier::BOLD);
        let dim_style = Style::default().fg(dim).bg(bg);

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
