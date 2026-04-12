mod app;
mod clipboard;
mod color;
mod command_alias;
mod commands;
mod config;
mod editor;
mod input;
mod shortcuts;
mod syntax;
mod ui;

use std::path::PathBuf;
use std::fmt::Write as _;
use clap::{CommandFactory, FromArgMatches, Parser};
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

    /// Print terminal environment diagnostics and resolved render mode, then exit.
    #[arg(long)]
    debug_env: bool,
}

fn main() -> Result<()> {
    let invoked_name = crate::command_alias::invoked_command_name();
    let args = parse_args(&invoked_name);
    if args.uninstall {
        return uninstall();
    }
    if args.debug_env {
        print_debug_env();
        return Ok(());
    }
    app::run(args.file)
}

fn parse_args(bin_name: &str) -> Args {
    let mut command = Args::command();
    command = command.bin_name(bin_name);
    match command.try_get_matches() {
        Ok(matches) => match Args::from_arg_matches(&matches) {
            Ok(args) => args,
            Err(err) => err.exit(),
        },
        Err(err) => err.exit(),
    }
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
    declared_color_support: &'static str,
    requested_color_mode: &'static str,
    effective_color_mode: &'static str,
    color_reason: &'static str,
    config_path: String,
    config_exists: bool,
    config_note: Option<String>,
    using_defaults: bool,
    theme: crate::config::ThemeSection,
}

fn collect_debug_env_report() -> DebugEnvReport {
    let env = crate::color::TerminalEnv::detect();
    let terminal_size = crossterm::terminal::size().ok();
    let config_path = crate::config::config_path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| crate::config::DISPLAY_CONFIG_PATH.to_string());
    let config_exists = crate::config::config_path()
        .is_some_and(|path| path.exists());
    let (loaded_config, config_note) = crate::config::load_existing_config();
    let using_defaults = loaded_config.is_none();
    let config = loaded_config.unwrap_or_default();
    let theme = config.theme.clone();
    let render_decision = crate::color::resolve_render_decision(&config, &env);

    DebugEnvReport {
        declared_color_support: render_decision.declared_support,
        requested_color_mode: render_decision.requested.as_str(),
        effective_color_mode: render_decision.effective.as_str(),
        color_reason: render_decision.reason,
        term: env.term,
        colorterm: env.colorterm,
        term_program: env.term_program,
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
    let _ = writeln!(out, "  Declared RGB : {}", report.declared_color_support);
    let _ = writeln!(out, "  Requested    : {}", report.requested_color_mode);
    let _ = writeln!(out, "  Effective    : {}", report.effective_color_mode);
    let _ = writeln!(out, "  Decision     : {}", report.color_reason);
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
    let _ = writeln!(out, "  Editor.bg    : built-in {}", crate::color::EDITOR_BG_HEX);
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
        "  Hint: chuch-term uses the effective render mode above, not just COLORTERM."
    );
    let _ = writeln!(
        out,
        "        Use [render].color_mode = \"rgb\" only on terminals that actually render RGB correctly."
    );
    let _ = writeln!(out, "        For safest compatibility, keep color_mode = \"auto\".");
    out
}

fn uninstall() -> Result<()> {
    let current_exe = std::env::current_exe()?;
    let (existing_config, _) = config::load_existing_config();
    if let Some(config) = existing_config.as_ref()
        && let Some(message) = crate::command_alias::cleanup_uninstall_alias(&config.command, &current_exe)?
    {
        println!("{message}");
    }

    for config_dir in [config::config_dir_path(), config::legacy_config_dir()]
        .into_iter()
        .flatten()
    {
        if config_dir.exists() {
            std::fs::remove_dir_all(&config_dir)?;
            println!("Removed config: {}", config_dir.display());
        }
    }

    // Binary (safe to delete a running executable on Unix — OS keeps the inode alive).
    std::fs::remove_file(&current_exe)?;
    println!("Removed binary: {}", current_exe.display());
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
            declared_color_support: "RGB announced by COLORTERM",
            requested_color_mode: "auto",
            effective_color_mode: "ansi256",
            color_reason: "Apple Terminal uses ANSI-256 fallback in auto mode for color reliability",
            config_path: "/tmp/.config/chuch/config.toml".to_string(),
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
        assert!(output.contains("/tmp/.config/chuch/config.toml"));
        assert!(output.contains("Theme.bg_bar"));
        assert!(output.contains("#121212"));
        assert!(output.contains("Editor.bg"));
        assert!(output.contains("Theme scope"));
        assert!(output.contains("Requested"));
        assert!(output.contains("Effective"));
        assert!(output.contains("Decision"));
    }

    #[test]
    fn debug_env_report_explains_effective_render_mode() {
        let output = format_debug_env_report(&sample_report());

        assert!(output.contains("uses the effective render mode above"));
        assert!(output.contains("Apple Terminal uses ANSI-256 fallback"));
    }

    #[test]
    fn collect_debug_env_report_tracks_auto_fallback() {
        let config = crate::config::EditorConfig::default();
        let decision = crate::color::resolve_render_decision(
            &config,
            &crate::color::TerminalEnv {
                term: "xterm-256color".into(),
                colorterm: "TRUECOLOR".into(),
                term_program: "Apple_Terminal".into(),
            },
        );

        assert_eq!(decision.requested.as_str(), "auto");
        assert_eq!(decision.effective.as_str(), "ansi256");
        assert_eq!(decision.declared_support, "RGB announced by COLORTERM");
    }

    #[test]
    fn parse_args_uses_invoked_bin_name_in_help() {
        let usage = Args::command().bin_name("cct").render_usage().to_string();

        assert!(usage.contains("cct"));
        assert!(!usage.contains("chuch-term"));
    }
}
