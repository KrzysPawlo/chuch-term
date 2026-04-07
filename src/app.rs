use std::io::{self, stdout};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    event,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::editor::EditorState;
use crate::input::handle_event;
use crate::ui::draw;

/// Main application entry point.
pub fn run(file_path: Option<PathBuf>) -> Result<()> {
    // ── Build editor state ─────────────────────────────────────────────
    let mut state = match file_path {
        Some(ref path) => EditorState::from_file(path)?,
        None => EditorState::new_empty(),
    };

    // ── Set up terminal ────────────────────────────────────────────────
    install_panic_hook();
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // ── Event loop ─────────────────────────────────────────────────────
    let result = event_loop(&mut terminal, &mut state);

    // ── Tear down terminal (always, even on error) ─────────────────────
    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    let _ = terminal.show_cursor();

    result
}

fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut EditorState,
) -> Result<()> {
    let mut last_config_check = Instant::now();
    const CONFIG_CHECK_INTERVAL: Duration = Duration::from_secs(2);

    loop {
        // Update viewport scroll before drawing.
        let viewport_height = terminal.size()?.height.saturating_sub(2) as usize;
        state.viewport_height = viewport_height;
        state
            .viewport
            .scroll_to_cursor(&state.cursor, viewport_height);

        // Draw the frame.
        terminal.draw(|frame| draw(frame, state))?;

        // Wait for next event (with a timeout so we can do periodic checks).
        if event::poll(Duration::from_millis(500))? {
            let ev = event::read()?;
            handle_event(ev, state)?;
        }

        // Hot-reload config every ~2 seconds.
        if last_config_check.elapsed() >= CONFIG_CHECK_INTERVAL {
            last_config_check = Instant::now();
            let new_mtime = crate::config::config_mtime();
            if new_mtime != state.config_mtime {
                let (new_config, msg) = crate::config::load_config();
                state.apply_config(new_config);
                state.config_mtime = crate::config::config_mtime();
                if let Some(m) = msg {
                    state.status_message = Some(m);
                }
            }
        }

        if state.should_quit {
            break;
        }
    }
    Ok(())
}

/// Install a panic hook that restores the terminal before printing the panic message.
fn install_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));
}
