use ratatui::style::Color;

use crate::config::EditorConfig;

pub const EDITOR_BG_HEX: &str = "#121212";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalEnv {
    pub term: String,
    pub colorterm: String,
    pub term_program: String,
}

impl TerminalEnv {
    pub fn detect() -> Self {
        Self {
            term: std::env::var("TERM").unwrap_or_else(|_| "(not set)".into()),
            colorterm: std::env::var("COLORTERM").unwrap_or_else(|_| "(not set)".into()),
            term_program: std::env::var("TERM_PROGRAM").unwrap_or_else(|_| "(not set)".into()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestedColorMode {
    Auto,
    Rgb,
    Ansi256,
}

impl RequestedColorMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Rgb => "rgb",
            Self::Ansi256 => "ansi256",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectiveColorMode {
    Rgb,
    Ansi256,
}

impl EffectiveColorMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Rgb => "rgb",
            Self::Ansi256 => "ansi256",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderDecision {
    pub declared_support: &'static str,
    pub requested: RequestedColorMode,
    pub effective: EffectiveColorMode,
    pub reason: &'static str,
}

#[derive(Debug, Clone)]
pub struct Palette {
    pub theme_accent: Color,
    pub theme_warning: Color,
    pub theme_dim: Color,
    pub theme_bg_bar: Color,
    pub editor_bg: Color,
    pub tilde_fg: Color,
    pub indent_guide_fg: Color,
    pub text_main: Color,
    pub lang_fg: Color,
    pub syntax_keyword_fg: Color,
    pub syntax_string_fg: Color,
    pub syntax_comment_fg: Color,
    pub syntax_number_fg: Color,
    pub syntax_type_fg: Color,
    pub syntax_attribute_fg: Color,
    pub search_match_bg: Color,
    pub search_current_fg: Color,
    pub selection_bg: Color,
    pub line_number_bg: Color,
    pub hints_sep_fg: Color,
    pub hints_warn_sep_fg: Color,
    pub overlay_bg: Color,
    pub overlay_version_fg: Color,
    pub overlay_section_fg: Color,
    pub overlay_desc_fg: Color,
    pub overlay_rule_fg: Color,
    pub overlay_footer_fg: Color,
    pub command_selected_fg: Color,
    pub command_separator_fg: Color,
    pub settings_label_fg: Color,
    pub settings_inactive_label_fg: Color,
    pub settings_check_on_fg: Color,
    pub settings_check_off_fg: Color,
    pub settings_dim_fg: Color,
    pub settings_footer_fg: Color,
    pub settings_rule_fg: Color,
}

pub fn parse_requested_color_mode(value: &str) -> Option<RequestedColorMode> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(RequestedColorMode::Auto),
        "rgb" => Some(RequestedColorMode::Rgb),
        "ansi256" => Some(RequestedColorMode::Ansi256),
        _ => None,
    }
}

pub fn resolve_render_decision(config: &EditorConfig, env: &TerminalEnv) -> RenderDecision {
    let requested = parse_requested_color_mode(&config.render.color_mode)
        .unwrap_or(RequestedColorMode::Auto);
    let term = env.term.trim().to_ascii_lowercase();
    let colorterm = env.colorterm.trim().to_ascii_lowercase();
    let term_program = env.term_program.trim().to_ascii_lowercase();

    let declared_support = match colorterm.as_str() {
        "truecolor" | "24bit" => "RGB announced by COLORTERM",
        _ if term.contains("256color") => {
            "256-color support reported by TERM; RGB not confirmed"
        }
        _ => "basic / unknown terminal colors reported by environment",
    };

    match requested {
        RequestedColorMode::Rgb => RenderDecision {
            declared_support,
            requested,
            effective: EffectiveColorMode::Rgb,
            reason: "render.color_mode=rgb override",
        },
        RequestedColorMode::Ansi256 => RenderDecision {
            declared_support,
            requested,
            effective: EffectiveColorMode::Ansi256,
            reason: "render.color_mode=ansi256 override",
        },
        RequestedColorMode::Auto => {
            if term_program == "apple_terminal" {
                RenderDecision {
                    declared_support,
                    requested,
                    effective: EffectiveColorMode::Ansi256,
                    reason: "Apple Terminal uses ANSI-256 fallback in auto mode for color reliability",
                }
            } else if is_known_rgb_terminal(&term, &term_program) {
                RenderDecision {
                    declared_support,
                    requested,
                    effective: EffectiveColorMode::Rgb,
                    reason: "known RGB-capable terminal detected in auto mode",
                }
            } else {
                RenderDecision {
                    declared_support,
                    requested,
                    effective: EffectiveColorMode::Ansi256,
                    reason: "terminal is not on the known-safe RGB allowlist; using ANSI-256 fallback",
                }
            }
        }
    }
}

pub fn build_palette(config: &EditorConfig, mode: EffectiveColorMode) -> Palette {
    let theme_accent = map_theme_color(mode, config.theme.accent_rgb());
    let theme_warning = map_theme_color(mode, config.theme.warning_rgb());
    let theme_dim = map_theme_color(mode, config.theme.dim_rgb());
    let theme_bg_bar = map_theme_color(mode, config.theme.bg_bar_rgb());

    Palette {
        theme_accent,
        theme_warning,
        theme_dim,
        theme_bg_bar,
        editor_bg: map_builtin_color(mode, (18, 18, 18)),
        tilde_fg: map_builtin_color(mode, (60, 60, 60)),
        indent_guide_fg: map_builtin_color(mode, (45, 45, 45)),
        text_main: map_builtin_color(mode, (220, 220, 220)),
        lang_fg: map_builtin_color(mode, (130, 170, 150)),
        syntax_keyword_fg: map_builtin_color(mode, (199, 146, 234)),
        syntax_string_fg: map_builtin_color(mode, (195, 232, 141)),
        syntax_comment_fg: map_builtin_color(mode, (90, 90, 90)),
        syntax_number_fg: map_builtin_color(mode, (247, 140, 108)),
        syntax_type_fg: map_builtin_color(mode, (130, 170, 255)),
        syntax_attribute_fg: map_builtin_color(mode, (255, 203, 107)),
        search_match_bg: map_builtin_color(mode, (60, 60, 30)),
        search_current_fg: map_builtin_color(mode, (10, 10, 10)),
        selection_bg: map_builtin_color(mode, (50, 70, 90)),
        line_number_bg: map_builtin_color(mode, (10, 10, 10)),
        hints_sep_fg: map_builtin_color(mode, (45, 45, 45)),
        hints_warn_sep_fg: map_builtin_color(mode, (100, 60, 20)),
        overlay_bg: map_builtin_color(mode, (10, 10, 10)),
        overlay_version_fg: map_builtin_color(mode, (50, 50, 50)),
        overlay_section_fg: map_builtin_color(mode, (130, 130, 130)),
        overlay_desc_fg: map_builtin_color(mode, (190, 190, 190)),
        overlay_rule_fg: map_builtin_color(mode, (32, 32, 32)),
        overlay_footer_fg: map_builtin_color(mode, (50, 50, 50)),
        command_selected_fg: map_builtin_color(mode, (10, 10, 10)),
        command_separator_fg: map_builtin_color(mode, (50, 50, 50)),
        settings_label_fg: map_builtin_color(mode, (190, 190, 190)),
        settings_inactive_label_fg: map_builtin_color(mode, (150, 150, 150)),
        settings_check_on_fg: map_builtin_color(mode, (130, 200, 150)),
        settings_check_off_fg: map_builtin_color(mode, (70, 70, 70)),
        settings_dim_fg: map_builtin_color(mode, (60, 60, 60)),
        settings_footer_fg: map_builtin_color(mode, (50, 50, 50)),
        settings_rule_fg: map_builtin_color(mode, (28, 28, 28)),
    }
}

pub fn resolve_config_rgb(mode: EffectiveColorMode, rgb: [u8; 3]) -> Color {
    map_color(mode, (rgb[0], rgb[1], rgb[2]))
}

pub fn rgb_to_ansi256_index(rgb: (u8, u8, u8)) -> u8 {
    let mut best = 16u8;
    let mut best_distance = u32::MAX;

    for index in 16u8..=255u8 {
        let candidate = ansi256_rgb(index);
        let distance = color_distance(rgb, candidate);
        if distance < best_distance {
            best_distance = distance;
            best = index;
        }
    }

    best
}

fn is_known_rgb_terminal(term: &str, term_program: &str) -> bool {
    term_program == "wezterm"
        || term_program == "iterm.app"
        || term_program == "ghostty"
        || term_program == "warpterminal"
        || term_program == "rio"
        || term.contains("kitty")
        || term.contains("alacritty")
        || term.contains("ghostty")
        || term.contains("wezterm")
}

fn map_theme_color(mode: EffectiveColorMode, rgb: (u8, u8, u8)) -> Color {
    map_color(mode, rgb)
}

fn map_builtin_color(mode: EffectiveColorMode, rgb: (u8, u8, u8)) -> Color {
    map_color(mode, rgb)
}

fn map_color(mode: EffectiveColorMode, rgb: (u8, u8, u8)) -> Color {
    match mode {
        EffectiveColorMode::Rgb => Color::Rgb(rgb.0, rgb.1, rgb.2),
        EffectiveColorMode::Ansi256 => Color::Indexed(rgb_to_ansi256_index(rgb)),
    }
}

fn color_distance(a: (u8, u8, u8), b: (u8, u8, u8)) -> u32 {
    let dr = a.0 as i32 - b.0 as i32;
    let dg = a.1 as i32 - b.1 as i32;
    let db = a.2 as i32 - b.2 as i32;
    (dr * dr + dg * dg + db * db) as u32
}

fn ansi256_rgb(index: u8) -> (u8, u8, u8) {
    if (16..=231).contains(&index) {
        let i = index - 16;
        let r = i / 36;
        let g = (i % 36) / 6;
        let b = i % 6;
        (cube_value(r), cube_value(g), cube_value(b))
    } else {
        let level = 8 + (index - 232) * 10;
        (level, level, level)
    }
}

fn cube_value(component: u8) -> u8 {
    match component {
        0 => 0,
        1 => 95,
        2 => 135,
        3 => 175,
        4 => 215,
        _ => 255,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::EditorConfig;

    #[test]
    fn apple_terminal_auto_uses_ansi256() {
        let config = EditorConfig::default();
        let decision = resolve_render_decision(
            &config,
            &TerminalEnv {
                term: "xterm-256color".into(),
                colorterm: "truecolor".into(),
                term_program: "Apple_Terminal".into(),
            },
        );

        assert_eq!(decision.requested, RequestedColorMode::Auto);
        assert_eq!(decision.effective, EffectiveColorMode::Ansi256);
    }

    #[test]
    fn override_rgb_wins_on_apple_terminal() {
        let mut config = EditorConfig::default();
        config.render.color_mode = "rgb".into();
        let decision = resolve_render_decision(
            &config,
            &TerminalEnv {
                term: "xterm-256color".into(),
                colorterm: "truecolor".into(),
                term_program: "Apple_Terminal".into(),
            },
        );

        assert_eq!(decision.effective, EffectiveColorMode::Rgb);
        assert_eq!(decision.reason, "render.color_mode=rgb override");
    }

    #[test]
    fn unknown_terminal_auto_uses_ansi256() {
        let config = EditorConfig::default();
        let decision = resolve_render_decision(
            &config,
            &TerminalEnv {
                term: "xterm-256color".into(),
                colorterm: "truecolor".into(),
                term_program: "(not set)".into(),
            },
        );

        assert_eq!(decision.effective, EffectiveColorMode::Ansi256);
    }

    #[test]
    fn known_rgb_terminal_auto_uses_rgb() {
        let config = EditorConfig::default();
        let decision = resolve_render_decision(
            &config,
            &TerminalEnv {
                term: "xterm-kitty".into(),
                colorterm: "truecolor".into(),
                term_program: "(not set)".into(),
            },
        );

        assert_eq!(decision.effective, EffectiveColorMode::Rgb);
    }

    #[test]
    fn ansi256_palette_quantizes_theme_colors() {
        let config = EditorConfig::default();
        let palette = build_palette(&config, EffectiveColorMode::Ansi256);

        assert_eq!(
            palette.theme_bg_bar,
            Color::Indexed(rgb_to_ansi256_index((18, 18, 18)))
        );
        assert_eq!(
            palette.theme_warning,
            Color::Indexed(rgb_to_ansi256_index((255, 153, 68)))
        );
    }

    #[test]
    fn rgb_palette_preserves_rgb_values() {
        let config = EditorConfig::default();
        let palette = build_palette(&config, EffectiveColorMode::Rgb);

        assert_eq!(palette.theme_bg_bar, Color::Rgb(18, 18, 18));
        assert_eq!(palette.theme_warning, Color::Rgb(255, 153, 68));
    }
}
