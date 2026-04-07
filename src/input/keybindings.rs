use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::editor::EditorMode;

/// All actions that the editor can perform, derived from input events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppAction {
    Noop,

    // App lifecycle
    /// Ctrl+Q in Normal mode — enter ConfirmQuit if dirty, else quit directly.
    RequestQuit,
    /// Ctrl+Q in ConfirmQuit mode — force quit without saving.
    ForceQuit,
    /// Esc in ConfirmQuit mode — cancel and return to Normal.
    CancelQuit,
    /// Ctrl+S — save (and if in ConfirmQuit mode, save then quit).
    Save,

    // Help overlay
    /// Ctrl+H — open full-screen help overlay.
    ShowHelp,
    /// Esc / Ctrl+H / Space / Enter while in Help mode — close overlay.
    CloseHelp,

    // Text editing
    InsertChar(char),
    DeleteBefore,   // Backspace
    DeleteAt,       // Delete
    InsertNewline,  // Enter

    // Cursor movement
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Home,
    End,
    PageUp,
    PageDown,

    // Undo / Redo
    Undo,
    Redo,

    // Search
    StartSearch,
    CloseSearch,
    SearchChar(char),
    SearchBackspace,
    SearchNext,
    SearchPrev,

    // Go-to-line
    OpenGoToLine,
    GoToLineChar(char),
    GoToLineSubmit,
    CloseGoToLine,
    GoToLineBackspace,

    // Line numbers
    ToggleLineNumbers,

    // Selection
    SelectAll,
    Copy,
    Cut,
    Paste,
    ClearSelection,
    ShiftUp,
    ShiftDown,
    ShiftLeft,
    ShiftRight,
    ShiftHome,
    ShiftEnd,

    // Command palette
    OpenCommandPalette,
    ClosePalette,
    PaletteChar(char),
    PaletteBackspace,
    PaletteUp,
    PaletteDown,
    PaletteSubmit,
    OpenConfig,

    // Navigation
    /// Ctrl+O — return to previous buffer (after OpenConfig).
    GoBackBuffer,

    // Find & Replace
    StartReplace,
    ReplaceChar(char),
    ReplaceBackspace,
    ReplaceSubmit,
    ReplaceAll,
    CloseReplace,

    // Search improvements
    SearchSelect,         // Enter in search mode — selects current match
    ToggleCaseSensitive,  // Ctrl+I in search/replace modes

    // Case change
    UppercaseSelection,   // Alt+U
    LowercaseSelection,   // Alt+L
}

/// Map a raw key event + current editor mode to a semantic AppAction.
pub fn map_key(event: KeyEvent, mode: EditorMode) -> AppAction {
    let ctrl = event.modifiers.contains(KeyModifiers::CONTROL);
    let shift = event.modifiers.contains(KeyModifiers::SHIFT);

    match mode {
        // ── Help overlay — any key closes it ──────────────────────────
        EditorMode::Help => match event.code {
            KeyCode::Esc => AppAction::CloseHelp,
            KeyCode::Char('h') if ctrl => AppAction::CloseHelp,
            KeyCode::Char(' ') => AppAction::CloseHelp,
            KeyCode::Enter => AppAction::CloseHelp,
            _ => AppAction::Noop,
        },

        // ── Confirm-quit dialog ───────────────────────────────────────
        EditorMode::ConfirmQuit => match event.code {
            KeyCode::Char('q') if ctrl => AppAction::ForceQuit,
            KeyCode::Char('s') if ctrl => AppAction::Save,
            KeyCode::Char('h') if ctrl => AppAction::ShowHelp,
            KeyCode::Esc => AppAction::CancelQuit,
            _ => AppAction::Noop,
        },

        // ── Search mode ───────────────────────────────────────────────
        EditorMode::Search => {
            if ctrl {
                match event.code {
                    KeyCode::Char('n') => AppAction::SearchNext,
                    KeyCode::Char('p') => AppAction::SearchPrev,
                    KeyCode::Char('f') => AppAction::CloseSearch,
                    KeyCode::Char('r') => AppAction::StartReplace,
                    KeyCode::Char('i') => AppAction::ToggleCaseSensitive,
                    _ => AppAction::Noop,
                }
            } else {
                match event.code {
                    KeyCode::Esc => AppAction::CloseSearch,
                    KeyCode::Backspace => AppAction::SearchBackspace,
                    KeyCode::Enter => AppAction::SearchSelect,
                    KeyCode::Char(c) => AppAction::SearchChar(c),
                    _ => AppAction::Noop,
                }
            }
        }

        // ── Replace mode ──────────────────────────────────────────────
        EditorMode::Replace => {
            if ctrl {
                match event.code {
                    KeyCode::Char('a') => AppAction::ReplaceAll,
                    KeyCode::Char('n') => AppAction::SearchNext,
                    KeyCode::Char('i') => AppAction::ToggleCaseSensitive,
                    _ => AppAction::Noop,
                }
            } else {
                match event.code {
                    KeyCode::Esc => AppAction::CloseReplace,
                    KeyCode::Enter => AppAction::ReplaceSubmit,
                    KeyCode::Backspace => AppAction::ReplaceBackspace,
                    KeyCode::Char(c) => AppAction::ReplaceChar(c),
                    _ => AppAction::Noop,
                }
            }
        }

        // ── Go-to-line mode ───────────────────────────────────────────
        EditorMode::GoToLine => match event.code {
            KeyCode::Esc => AppAction::CloseGoToLine,
            KeyCode::Enter => AppAction::GoToLineSubmit,
            KeyCode::Backspace => AppAction::GoToLineBackspace,
            KeyCode::Char(c) if c.is_ascii_digit() => AppAction::GoToLineChar(c),
            _ => AppAction::Noop,
        },

        // ── Command palette mode ──────────────────────────────────────
        EditorMode::CommandPalette => {
            if ctrl {
                match event.code {
                    KeyCode::Char('p') => AppAction::ClosePalette,
                    _ => AppAction::Noop,
                }
            } else {
                match event.code {
                    KeyCode::Esc => AppAction::ClosePalette,
                    KeyCode::Enter => AppAction::PaletteSubmit,
                    KeyCode::Backspace => AppAction::PaletteBackspace,
                    KeyCode::Up => AppAction::PaletteUp,
                    KeyCode::Down => AppAction::PaletteDown,
                    KeyCode::Char(c) => AppAction::PaletteChar(c),
                    _ => AppAction::Noop,
                }
            }
        }

        // ── Normal editing ────────────────────────────────────────────
        EditorMode::Normal => {
            let alt = event.modifiers.contains(KeyModifiers::ALT);
            if ctrl {
                match event.code {
                    KeyCode::Char('s') => AppAction::Save,
                    KeyCode::Char('q') => AppAction::RequestQuit,
                    KeyCode::Char('h') => AppAction::ShowHelp,
                    KeyCode::Char('z') => AppAction::Undo,
                    KeyCode::Char('y') => AppAction::Redo,
                    KeyCode::Char('f') => AppAction::StartSearch,
                    KeyCode::Char('g') => AppAction::OpenGoToLine,
                    KeyCode::Char('l') => AppAction::ToggleLineNumbers,
                    KeyCode::Char('p') => AppAction::OpenCommandPalette,
                    KeyCode::Char('a') => AppAction::SelectAll,
                    KeyCode::Char('c') => AppAction::Copy,
                    KeyCode::Char('x') => AppAction::Cut,
                    KeyCode::Char('v') => AppAction::Paste,
                    KeyCode::Char('n') => AppAction::SearchNext,
                    KeyCode::Char('o') => AppAction::GoBackBuffer,
                    KeyCode::Char('r') => AppAction::StartReplace,
                    _ => AppAction::Noop,
                }
            } else if shift {
                match event.code {
                    KeyCode::Up => AppAction::ShiftUp,
                    KeyCode::Down => AppAction::ShiftDown,
                    KeyCode::Left => AppAction::ShiftLeft,
                    KeyCode::Right => AppAction::ShiftRight,
                    KeyCode::Home => AppAction::ShiftHome,
                    KeyCode::End => AppAction::ShiftEnd,
                    KeyCode::Char(c) => AppAction::InsertChar(c),
                    _ => AppAction::Noop,
                }
            } else if alt {
                match event.code {
                    KeyCode::Char('u') => AppAction::UppercaseSelection,
                    KeyCode::Char('l') => AppAction::LowercaseSelection,
                    _ => AppAction::Noop,
                }
            } else {
                match event.code {
                    KeyCode::Up => AppAction::MoveUp,
                    KeyCode::Down => AppAction::MoveDown,
                    KeyCode::Left => AppAction::MoveLeft,
                    KeyCode::Right => AppAction::MoveRight,
                    KeyCode::Home => AppAction::Home,
                    KeyCode::End => AppAction::End,
                    KeyCode::PageUp => AppAction::PageUp,
                    KeyCode::PageDown => AppAction::PageDown,
                    KeyCode::Backspace => AppAction::DeleteBefore,
                    KeyCode::Delete => AppAction::DeleteAt,
                    KeyCode::Enter => AppAction::InsertNewline,
                    KeyCode::Tab => AppAction::InsertChar('\t'),
                    KeyCode::Char(c) => AppAction::InsertChar(c),
                    KeyCode::Esc => AppAction::ClearSelection,
                    _ => AppAction::Noop,
                }
            }
        }
    }
}
