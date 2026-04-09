use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use crate::editor::EditorState;

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

        if area.width < 50 || area.height < 12 {
            render_compact(area, buf, accent, overlay_bg);
            return;
        }

        render_full(area, buf, &self.state.palette, accent, key_fg);
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
    let rows: &[(&str, &str, &str, &str)] = &[
        ("\u{2191}\u{2193}\u{2190}\u{2192}", "Move cursor",    "Type",          "Insert character"),
        ("Home",          "Line start",      "Backspace",      "Delete backward"),
        ("End",           "Line end",        "Delete",         "Delete forward"),
        ("PgUp",          "Scroll up",       "Enter",          "New line"),
        ("PgDn",          "Scroll down",     "Ctrl+S",         "Save file"),
        ("Shift+Arrows",  "Select text",     "Ctrl+Q",         "Quit"),
        ("Ctrl+\u{2190}/\u{2192}", "Word left/right", "Ctrl+H", "This help"),
        ("Ctrl+Z",        "Undo",            "Ctrl+F",         "Find in file"),
        ("Ctrl+Y",        "Redo",            "Ctrl+L",         "Toggle line numbers"),
        ("Ctrl+G",        "Go to line",      "Ctrl+C/X/V",     "Copy/Cut/Paste"),
        ("Ctrl+P",        "Command palette", "Ctrl+R",         "Find & replace"),
        ("Ctrl+A",        "Select all",      "Ctrl+O",         "Go back (prev file)"),
        ("Alt+U / Alt+L", "Upper/Lowercase", "Ctrl+W / Del",   "Delete word \u{2190}/\u{2192}"),
        ("Ctrl+D",        "Duplicate line",  "Alt+, / Ctrl+T", "Settings"),
    ];

    for (k1, d1, k2, d2) in rows {
        if y >= area.bottom().saturating_sub(3) {
            break;
        }
        if !k1.is_empty() {
            kv(buf, y, col1, col2, k1, d1, &kv_styles);
        }
        if !k2.is_empty() {
            kv(buf, y, col2, right, k2, d2, &kv_styles);
        }
        y += 1;
    }

    y += 1;

    // ── Footer ────────────────────────────────────────────────────────
    if y + 2 < area.bottom() {
        y = rule(buf, y, margin, right, palette.overlay_rule_fg, overlay_bg);
        let footer = "Esc or ^H to close";
        let footer_x = right.saturating_sub(footer.len() as u16);
        put(
            buf,
            footer_x,
            y,
            footer,
            Style::default().fg(palette.overlay_footer_fg).bg(overlay_bg),
            area.right(),
        );
    }
}

// ── Compact layout (narrow / short terminal) ──────────────────────────

fn render_compact(area: Rect, buf: &mut Buffer, accent: Color, overlay_bg: Color) {
    let y = area.top() + area.height / 2;
    let msg = "chuch-term  \u{2014}  Esc to close help";
    let msg_len = msg.chars().count() as u16;
    let x = area
        .left()
        .saturating_add(area.width.saturating_sub(msg_len) / 2);
    put(
        buf,
        x,
        y,
        msg,
        Style::default().fg(accent).bg(overlay_bg),
        area.right(),
    );
}
