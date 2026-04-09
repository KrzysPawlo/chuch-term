use crate::input::keybindings::AppAction;
use crate::shortcuts::ShortcutAction;

pub struct PaletteCommand {
    pub name: &'static str,
    pub shortcut: Option<ShortcutAction>,
    pub action: AppAction,
    pub description: &'static str,
}

pub static COMMANDS: &[PaletteCommand] = &[
    PaletteCommand {
        name: "save",
        shortcut: Some(ShortcutAction::Save),
        action: AppAction::Save,
        description: "Save current file",
    },
    PaletteCommand {
        name: "quit",
        shortcut: Some(ShortcutAction::Quit),
        action: AppAction::RequestQuit,
        description: "Quit editor",
    },
    PaletteCommand {
        name: "undo",
        shortcut: Some(ShortcutAction::Undo),
        action: AppAction::Undo,
        description: "Undo last change",
    },
    PaletteCommand {
        name: "redo",
        shortcut: Some(ShortcutAction::Redo),
        action: AppAction::Redo,
        description: "Redo undone change",
    },
    PaletteCommand {
        name: "search",
        shortcut: Some(ShortcutAction::Search),
        action: AppAction::StartSearch,
        description: "Find in file",
    },
    PaletteCommand {
        name: "go to line",
        shortcut: Some(ShortcutAction::GoToLine),
        action: AppAction::OpenGoToLine,
        description: "Jump to line number",
    },
    PaletteCommand {
        name: "toggle line numbers",
        shortcut: Some(ShortcutAction::ToggleLineNumbers),
        action: AppAction::ToggleLineNumbers,
        description: "Cycle line number modes",
    },
    PaletteCommand {
        name: "copy",
        shortcut: Some(ShortcutAction::Copy),
        action: AppAction::Copy,
        description: "Copy selection to clipboard",
    },
    PaletteCommand {
        name: "cut",
        shortcut: Some(ShortcutAction::Cut),
        action: AppAction::Cut,
        description: "Cut selection to clipboard",
    },
    PaletteCommand {
        name: "paste",
        shortcut: Some(ShortcutAction::Paste),
        action: AppAction::Paste,
        description: "Paste from clipboard",
    },
    PaletteCommand {
        name: "select all",
        shortcut: Some(ShortcutAction::SelectAll),
        action: AppAction::SelectAll,
        description: "Select entire file",
    },
    PaletteCommand {
        name: "help",
        shortcut: Some(ShortcutAction::Help),
        action: AppAction::ShowHelp,
        description: "Show keyboard shortcuts",
    },
    PaletteCommand {
        name: "find and replace",
        shortcut: Some(ShortcutAction::Replace),
        action: AppAction::StartReplace,
        description: "Find and replace text",
    },
    PaletteCommand {
        name: "uppercase selection",
        shortcut: Some(ShortcutAction::UppercaseSelection),
        action: AppAction::UppercaseSelection,
        description: "Convert selection to UPPERCASE",
    },
    PaletteCommand {
        name: "lowercase selection",
        shortcut: Some(ShortcutAction::LowercaseSelection),
        action: AppAction::LowercaseSelection,
        description: "Convert selection to lowercase",
    },
    PaletteCommand {
        name: "delete word before",
        shortcut: Some(ShortcutAction::DeleteWordBefore),
        action: AppAction::DeleteWordBefore,
        description: "Delete word before cursor",
    },
    PaletteCommand {
        name: "delete word after",
        shortcut: Some(ShortcutAction::DeleteWordAfter),
        action: AppAction::DeleteWordAfter,
        description: "Delete word after cursor",
    },
    PaletteCommand {
        name: "open settings",
        shortcut: Some(ShortcutAction::Settings),
        action: AppAction::OpenSettings,
        description: "Open interactive settings panel",
    },
    PaletteCommand {
        name: "open config",
        shortcut: None,
        action: AppAction::OpenConfig,
        description: "Open config.toml in editor",
    },
];

pub fn filter_commands(query: &str) -> Vec<usize> {
    if query.is_empty() {
        return (0..COMMANDS.len()).collect();
    }
    let q = query.to_lowercase();
    let mut results: Vec<(usize, usize)> = COMMANDS
        .iter()
        .enumerate()
        .filter_map(|(i, cmd)| {
            let name = cmd.name.to_lowercase();
            name.find(&q).map(|pos| (i, pos))
        })
        .collect();
    results.sort_by_key(|(_, pos)| *pos);
    results.into_iter().map(|(i, _)| i).collect()
}
