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
}

fn main() -> Result<()> {
    let args = Args::parse();
    if args.uninstall {
        return uninstall();
    }
    app::run(args.file)
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
