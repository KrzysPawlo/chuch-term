use crate::input::keybindings::AppAction;

pub struct PaletteCommand {
    pub name: &'static str,
    pub key: &'static str,
    pub action: AppAction,
    pub description: &'static str,
}

pub static COMMANDS: &[PaletteCommand] = &[
    PaletteCommand {
        name: "save",
        key: "Ctrl+S",
        action: AppAction::Save,
        description: "Save current file",
    },
    PaletteCommand {
        name: "quit",
        key: "Ctrl+Q",
        action: AppAction::RequestQuit,
        description: "Quit editor",
    },
    PaletteCommand {
        name: "undo",
        key: "Ctrl+Z",
        action: AppAction::Undo,
        description: "Undo last change",
    },
    PaletteCommand {
        name: "redo",
        key: "Ctrl+Y",
        action: AppAction::Redo,
        description: "Redo undone change",
    },
    PaletteCommand {
        name: "search",
        key: "Ctrl+F",
        action: AppAction::StartSearch,
        description: "Find in file",
    },
    PaletteCommand {
        name: "go to line",
        key: "Ctrl+G",
        action: AppAction::OpenGoToLine,
        description: "Jump to line number",
    },
    PaletteCommand {
        name: "toggle line numbers",
        key: "Ctrl+L",
        action: AppAction::ToggleLineNumbers,
        description: "Cycle line number modes",
    },
    PaletteCommand {
        name: "copy",
        key: "Ctrl+C",
        action: AppAction::Copy,
        description: "Copy selection to clipboard",
    },
    PaletteCommand {
        name: "cut",
        key: "Ctrl+X",
        action: AppAction::Cut,
        description: "Cut selection to clipboard",
    },
    PaletteCommand {
        name: "paste",
        key: "Ctrl+V",
        action: AppAction::Paste,
        description: "Paste from clipboard",
    },
    PaletteCommand {
        name: "select all",
        key: "Ctrl+A",
        action: AppAction::SelectAll,
        description: "Select entire file",
    },
    PaletteCommand {
        name: "help",
        key: "Ctrl+H",
        action: AppAction::ShowHelp,
        description: "Show keyboard shortcuts",
    },
    PaletteCommand {
        name: "open config",
        key: "",
        action: AppAction::OpenConfig,
        description: "Open config.toml in editor",
    },
    PaletteCommand {
        name: "find and replace",
        key: "Ctrl+R",
        action: AppAction::StartReplace,
        description: "Find and replace text",
    },
    PaletteCommand {
        name: "uppercase selection",
        key: "Alt+U",
        action: AppAction::UppercaseSelection,
        description: "Convert selection to UPPERCASE",
    },
    PaletteCommand {
        name: "lowercase selection",
        key: "Alt+L",
        action: AppAction::LowercaseSelection,
        description: "Convert selection to lowercase",
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
