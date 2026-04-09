use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use crate::editor::EditorState;
use crate::shortcuts::{LabelStyle, ShortcutAction};

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Full-screen help overlay. Rendered on top of everything when mode == Help.
pub struct HelpOverlay<'a> {
    pub state: &'a EditorState,
}

impl<'a> Widget for HelpOverlay<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let overlay_bg = self.state.palette.overlay_bg;
        // Fill entire area with overlay background first — this covers all existing cells.
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buf[(x, y)]
                    .set_char(' ')
                    .set_bg(overlay_bg)
                    .set_fg(overlay_bg);
            }
        }

        let accent = self.state.palette.theme_accent;
        let key_fg = self.state.palette.theme_warning;

        if area.width < 68 || area.height < 14 {
            render_compact(self.state, area, buf, accent, overlay_bg);
            return;
        }

        render_full(self.state, area, buf, &self.state.palette, accent, key_fg);
    }
}

// ── Render helpers ─────────────────────────────────────────────────────

/// Write text at (x, y) with style, clipped to max_x. Returns new x.
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

/// Draw a horizontal rule from x to max_x at row y. Returns next y.
fn rule(
    buf: &mut Buffer,
    y: u16,
    x: u16,
    max_x: u16,
    rule_fg: Color,
    overlay_bg: Color,
) -> u16 {
    let style = Style::default().fg(rule_fg).bg(overlay_bg);
    let mut cx = x;
    while cx < max_x {
        buf[(cx, y)].set_char('\u{2500}').set_style(style); // ─
        cx += 1;
    }
    y.saturating_add(1)
}

/// Styles bundle for `kv()` to stay under clippy's argument-count limit.
struct KvStyles {
    key_w: u16,
    key: Style,
    desc: Style,
}

/// Draw a key+description pair. Key occupies `styles.key_w` columns, desc follows.
fn kv(buf: &mut Buffer, y: u16, x: u16, max_x: u16, key: &str, desc: &str, styles: &KvStyles) {
    let mut cx = x;
    // Write key, left-aligned within key_w.
    for ch in key.chars() {
        if cx >= x + styles.key_w || cx >= max_x {
            break;
        }
        buf[(cx, y)].set_char(ch).set_style(styles.key);
        cx += 1;
    }
    // Pad remaining key column with spaces.
    let fill_bg = styles.key.bg.unwrap_or(Color::Reset);
    let bg_style = Style::default().bg(fill_bg).fg(fill_bg);
    while cx < x + styles.key_w && cx < max_x {
        buf[(cx, y)].set_char(' ').set_style(bg_style);
        cx += 1;
    }
    // Write description.
    for ch in desc.chars() {
        if cx >= max_x {
            break;
        }
        buf[(cx, y)].set_char(ch).set_style(styles.desc);
        cx += 1;
    }
}

// ── Full layout (≥ 50 cols, ≥ 12 rows) ────────────────────────────────

fn render_full(
    state: &EditorState,
    area: Rect,
    buf: &mut Buffer,
    palette: &crate::color::Palette,
    accent: Color,
    key_fg: Color,
) {
    let overlay_bg = palette.overlay_bg;
    let margin = area.left() + 3;
    let right = area.right().saturating_sub(3);
    let mut y = area.top() + 1;

    // ── Header ────────────────────────────────────────────────────────
    let header_style = Style::default()
        .fg(accent)
        .bg(overlay_bg)
        .add_modifier(Modifier::BOLD);

    let hx = put(buf, margin, y, "chuch-term", header_style, right);
    // Version right-aligned on the same row.
    let ver = format!("v{VERSION}");
    let ver_x = right.saturating_sub(ver.len() as u16);
    if ver_x > hx {
        put(buf, ver_x, y, &ver, Style::default().fg(palette.overlay_version_fg).bg(overlay_bg), right);
    }
    y += 1;
    y += 1;

    y = rule(buf, y, margin, right, palette.overlay_rule_fg, overlay_bg);
    y += 1;

    // ── Two-column keybindings ─────────────────────────────────────────
    let half = (area.width.saturating_sub(6)) / 2;
    let col1 = margin;
    let col2 = margin + half;
    const KEY_W: u16 = 15;

    let sec_style = Style::default()
        .fg(palette.overlay_section_fg)
        .bg(overlay_bg)
        .add_modifier(Modifier::BOLD);
    let kv_styles = KvStyles {
        key_w: KEY_W,
        key: Style::default().fg(key_fg).bg(overlay_bg).add_modifier(Modifier::BOLD),
        desc: Style::default().fg(palette.overlay_desc_fg).bg(overlay_bg),
    };

    // Section headers.
    put(buf, col1, y, "NAVIGATION", sec_style, col2);
    put(buf, col2, y, "EDITING & FILE", sec_style, right);
    y += 1;

    // Paired rows: (left_key, left_desc, right_key, right_desc).
    // Empty string = skip that cell.
    let save = state.active_shortcuts.label_for(ShortcutAction::Save, LabelStyle::Long);
    let quit = state.active_shortcuts.label_for(ShortcutAction::Quit, LabelStyle::Long);
    let help = state.active_shortcuts.label_for(ShortcutAction::Help, LabelStyle::Long);
    let undo = state.active_shortcuts.label_for(ShortcutAction::Undo, LabelStyle::Long);
    let redo = state.active_shortcuts.label_for(ShortcutAction::Redo, LabelStyle::Long);
    let search = state.active_shortcuts.label_for(ShortcutAction::Search, LabelStyle::Long);
    let goto = state.active_shortcuts.label_for(ShortcutAction::GoToLine, LabelStyle::Long);
    let line_numbers = state.active_shortcuts.label_for(ShortcutAction::ToggleLineNumbers, LabelStyle::Long);
    let palette_shortcut = state.active_shortcuts.label_for(ShortcutAction::Palette, LabelStyle::Long);
    let select_all = state.active_shortcuts.label_for(ShortcutAction::SelectAll, LabelStyle::Long);
    let copy = state.active_shortcuts.label_for(ShortcutAction::Copy, LabelStyle::Long);
    let cut = state.active_shortcuts.label_for(ShortcutAction::Cut, LabelStyle::Long);
    let paste = state.active_shortcuts.label_for(ShortcutAction::Paste, LabelStyle::Long);
    let go_back = state.active_shortcuts.label_for(ShortcutAction::GoBackBuffer, LabelStyle::Long);
    let replace = state.active_shortcuts.label_for(ShortcutAction::Replace, LabelStyle::Long);
    let uppercase = state.active_shortcuts.label_for(ShortcutAction::UppercaseSelection, LabelStyle::Long);
    let lowercase = state.active_shortcuts.label_for(ShortcutAction::LowercaseSelection, LabelStyle::Long);
    let delete_word_before = state.active_shortcuts.label_for(ShortcutAction::DeleteWordBefore, LabelStyle::Long);
    let delete_word_after = state.active_shortcuts.label_for(ShortcutAction::DeleteWordAfter, LabelStyle::Long);
    let word_left = state.active_shortcuts.label_for(ShortcutAction::WordLeft, LabelStyle::Long);
    let word_right = state.active_shortcuts.label_for(ShortcutAction::WordRight, LabelStyle::Long);
    let duplicate = state.active_shortcuts.label_for(ShortcutAction::DuplicateLine, LabelStyle::Long);
    let settings = state.active_shortcuts.label_for(ShortcutAction::Settings, LabelStyle::Long);

    let rows: [(String, &'static str, String, &'static str); 14] = [
        ("\u{2191}\u{2193}\u{2190}\u{2192}".to_string(), "Move cursor", "Type".to_string(), "Insert character"),
        ("Home".to_string(), "Line start", "Backspace".to_string(), "Delete backward"),
        ("End".to_string(), "Line end", "Delete".to_string(), "Delete forward"),
        ("PgUp".to_string(), "Scroll up", "Enter".to_string(), "New line"),
        ("PgDn".to_string(), "Scroll down", save, "Save file"),
        ("Shift+Arrows".to_string(), "Select text", quit, "Quit"),
        (format!("{word_left} / {word_right}"), "Word left/right", help.clone(), "This help"),
        (undo, "Undo", search, "Find in file"),
        (redo, "Redo", line_numbers, "Toggle line numbers"),
        (goto, "Go to line", format!("{copy} / {cut} / {paste}"), "Copy/Cut/Paste"),
        (palette_shortcut, "Command palette", replace, "Find & replace"),
        (select_all, "Select all", go_back, "Go back (prev file)"),
        (format!("{uppercase} / {lowercase}"), "Upper/Lowercase", format!("{delete_word_before} / {delete_word_after}"), "Delete word \u{2190}/\u{2192}"),
        (duplicate, "Duplicate line", settings, "Settings"),
    ];

    for (k1, d1, k2, d2) in rows {
        if y >= area.bottom().saturating_sub(3) {
            break;
        }
        if !k1.is_empty() {
            kv(buf, y, col1, col2, &k1, d1, &kv_styles);
        }
        if !k2.is_empty() {
            kv(buf, y, col2, right, &k2, d2, &kv_styles);
        }
        y += 1;
    }

    y += 1;

    // ── Footer ────────────────────────────────────────────────────────
    if y + 2 < area.bottom() {
        y = rule(buf, y, margin, right, palette.overlay_rule_fg, overlay_bg);
        let footer = format!("Esc or {} to close", state.active_shortcuts.label_for(ShortcutAction::Help, LabelStyle::Compact));
        let footer_x = right.saturating_sub(footer.len() as u16);
        put(
            buf,
            footer_x,
            y,
            &footer,
            Style::default().fg(palette.overlay_footer_fg).bg(overlay_bg),
            area.right(),
        );
    }
}

// ── Compact layout (narrow / short terminal) ──────────────────────────

fn render_compact(state: &EditorState, area: Rect, buf: &mut Buffer, accent: Color, overlay_bg: Color) {
    let y = area.top() + area.height / 2;
    let msg = format!(
        "chuch-term  \u{2014}  Esc or {} to close help",
        state.active_shortcuts.label_for(ShortcutAction::Help, LabelStyle::Compact)
    );
    let msg_len = msg.chars().count() as u16;
    let x = area
        .left()
        .saturating_add(area.width.saturating_sub(msg_len) / 2);
    put(
        buf,
        x,
        y,
        &msg,
        Style::default().fg(accent).bg(overlay_bg),
        area.right(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::{EditorMode, EditorState};

    #[test]
    fn help_overlay_has_compact_fallback() {
        let mut state = EditorState::new_empty();
        state.mode = EditorMode::Help;
        let area = Rect::new(0, 0, 42, 4);
        let mut buf = Buffer::empty(area);

        HelpOverlay { state: &state }.render(area, &mut buf);

        let mut rendered = String::new();
        for y in 0..area.height {
            for x in 0..area.width {
                rendered.push_str(buf[(x, y)].symbol());
            }
        }
        assert!(rendered.contains("close help"));
    }
}
