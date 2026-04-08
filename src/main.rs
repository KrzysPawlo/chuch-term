mod app;
mod clipboard;
mod commands;
mod config;
mod editor;
mod input;
mod syntax;
mod ui;

use std::path::PathBuf;
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
    use std::env;
    println!("chuch-term {} — environment diagnostics", env!("CARGO_PKG_VERSION"));
    println!();
    println!("  TERM         : {}", env::var("TERM").unwrap_or_else(|_| "(not set)".into()));
    println!("  COLORTERM    : {}", env::var("COLORTERM").unwrap_or_else(|_| "(not set)".into()));
    println!("  TERM_PROGRAM : {}", env::var("TERM_PROGRAM").unwrap_or_else(|_| "(not set)".into()));
    if let Ok((w, h)) = crossterm::terminal::size() {
        println!("  Terminal size: {}×{}", w, h);
    } else {
        println!("  Terminal size: (unable to detect)");
    }
    println!("  OS           : {}", std::env::consts::OS);
    println!("  Arch         : {}", std::env::consts::ARCH);
    let color_depth = match env::var("COLORTERM").as_deref() {
        Ok("truecolor") | Ok("24bit") => "truecolor (24-bit RGB) ✓",
        _ => match env::var("TERM").as_deref() {
            Ok(t) if t.contains("256color") => "256-color (truecolor NOT confirmed — colours may look wrong)",
            _ => "basic / unknown — colours will likely render incorrectly",
        },
    };
    println!("  Color support: {}", color_depth);
    println!();
    println!("  Hint: add the following to your shell profile for best results:");
    println!("        export COLORTERM=truecolor");
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
