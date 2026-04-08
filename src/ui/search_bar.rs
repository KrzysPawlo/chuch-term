use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use crate::editor::EditorState;


/// Search bar rendered in the hints area during Search mode.
pub struct SearchBar<'a> {
    pub state: &'a EditorState,
}

impl<'a> Widget for SearchBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }
        let (r, g, b) = self.state.config.theme.bg_bar_rgb();
        let bg = Color::Rgb(r, g, b);
        let (r, g, b) = self.state.config.theme.accent_rgb();
        let key_fg = Color::Rgb(r, g, b);
        let (r, g, b) = self.state.config.theme.dim_rgb();
        let dim_fg = Color::Rgb(r, g, b);

        let y = area.top();
        for x in area.left()..area.right() {
            buf[(x, y)].set_bg(bg).set_fg(dim_fg).set_char(' ');
        }

        let total = self.state.search_results.len();
        let current = if total == 0 {
            0
        } else {
            self.state.search_result_idx + 1
        };

        let prompt = format!(" / {}   [{}/{}]", self.state.search_query, current, total);
        let hint = if self.state.search_case_sensitive {
            "  [Cc]  ^N Next  \u{00b7}  ^P Prev  \u{00b7}  ^I Case  \u{00b7}  ^R Replace  \u{00b7}  Enter Select  \u{00b7}  Esc Close"
        } else {
            "  ^N Next  \u{00b7}  ^P Prev  \u{00b7}  ^I Case  \u{00b7}  ^R Replace  \u{00b7}  Enter Select  \u{00b7}  Esc Close"
        };

        let accent_style = Style::default()
            .fg(key_fg)
            .bg(bg)
            .add_modifier(Modifier::BOLD);
        let dim_style = Style::default().fg(dim_fg).bg(bg);

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
