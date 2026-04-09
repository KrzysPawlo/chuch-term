use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::Widget,
};

use crate::editor::EditorState;
use crate::shortcuts::{configurable_actions, LabelStyle, ShortcutAction};

pub struct KeybindingsOverlay<'a> {
    pub state: &'a EditorState,
}

impl<'a> Widget for KeybindingsOverlay<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let overlay_bg = self.state.palette.overlay_bg;
        let accent = self.state.palette.theme_accent;
        let dim = self.state.palette.theme_dim;
        let warning = self.state.palette.theme_warning;

        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buf[(x, y)].set_char(' ').set_bg(overlay_bg).set_fg(overlay_bg);
            }
        }

        if area.width < 72 || area.height < 12 {
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
        put(buf, margin, y, "Shortcut Overrides", title_style, right);
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
            "Profile: {}  ·  Enter to capture  ·  Backspace resets one shortcut",
            self.state.active_shortcuts.profile().name()
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

        let visible_rows = area.bottom().saturating_sub(y + 2) as usize;
        let cursor = self.state.keybindings_cursor;
        let scroll_offset = if cursor >= visible_rows {
            cursor - visible_rows + 1
        } else {
            0
        };

        let key_col = margin + 26;
        let desc_col = margin + 42;

        for (idx, action) in configurable_actions()
            .iter()
            .copied()
            .enumerate()
            .skip(scroll_offset)
            .take(visible_rows)
        {
            let is_selected = idx == cursor;
            let row_style = if is_selected {
                Style::default().fg(self.state.palette.command_selected_fg).bg(accent)
            } else {
                Style::default().fg(accent).bg(overlay_bg)
            };
            let key_style = if is_selected {
                Style::default()
                    .fg(self.state.palette.command_selected_fg)
                    .bg(accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(warning).bg(overlay_bg).add_modifier(Modifier::BOLD)
            };
            let desc_style = if is_selected {
                Style::default().fg(self.state.palette.command_selected_fg).bg(accent)
            } else {
                Style::default().fg(dim).bg(overlay_bg)
            };

            if is_selected {
                for x in margin..right {
                    buf[(x, y)].set_bg(accent).set_fg(self.state.palette.command_selected_fg);
                }
            }

            put(buf, margin + 1, y, action.name(), row_style, right);
            let key = self.state.active_shortcuts.label_for(action, LabelStyle::Long);
            put(buf, key_col.min(right), y, &key, key_style, right);
            put(buf, desc_col.min(right), y, action.description(), desc_style, right);
            y += 1;
        }

        let footer = if self.state.keybinding_capture {
            capture_message(self.state.selected_keybinding_action())
        } else {
            "Capture mode is off".to_string()
        };
        let footer_style = if self.state.keybinding_capture {
            Style::default().fg(warning).bg(overlay_bg).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(dim).bg(overlay_bg)
        };
        put(buf, margin, area.bottom().saturating_sub(2), &footer, footer_style, right);
    }
}

fn render_compact(
    area: Rect,
    buf: &mut Buffer,
    accent: ratatui::style::Color,
    dim: ratatui::style::Color,
    overlay_bg: ratatui::style::Color,
) {
    let y = area.top() + area.height / 2;
    let msg = "Shortcut Overrides  —  Resize wider or press Esc";
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
            "The active keymap stays valid. Use a wider terminal to edit it.",
            Style::default().fg(dim).bg(overlay_bg),
            area.right(),
        );
    }
}

fn capture_message(action: Option<ShortcutAction>) -> String {
    match action {
        Some(action) => format!(
            "Press a-z, comma, Delete, Left or Right to set {}. Esc cancels.",
            action.name()
        ),
        None => "Press a supported key token. Esc cancels.".to_string(),
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
    fn keybindings_overlay_has_compact_fallback() {
        let mut state = EditorState::new_empty();
        state.mode = EditorMode::Keybindings;
        let area = Rect::new(0, 0, 44, 4);
        let mut buf = Buffer::empty(area);

        KeybindingsOverlay { state: &state }.render(area, &mut buf);

        let mut rendered = String::new();
        for y in 0..area.height {
            for x in 0..area.width {
                rendered.push_str(buf[(x, y)].symbol());
            }
        }
        assert!(rendered.contains("Shortcut Overrides"));
    }
}
