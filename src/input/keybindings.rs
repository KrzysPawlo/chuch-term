use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::editor::{EditorMode, EditorState};
use crate::shortcuts::{capture_token, KeyToken, ShortcutAction, ShortcutContext};

/// All actions that the editor can perform, derived from input events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppAction {
    Noop,

    // App lifecycle
    RequestQuit,
    ForceQuit,
    CancelQuit,
    Save,

    // Help overlay
    ShowHelp,
    CloseHelp,

    // Text editing
    InsertChar(char),
    DeleteBefore,
    DeleteAt,
    InsertNewline,

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
    ShiftWordLeft,
    ShiftWordRight,

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
    GoBackBuffer,

    // Find & Replace
    StartReplace,
    ReplaceChar(char),
    ReplaceBackspace,
    ReplaceSubmit,
    ReplaceAll,
    CloseReplace,

    // Search improvements
    SearchSelect,
    ToggleCaseSensitive,

    // Case change
    UppercaseSelection,
    LowercaseSelection,

    // Word navigation
    WordLeft,
    WordRight,

    // Delete word
    DeleteWordBefore,
    DeleteWordAfter,

    // Save-as
    SaveAsChar(char),
    SaveAsBackspace,
    SaveAsSubmit,
    CancelSaveAs,

    // Duplicate line
    DuplicateLine,

    // Settings overlay
    OpenSettings,
    CloseSettings,
    SettingsUp,
    SettingsDown,
    SettingsToggle,
    SettingsAdjust(i8),

    // Keybindings overlay
    CloseKeybindings,
    KeybindingsUp,
    KeybindingsDown,
    StartKeybindingCapture,
    CancelKeybindingCapture,
    CaptureKeyToken(KeyToken),
    ResetSelectedKeybinding,
    ResetShortcutOverrides,

    // Command alias overlay
    CloseCommandAlias,
    CommandAliasChar(char),
    CommandAliasBackspace,
    CommandAliasSubmit,
}

/// Map a raw key event + current editor state to a semantic AppAction.
pub fn map_key(event: KeyEvent, state: &EditorState) -> AppAction {
    match state.mode {
        EditorMode::Help => map_help_key(event, state),
        EditorMode::ConfirmQuit => map_confirm_quit_key(event, state),
        EditorMode::Search => map_search_key(event, state),
        EditorMode::Replace => map_replace_key(event, state),
        EditorMode::GoToLine => map_goto_key(event),
        EditorMode::CommandPalette => map_palette_key(event, state),
        EditorMode::Settings => map_settings_key(event),
        EditorMode::Keybindings => map_keybindings_overlay_key(event, state),
        EditorMode::CommandAlias => map_command_alias_key(event),
        EditorMode::SaveAs => map_saveas_key(event),
        EditorMode::Normal => map_normal_key(event, state),
    }
}

fn map_help_key(event: KeyEvent, state: &EditorState) -> AppAction {
    match event.code {
        KeyCode::Esc | KeyCode::Char(' ') | KeyCode::Enter => AppAction::CloseHelp,
        _ => match state
            .active_shortcuts
            .resolve_action(ShortcutContext::Help, event)
        {
            Some(ShortcutAction::Help) => AppAction::CloseHelp,
            _ => AppAction::Noop,
        },
    }
}

fn map_confirm_quit_key(event: KeyEvent, state: &EditorState) -> AppAction {
    if event.code == KeyCode::Esc {
        return AppAction::CancelQuit;
    }
    match state
        .active_shortcuts
        .resolve_action(ShortcutContext::ConfirmQuit, event)
    {
        Some(ShortcutAction::Quit) => AppAction::ForceQuit,
        Some(ShortcutAction::Save) => AppAction::Save,
        Some(ShortcutAction::Help) => AppAction::ShowHelp,
        _ => AppAction::Noop,
    }
}

fn map_search_key(event: KeyEvent, state: &EditorState) -> AppAction {
    if let Some(action) = state
        .active_shortcuts
        .resolve_action(ShortcutContext::Search, event)
    {
        return match action {
            ShortcutAction::SearchNext => AppAction::SearchNext,
            ShortcutAction::SearchPrev => AppAction::SearchPrev,
            ShortcutAction::Search => AppAction::CloseSearch,
            ShortcutAction::Replace => AppAction::StartReplace,
            ShortcutAction::ToggleCaseSensitive => AppAction::ToggleCaseSensitive,
            _ => AppAction::Noop,
        };
    }

    match event.code {
        KeyCode::Esc => AppAction::CloseSearch,
        KeyCode::Backspace => AppAction::SearchBackspace,
        KeyCode::Enter => AppAction::SearchSelect,
        KeyCode::Char(c) => AppAction::SearchChar(c),
        _ => AppAction::Noop,
    }
}

fn map_replace_key(event: KeyEvent, state: &EditorState) -> AppAction {
    if let Some(action) = state
        .active_shortcuts
        .resolve_action(ShortcutContext::Replace, event)
    {
        return match action {
            ShortcutAction::ReplaceAll => AppAction::ReplaceAll,
            ShortcutAction::SearchNext => AppAction::SearchNext,
            ShortcutAction::ToggleCaseSensitive => AppAction::ToggleCaseSensitive,
            _ => AppAction::Noop,
        };
    }

    match event.code {
        KeyCode::Esc => AppAction::CloseReplace,
        KeyCode::Enter => AppAction::ReplaceSubmit,
        KeyCode::Backspace => AppAction::ReplaceBackspace,
        KeyCode::Char(c) => AppAction::ReplaceChar(c),
        _ => AppAction::Noop,
    }
}

fn map_goto_key(event: KeyEvent) -> AppAction {
    match event.code {
        KeyCode::Esc => AppAction::CloseGoToLine,
        KeyCode::Enter => AppAction::GoToLineSubmit,
        KeyCode::Backspace => AppAction::GoToLineBackspace,
        KeyCode::Char(c) if c.is_ascii_digit() => AppAction::GoToLineChar(c),
        _ => AppAction::Noop,
    }
}

fn map_palette_key(event: KeyEvent, state: &EditorState) -> AppAction {
    if let Some(ShortcutAction::Palette) = state
        .active_shortcuts
        .resolve_action(ShortcutContext::CommandPalette, event)
    {
        return AppAction::ClosePalette;
    }

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

fn map_settings_key(event: KeyEvent) -> AppAction {
    match event.code {
        KeyCode::Esc => AppAction::CloseSettings,
        KeyCode::Up => AppAction::SettingsUp,
        KeyCode::Down => AppAction::SettingsDown,
        KeyCode::Enter | KeyCode::Char(' ') => AppAction::SettingsToggle,
        KeyCode::Left => AppAction::SettingsAdjust(-1),
        KeyCode::Right => AppAction::SettingsAdjust(1),
        _ => AppAction::Noop,
    }
}

fn map_keybindings_overlay_key(event: KeyEvent, state: &EditorState) -> AppAction {
    if state.keybinding_capture {
        return match event.code {
            KeyCode::Esc => AppAction::CancelKeybindingCapture,
            _ => match capture_token(event) {
                Some(token) => AppAction::CaptureKeyToken(token),
                None => AppAction::Noop,
            },
        };
    }

    match event.code {
        KeyCode::Esc => AppAction::CloseKeybindings,
        KeyCode::Up => AppAction::KeybindingsUp,
        KeyCode::Down => AppAction::KeybindingsDown,
        KeyCode::Enter | KeyCode::Char(' ') => AppAction::StartKeybindingCapture,
        KeyCode::Backspace | KeyCode::Delete => AppAction::ResetSelectedKeybinding,
        KeyCode::Char('r') if event.modifiers == KeyModifiers::CONTROL => AppAction::ResetShortcutOverrides,
        _ => AppAction::Noop,
    }
}

fn map_command_alias_key(event: KeyEvent) -> AppAction {
    match event.code {
        KeyCode::Esc => AppAction::CloseCommandAlias,
        KeyCode::Enter => AppAction::CommandAliasSubmit,
        KeyCode::Backspace => AppAction::CommandAliasBackspace,
        KeyCode::Char(c)
            if c.is_ascii_alphanumeric() || matches!(c, '_' | '-') =>
        {
            AppAction::CommandAliasChar(c.to_ascii_lowercase())
        }
        _ => AppAction::Noop,
    }
}

fn map_saveas_key(event: KeyEvent) -> AppAction {
    match event.code {
        KeyCode::Esc => AppAction::CancelSaveAs,
        KeyCode::Enter => AppAction::SaveAsSubmit,
        KeyCode::Backspace => AppAction::SaveAsBackspace,
        KeyCode::Char(c) => AppAction::SaveAsChar(c),
        _ => AppAction::Noop,
    }
}

fn map_normal_key(event: KeyEvent, state: &EditorState) -> AppAction {
    if matches_shift_word_motion(event, state, KeyCode::Left) {
        return AppAction::ShiftWordLeft;
    }
    if matches_shift_word_motion(event, state, KeyCode::Right) {
        return AppAction::ShiftWordRight;
    }

    if let Some(action) = state
        .active_shortcuts
        .resolve_action(ShortcutContext::Normal, event)
    {
        return map_normal_shortcut(action);
    }

    let shift_only = event.modifiers == KeyModifiers::SHIFT;
    if shift_only {
        return match event.code {
            KeyCode::Up => AppAction::ShiftUp,
            KeyCode::Down => AppAction::ShiftDown,
            KeyCode::Left => AppAction::ShiftLeft,
            KeyCode::Right => AppAction::ShiftRight,
            KeyCode::Home => AppAction::ShiftHome,
            KeyCode::End => AppAction::ShiftEnd,
            KeyCode::Char(c) => AppAction::InsertChar(c),
            _ => AppAction::Noop,
        };
    }

    if !event.modifiers.is_empty() {
        return AppAction::Noop;
    }

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

fn matches_shift_word_motion(event: KeyEvent, state: &EditorState, code: KeyCode) -> bool {
    let expected = state.active_shortcuts.profile().modifier() | KeyModifiers::SHIFT;
    event.modifiers == expected
        && match code {
            KeyCode::Left => {
                event.code == KeyCode::Left
                    && state.active_shortcuts.token_for(ShortcutAction::WordLeft) == KeyToken::Left
            }
            KeyCode::Right => {
                event.code == KeyCode::Right
                    && state.active_shortcuts.token_for(ShortcutAction::WordRight) == KeyToken::Right
            }
            _ => false,
        }
}

fn map_normal_shortcut(action: ShortcutAction) -> AppAction {
    match action {
        ShortcutAction::Save => AppAction::Save,
        ShortcutAction::Quit => AppAction::RequestQuit,
        ShortcutAction::Help => AppAction::ShowHelp,
        ShortcutAction::Undo => AppAction::Undo,
        ShortcutAction::Redo => AppAction::Redo,
        ShortcutAction::Search => AppAction::StartSearch,
        ShortcutAction::SearchNext => AppAction::SearchNext,
        ShortcutAction::GoToLine => AppAction::OpenGoToLine,
        ShortcutAction::ToggleLineNumbers => AppAction::ToggleLineNumbers,
        ShortcutAction::Palette => AppAction::OpenCommandPalette,
        ShortcutAction::SelectAll => AppAction::SelectAll,
        ShortcutAction::Copy => AppAction::Copy,
        ShortcutAction::Cut => AppAction::Cut,
        ShortcutAction::Paste => AppAction::Paste,
        ShortcutAction::GoBackBuffer => AppAction::GoBackBuffer,
        ShortcutAction::Replace => AppAction::StartReplace,
        ShortcutAction::DuplicateLine => AppAction::DuplicateLine,
        ShortcutAction::Settings => AppAction::OpenSettings,
        ShortcutAction::DeleteWordBefore => AppAction::DeleteWordBefore,
        ShortcutAction::DeleteWordAfter => AppAction::DeleteWordAfter,
        ShortcutAction::WordLeft => AppAction::WordLeft,
        ShortcutAction::WordRight => AppAction::WordRight,
        ShortcutAction::UppercaseSelection => AppAction::UppercaseSelection,
        ShortcutAction::LowercaseSelection => AppAction::LowercaseSelection,
        _ => AppAction::Noop,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shortcuts::ShortcutProfile;

    #[test]
    fn ctrl_profile_opens_settings_with_ctrl_t() {
        let state = EditorState::new_empty();
        let action = map_key(
            KeyEvent::new(KeyCode::Char('t'), KeyModifiers::CONTROL),
            &state,
        );

        assert_eq!(action, AppAction::OpenSettings);
    }

    #[test]
    fn alt_profile_opens_settings_with_alt_comma() {
        let mut state = EditorState::new_empty();
        let mut config = state.config.clone();
        config.shortcuts.profile = ShortcutProfile::Alt;
        state.apply_config(config);

        let action = map_key(
            KeyEvent::new(KeyCode::Char(','), KeyModifiers::ALT),
            &state,
        );

        assert_eq!(action, AppAction::OpenSettings);
    }
}
