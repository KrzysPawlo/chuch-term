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
}

fn main() -> Result<()> {
    let args = Args::parse();
    app::run(args.file)
}
