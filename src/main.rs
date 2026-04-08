mod app;
mod clipboard;
mod commands;
mod config;
mod editor;
mod input;
mod syntax;
mod ui;

use std::path::PathBuf;
use std::fmt::Write as _;
use clap::Parser;
use anyhow::Result;

/// chuch-term — a minimal, beautiful terminal text editor.
#[derive(Parser, Debug)]
#[command(name = "chuch-term", version, about)]
struct Args {
    /// File to open. If omitted, opens an empty buffer.
    file: Option<PathBuf>,

    /// Remove the chuch-term binary and all configuration, then exit.
    #[arg(long)]
    uninstall: bool,

    /// Print terminal environment diagnostics (TERM, COLORTERM, color depth) and exit.
    #[arg(long)]
    debug_env: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    if args.uninstall {
        return uninstall();
    }
    if args.debug_env {
        print_debug_env();
        return Ok(());
    }
    app::run(args.file)
}

fn print_debug_env() {
    print!("{}", format_debug_env_report(&collect_debug_env_report()));
}

#[derive(Debug, Clone)]
struct DebugEnvReport {
    term: String,
    colorterm: String,
    term_program: String,
    terminal_size: Option<(u16, u16)>,
    os: &'static str,
    arch: &'static str,
    color_support: &'static str,
    config_path: String,
    config_exists: bool,
    config_note: Option<String>,
    using_defaults: bool,
    theme: crate::config::ThemeSection,
}

fn collect_debug_env_report() -> DebugEnvReport {
    use std::env;

    let term = env::var("TERM").unwrap_or_else(|_| "(not set)".into());
    let colorterm = env::var("COLORTERM").unwrap_or_else(|_| "(not set)".into());
    let term_program = env::var("TERM_PROGRAM").unwrap_or_else(|_| "(not set)".into());
    let terminal_size = crossterm::terminal::size().ok();
    let config_path = crate::config::config_path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| crate::config::DISPLAY_CONFIG_PATH.to_string());
    let config_exists = crate::config::config_path()
        .is_some_and(|path| path.exists());
    let (loaded_config, config_note) = crate::config::load_existing_config();
    let using_defaults = loaded_config.is_none();
    let theme = loaded_config
        .map(|cfg| cfg.theme)
        .unwrap_or_default();

    DebugEnvReport {
        color_support: detect_color_support(&term, &colorterm),
        term,
        colorterm,
        term_program,
        terminal_size,
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
        config_path,
        config_exists,
        config_note,
        using_defaults,
        theme,
    }
}

fn detect_color_support(term: &str, colorterm: &str) -> &'static str {
    let colorterm = colorterm.trim().to_ascii_lowercase();
    let term = term.trim().to_ascii_lowercase();

    match colorterm.as_str() {
        "truecolor" | "24bit" => "truecolor declared by COLORTERM",
        _ if term.contains("256color") => {
            "256-color terminal reported; truecolor not confirmed, so colors may differ"
        }
        _ => "basic / unknown terminal colors; UI may render incorrectly",
    }
}

fn format_debug_env_report(report: &DebugEnvReport) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "chuch-term {} — environment diagnostics",
        env!("CARGO_PKG_VERSION")
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "  TERM         : {}", report.term);
    let _ = writeln!(out, "  COLORTERM    : {}", report.colorterm);
    let _ = writeln!(out, "  TERM_PROGRAM : {}", report.term_program);
    if let Some((w, h)) = report.terminal_size {
        let _ = writeln!(out, "  Terminal size: {}×{}", w, h);
    } else {
        let _ = writeln!(out, "  Terminal size: (unable to detect)");
    }
    let _ = writeln!(out, "  OS           : {}", report.os);
    let _ = writeln!(out, "  Arch         : {}", report.arch);
    let _ = writeln!(out, "  Color support: {}", report.color_support);
    let _ = writeln!(out);
    let _ = writeln!(out, "  Config path  : {}", report.config_path);
    let _ = writeln!(
        out,
        "  Config exists: {}",
        if report.config_exists { "yes" } else { "no" }
    );
    let _ = writeln!(
        out,
        "  Config source: {}",
        if report.using_defaults {
            "defaults"
        } else {
            "config file"
        }
    );
    let _ = writeln!(out, "  Theme.accent : {}", report.theme.accent);
    let _ = writeln!(out, "  Theme.warning: {}", report.theme.warning);
    let _ = writeln!(out, "  Theme.dim    : {}", report.theme.dim);
    let _ = writeln!(out, "  Theme.bg_bar : {}", report.theme.bg_bar);
    let _ = writeln!(out, "  Editor.bg    : built-in #121212");
    let _ = writeln!(
        out,
        "  Theme scope  : bg_bar applies to bottom bars; the editor background is fixed"
    );
    if let Some(note) = &report.config_note {
        let _ = writeln!(out, "  Config note  : {note}");
    }
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "  Hint: chuch-term expects a truecolor terminal and the active config above."
    );
    let _ = writeln!(
        out,
        "        If colors look wrong and COLORTERM is not set to truecolor, add:"
    );
    let _ = writeln!(out, "        export COLORTERM=truecolor");
    out
}

fn uninstall() -> Result<()> {
    // Config directory: ~/.config/chuch-term/
    if let Some(config_file) = config::config_path() {
        if let Some(config_dir) = config_file.parent() {
            if config_dir.exists() {
                std::fs::remove_dir_all(config_dir)?;
                println!("Removed config: {}", config_dir.display());
            }
        }
    }

    // Binary (safe to delete a running executable on Unix — OS keeps the inode alive).
    let exe = std::env::current_exe()?;
    std::fs::remove_file(&exe)?;
    println!("Removed binary: {}", exe.display());
    println!("chuch-term uninstalled.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_report() -> DebugEnvReport {
        DebugEnvReport {
            term: "xterm-256color".to_string(),
            colorterm: "truecolor".to_string(),
            term_program: "Apple_Terminal".to_string(),
            terminal_size: Some((120, 40)),
            os: "macos",
            arch: "aarch64",
            color_support: "truecolor declared by COLORTERM",
            config_path: "/tmp/.config/chuch-term/config.toml".to_string(),
            config_exists: true,
            config_note: None,
            using_defaults: false,
            theme: crate::config::ThemeSection::default(),
        }
    }

    #[test]
    fn debug_env_report_includes_config_path_and_theme() {
        let output = format_debug_env_report(&sample_report());

        assert!(output.contains("Config path"));
        assert!(output.contains("/tmp/.config/chuch-term/config.toml"));
        assert!(output.contains("Theme.bg_bar"));
        assert!(output.contains("#121212"));
        assert!(output.contains("Editor.bg"));
        assert!(output.contains("Theme scope"));
    }

    #[test]
    fn debug_env_report_explains_truecolor_without_overclaiming() {
        let output = format_debug_env_report(&sample_report());

        assert!(output.contains("expects a truecolor terminal"));
        assert!(!output.contains("best results"));
    }

    #[test]
    fn detect_color_support_handles_non_truecolor_term() {
        assert_eq!(
            detect_color_support("xterm-256color", "(not set)"),
            "256-color terminal reported; truecolor not confirmed, so colors may differ"
        );
    }

    #[test]
    fn detect_color_support_is_case_insensitive() {
        assert_eq!(
            detect_color_support("XTERM-256COLOR", "TRUECOLOR"),
            "truecolor declared by COLORTERM"
        );
    }
}
