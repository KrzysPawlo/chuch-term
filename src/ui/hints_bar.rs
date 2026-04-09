use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use crate::editor::{EditorMode, EditorState};
use crate::shortcuts::{LabelStyle, ShortcutAction};

/// One-row contextual hints bar rendered below the status bar.
pub struct HintsBar<'a> {
    pub state: &'a EditorState,
}

impl<'a> Widget for HintsBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }
        let y = area.top();

        // Resolve theme colours from config.
        let bg = self.state.palette.theme_bg_bar;
        let accent = self.state.palette.theme_accent;
        let dim = self.state.palette.theme_dim;
        let warning = self.state.palette.theme_warning;
        let normal_colors = HintColors {
            accent,
            dim,
            bg,
            sep_fg: self.state.palette.hints_sep_fg,
        };

        // Fill background — set both fg and bg explicitly to prevent style leaks
        // across ratatui frames (set_bg alone leaves fg from the previous frame).
        for x in area.left()..area.right() {
            buf[(x, y)].set_style(Style::default().bg(bg).fg(dim)).set_char(' ');
        }

        match self.state.mode {
            EditorMode::Normal => {
                if self.state.selection_anchor.is_some() {
                    render_selection_hints(
                        area,
                        buf,
                        self.state,
                        &normal_colors,
                    );
                } else {
                    render_normal(
                        area,
                        buf,
                        self.state,
                        self.state.previous_buffer.is_some(),
                        &normal_colors,
                    );
                }
            }
            EditorMode::ConfirmQuit => {
                render_confirm(
                    area,
                    buf,
                    self.state,
                    warning,
                    bg,
                    self.state.palette.hints_warn_sep_fg,
                )
            }
            EditorMode::Help => render_help(area, buf, self.state, dim, bg),
            EditorMode::Search | EditorMode::GoToLine => {
                // These modes render their own bar widget (search_bar / goto_bar).
            }
            EditorMode::Replace | EditorMode::CommandPalette
            | EditorMode::SaveAs | EditorMode::Settings | EditorMode::Keybindings
            | EditorMode::CommandAlias => {
                // Respective overlays/bars handle these modes.
            }
        }
    }
}

// ── Render helpers ─────────────────────────────────────────────────────

/// Write `text` at (x, y) with `style`, clipped to `max_x`. Returns new x.
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

/// Render a sequence of (key, description) hint pairs separated by ` · `.
fn render_hints(
    area: Rect,
    buf: &mut Buffer,
    hints: &[(&str, &str)],
    key_style: Style,
    desc_style: Style,
    sep_style: Style,
) {
    let y = area.top();
    let max_x = area.right();
    let mut x = area.left() + 1; // 1-cell left padding

    for (i, (key, desc)) in hints.iter().enumerate() {
        if i > 0 {
            x = put(buf, x, y, "  \u{00b7}  ", sep_style, max_x);
        }
        x = put(buf, x, y, key, key_style, max_x);
        x = put(buf, x, y, "  ", desc_style, max_x);
        x = put(buf, x, y, desc, desc_style, max_x);
        let _ = x;
    }
}

fn render_normal(
    area: Rect,
    buf: &mut Buffer,
    state: &EditorState,
    has_prev: bool,
    colors: &HintColors,
) {
    let key_style  = Style::default().fg(colors.accent).bg(colors.bg).add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(colors.dim).bg(colors.bg);
    let sep_style  = Style::default().fg(colors.sep_fg).bg(colors.bg);
    let save = state.active_shortcuts.label_for(ShortcutAction::Save, LabelStyle::Compact);
    let undo = state.active_shortcuts.label_for(ShortcutAction::Undo, LabelStyle::Compact);
    let search = state.active_shortcuts.label_for(ShortcutAction::Search, LabelStyle::Compact);
    let palette = state.active_shortcuts.label_for(ShortcutAction::Palette, LabelStyle::Compact);
    let back = state.active_shortcuts.label_for(ShortcutAction::GoBackBuffer, LabelStyle::Compact);
    let help = state.active_shortcuts.label_for(ShortcutAction::Help, LabelStyle::Compact);

    if has_prev {
        render_hints(
            area, buf,
            &[
                (&save, "Save"),
                (&undo, "Undo"),
                (&search, "Find"),
                (&palette, "Commands"),
                (&back, "Back"),
                (&help, "Help"),
            ],
            key_style, desc_style, sep_style,
        );
    } else {
        render_hints(
            area, buf,
            &[
                (&save, "Save"),
                (&undo, "Undo"),
                (&search, "Find"),
                (&palette, "Commands"),
                (&help, "Help"),
            ],
            key_style, desc_style, sep_style,
        );
    }
}

fn render_selection_hints(
    area: Rect,
    buf: &mut Buffer,
    state: &EditorState,
    colors: &HintColors,
) {
    let key_style  = Style::default().fg(colors.accent).bg(colors.bg).add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(colors.dim).bg(colors.bg);
    let sep_style  = Style::default().fg(colors.sep_fg).bg(colors.bg);
    let copy = state.active_shortcuts.label_for(ShortcutAction::Copy, LabelStyle::Compact);
    let cut = state.active_shortcuts.label_for(ShortcutAction::Cut, LabelStyle::Compact);
    let paste = state.active_shortcuts.label_for(ShortcutAction::Paste, LabelStyle::Compact);

    render_hints(
        area, buf,
        &[
            (&copy, "Copy"),
            (&cut, "Cut"),
            (&paste, "Paste"),
            ("Esc", "Clear"),
        ],
        key_style, desc_style, sep_style,
    );
}

fn render_confirm(area: Rect, buf: &mut Buffer, state: &EditorState, warning: Color, bg: Color, sep_fg: Color) {
    let key_style  = Style::default().fg(warning).bg(bg).add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(warning).bg(bg);
    let sep_style  = Style::default().fg(sep_fg).bg(bg);
    let quit = state.active_shortcuts.label_for(ShortcutAction::Quit, LabelStyle::Compact);
    let save = state.active_shortcuts.label_for(ShortcutAction::Save, LabelStyle::Compact);

    render_hints(
        area, buf,
        &[
            (&quit, "Force Quit"),
            (&save, "Save & Quit"),
            ("Esc", "Cancel"),
        ],
        key_style, desc_style, sep_style,
    );
}

fn render_help(area: Rect, buf: &mut Buffer, state: &EditorState, dim: Color, bg: Color) {
    let text = format!(
        "Esc or {}  Close Help",
        state.active_shortcuts.label_for(ShortcutAction::Help, LabelStyle::Compact)
    );
    let text_len = text.chars().count() as u16;
    let x = area
        .left()
        .saturating_add(area.width.saturating_sub(text_len) / 2);
    let style = Style::default().fg(dim).bg(bg);
    put(buf, x, area.top(), &text, style, area.right());
}

struct HintColors {
    accent: Color,
    dim: Color,
    bg: Color,
    sep_fg: Color,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::EditorState;

    #[test]
    fn hints_bar_uses_theme_bg_bar() {
        let mut state = EditorState::new_empty();
        state.config.render.color_mode = "rgb".to_string();
        state.config.theme.bg_bar = "#224466".to_string();
        let config = state.config.clone();
        state.apply_config(config);

        let area = Rect::new(0, 0, 16, 1);
        let mut buf = Buffer::empty(area);
        HintsBar { state: &state }.render(area, &mut buf);

        for x in area.left()..area.right() {
            assert_eq!(buf[(x, 0)].bg, Color::Rgb(34, 68, 102));
        }
    }
}
