use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::Widget,
};

use crate::editor::EditorState;

pub struct CommandAliasOverlay<'a> {
    pub state: &'a EditorState,
}

impl<'a> Widget for CommandAliasOverlay<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let overlay_bg = self.state.palette.overlay_bg;
        let accent = self.state.palette.theme_accent;
        let dim = self.state.palette.theme_dim;

        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buf[(x, y)].set_char(' ').set_bg(overlay_bg).set_fg(overlay_bg);
            }
        }

        if area.width < 52 || area.height < 9 {
            render_compact(area, buf, accent, dim, overlay_bg);
            return;
        }

        let margin = area.left() + 3;
        let right = area.right().saturating_sub(3);
        let mut y = area.top() + 1;

        let title_style = Style::default()
            .fg(accent)
            .bg(overlay_bg)
            .add_modifier(Modifier::BOLD);
        put(buf, margin, y, "Command Alias", title_style, right);
        let close_hint = "Esc to go back";
        let close_x = right.saturating_sub(close_hint.len() as u16);
        put(
            buf,
            close_x,
            y,
            close_hint,
            Style::default().fg(dim).bg(overlay_bg),
            area.right(),
        );
        y += 2;

        let intro = format!(
            "Choose one short personal command. chuch-term stays canonical; e.g. {} file.rs",
            if self.state.command_alias_input.trim().is_empty() {
                "cct"
            } else {
                self.state.command_alias_input.trim()
            }
        );
        put(
            buf,
            margin,
            y,
            &intro,
            Style::default().fg(dim).bg(overlay_bg),
            right,
        );
        y += 2;

        let prompt = format!(" Alias: {}_", self.state.command_alias_input);
        put(
            buf,
            margin,
            y,
            &prompt,
            Style::default()
                .fg(accent)
                .bg(overlay_bg)
                .add_modifier(Modifier::BOLD),
            right,
        );
        y += 2;

        let hint = "Allowed: a-z, 0-9, '_' and '-'  ·  Enter Save  ·  Backspace Delete";
        put(
            buf,
            margin,
            y,
            hint,
            Style::default().fg(dim).bg(overlay_bg),
            right,
        );
    }
}

fn render_compact(area: Rect, buf: &mut Buffer, accent: ratatui::style::Color, dim: ratatui::style::Color, overlay_bg: ratatui::style::Color) {
    let y = area.top() + area.height / 2;
    let msg = "Command Alias  —  Resize wider or press Esc";
    let x = area
        .left()
        .saturating_add(area.width.saturating_sub(msg.chars().count() as u16) / 2);
    put(
        buf,
        x,
        y,
        msg,
        Style::default().fg(accent).bg(overlay_bg).add_modifier(Modifier::BOLD),
        area.right(),
    );
    if y + 1 < area.bottom() {
        put(
            buf,
            area.left().saturating_add(2),
            y + 1,
            "Edit your personal launch alias here.",
            Style::default().fg(dim).bg(overlay_bg),
            area.right(),
        );
    }
}

fn put(buf: &mut Buffer, x: u16, y: u16, text: &str, style: Style, max_x: u16) -> u16 {
    let mut cx = x;
    for ch in text.chars() {
        if cx >= max_x {
            break;
        }
        buf[(cx, y)].set_char(ch).set_style(style);
        cx += 1;
    }
    cx
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::{EditorMode, EditorState};

    #[test]
    fn command_alias_overlay_has_compact_fallback() {
        let mut state = EditorState::new_empty();
        state.mode = EditorMode::CommandAlias;
        let area = Rect::new(0, 0, 38, 4);
        let mut buf = Buffer::empty(area);

        CommandAliasOverlay { state: &state }.render(area, &mut buf);

        let mut rendered = String::new();
        for y in 0..area.height {
            for x in 0..area.width {
                rendered.push_str(buf[(x, y)].symbol());
            }
        }
        assert!(rendered.contains("Command Alias"));
    }
}
