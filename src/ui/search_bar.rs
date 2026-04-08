use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
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
        let bg = self.state.palette.theme_bg_bar;
        let key_fg = self.state.palette.theme_accent;
        let dim_fg = self.state.palette.theme_dim;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::{EditorMode, EditorState};
    use ratatui::style::Color;

    #[test]
    fn search_bar_uses_theme_bg_bar() {
        let mut state = EditorState::new_empty();
        state.mode = EditorMode::Search;
        state.config.render.color_mode = "rgb".to_string();
        state.config.theme.bg_bar = "#334455".to_string();
        let config = state.config.clone();
        state.apply_config(config);

        let area = Rect::new(0, 0, 18, 1);
        let mut buf = Buffer::empty(area);
        SearchBar { state: &state }.render(area, &mut buf);

        for x in area.left()..area.right() {
            assert_eq!(buf[(x, 0)].bg, Color::Rgb(51, 68, 85));
        }
    }
}
