use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use crate::editor::EditorState;


/// Save-as bar rendered in the hints area when EditorMode::SaveAs is active.
pub struct SaveAsBar<'a> {
    pub state: &'a EditorState,
}

impl<'a> Widget for SaveAsBar<'a> {
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

        let prompt = format!(" Save as: {}_", self.state.saveas_input);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::{EditorMode, EditorState};

    #[test]
    fn saveas_bar_uses_theme_bg_bar() {
        let mut state = EditorState::new_empty();
        state.mode = EditorMode::SaveAs;
        state.config.theme.bg_bar = "#556677".to_string();

        let area = Rect::new(0, 0, 18, 1);
        let mut buf = Buffer::empty(area);
        SaveAsBar { state: &state }.render(area, &mut buf);

        for x in area.left()..area.right() {
            assert_eq!(buf[(x, 0)].bg, Color::Rgb(85, 102, 119));
        }
    }
}
