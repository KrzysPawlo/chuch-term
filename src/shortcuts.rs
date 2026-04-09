use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShortcutProfile {
    #[default]
    Ctrl,
    Alt,
}

impl ShortcutProfile {
    pub fn modifier(self) -> KeyModifiers {
        match self {
            Self::Ctrl => KeyModifiers::CONTROL,
            Self::Alt => KeyModifiers::ALT,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Ctrl => "ctrl",
            Self::Alt => "alt",
        }
    }

    fn prefix_long(self) -> &'static str {
        match self {
            Self::Ctrl => "Ctrl+",
            Self::Alt => "Alt+",
        }
    }

    fn prefix_compact(self) -> &'static str {
        match self {
            Self::Ctrl => "^",
            Self::Alt => "Alt+",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyToken {
    Char(char),
    Comma,
    Delete,
    Left,
    Right,
}

impl KeyToken {
    pub fn parse(raw: &str) -> Option<Self> {
        let lower = raw.trim().to_ascii_lowercase();
        match lower.as_str() {
            "comma" => Some(Self::Comma),
            "delete" => Some(Self::Delete),
            "left" => Some(Self::Left),
            "right" => Some(Self::Right),
            _ if lower.len() == 1 => {
                let ch = lower.chars().next()?;
                if ch.is_ascii_lowercase() {
                    Some(Self::Char(ch))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn config_value(self) -> String {
        match self {
            Self::Char(ch) => ch.to_string(),
            Self::Comma => "comma".to_string(),
            Self::Delete => "delete".to_string(),
            Self::Left => "left".to_string(),
            Self::Right => "right".to_string(),
        }
    }

    pub fn matches(self, code: KeyCode) -> bool {
        match (self, code) {
            (Self::Char(expected), KeyCode::Char(actual)) => expected == actual.to_ascii_lowercase(),
            (Self::Comma, KeyCode::Char(',')) => true,
            (Self::Delete, KeyCode::Delete) => true,
            (Self::Left, KeyCode::Left) => true,
            (Self::Right, KeyCode::Right) => true,
            _ => false,
        }
    }

    fn long_suffix(self) -> &'static str {
        match self {
            Self::Char('a') => "A",
            Self::Char('b') => "B",
            Self::Char('c') => "C",
            Self::Char('d') => "D",
            Self::Char('e') => "E",
            Self::Char('f') => "F",
            Self::Char('g') => "G",
            Self::Char('h') => "H",
            Self::Char('i') => "I",
            Self::Char('j') => "J",
            Self::Char('k') => "K",
            Self::Char('l') => "L",
            Self::Char('m') => "M",
            Self::Char('n') => "N",
            Self::Char('o') => "O",
            Self::Char('p') => "P",
            Self::Char('q') => "Q",
            Self::Char('r') => "R",
            Self::Char('s') => "S",
            Self::Char('t') => "T",
            Self::Char('u') => "U",
            Self::Char('v') => "V",
            Self::Char('w') => "W",
            Self::Char('x') => "X",
            Self::Char('y') => "Y",
            Self::Char('z') => "Z",
            Self::Comma => ",",
            Self::Delete => "Delete",
            Self::Left => "Left",
            Self::Right => "Right",
            Self::Char(_) => "?",
        }
    }

    fn compact_suffix(self) -> &'static str {
        match self {
            Self::Char('a') => "A",
            Self::Char('b') => "B",
            Self::Char('c') => "C",
            Self::Char('d') => "D",
            Self::Char('e') => "E",
            Self::Char('f') => "F",
            Self::Char('g') => "G",
            Self::Char('h') => "H",
            Self::Char('i') => "I",
            Self::Char('j') => "J",
            Self::Char('k') => "K",
            Self::Char('l') => "L",
            Self::Char('m') => "M",
            Self::Char('n') => "N",
            Self::Char('o') => "O",
            Self::Char('p') => "P",
            Self::Char('q') => "Q",
            Self::Char('r') => "R",
            Self::Char('s') => "S",
            Self::Char('t') => "T",
            Self::Char('u') => "U",
            Self::Char('v') => "V",
            Self::Char('w') => "W",
            Self::Char('x') => "X",
            Self::Char('y') => "Y",
            Self::Char('z') => "Z",
            Self::Comma => ",",
            Self::Delete => "Del",
            Self::Left => "←",
            Self::Right => "→",
            Self::Char(_) => "?",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutAction {
    Save,
    Quit,
    Help,
    Undo,
    Redo,
    Search,
    SearchNext,
    SearchPrev,
    GoToLine,
    ToggleLineNumbers,
    Palette,
    SelectAll,
    Copy,
    Cut,
    Paste,
    GoBackBuffer,
    Replace,
    ReplaceAll,
    ToggleCaseSensitive,
    DuplicateLine,
    Settings,
    DeleteWordBefore,
    DeleteWordAfter,
    WordLeft,
    WordRight,
    UppercaseSelection,
    LowercaseSelection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutContext {
    Normal,
    ConfirmQuit,
    Help,
    Search,
    Replace,
    CommandPalette,
}

#[derive(Debug, Clone)]
pub struct ActiveShortcuts {
    profile: ShortcutProfile,
    bindings: HashMap<ShortcutAction, KeyToken>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelStyle {
    Long,
    Compact,
}

struct ShortcutSpec {
    action: ShortcutAction,
    id: &'static str,
    name: &'static str,
    description: &'static str,
    ctrl: KeyToken,
    alt: KeyToken,
    contexts: &'static [ShortcutContext],
    policy: ShortcutPolicy,
}

#[derive(Debug, Clone, Copy)]
enum ShortcutPolicy {
    AnyToken,
    LettersAndComma,
    DeleteOnly,
    ArrowsOnly,
}

const CTX_NORMAL_ONLY: &[ShortcutContext] = &[ShortcutContext::Normal];
const CTX_HELP: &[ShortcutContext] = &[ShortcutContext::Normal, ShortcutContext::ConfirmQuit, ShortcutContext::Help];
const CTX_QUIT: &[ShortcutContext] = &[ShortcutContext::Normal, ShortcutContext::ConfirmQuit];
const CTX_SAVE: &[ShortcutContext] = &[ShortcutContext::Normal, ShortcutContext::ConfirmQuit];
const CTX_SEARCH: &[ShortcutContext] = &[ShortcutContext::Normal, ShortcutContext::Search];
const CTX_SEARCH_NEXT: &[ShortcutContext] = &[ShortcutContext::Normal, ShortcutContext::Search, ShortcutContext::Replace];
const CTX_SEARCH_PREV: &[ShortcutContext] = &[ShortcutContext::Search];
const CTX_REPLACE: &[ShortcutContext] = &[ShortcutContext::Normal, ShortcutContext::Search];
const CTX_REPLACE_ALL: &[ShortcutContext] = &[ShortcutContext::Replace];
const CTX_CASE_TOGGLE: &[ShortcutContext] = &[ShortcutContext::Search, ShortcutContext::Replace];
const CTX_PALETTE: &[ShortcutContext] = &[ShortcutContext::Normal, ShortcutContext::CommandPalette];

const SHORTCUT_SPECS: &[ShortcutSpec] = &[
    ShortcutSpec { action: ShortcutAction::Save, id: "save", name: "Save", description: "Save current file", ctrl: KeyToken::Char('s'), alt: KeyToken::Char('s'), contexts: CTX_SAVE, policy: ShortcutPolicy::LettersAndComma },
    ShortcutSpec { action: ShortcutAction::Quit, id: "quit", name: "Quit", description: "Quit editor", ctrl: KeyToken::Char('q'), alt: KeyToken::Char('q'), contexts: CTX_QUIT, policy: ShortcutPolicy::LettersAndComma },
    ShortcutSpec { action: ShortcutAction::Help, id: "help", name: "Help", description: "Show keyboard shortcuts", ctrl: KeyToken::Char('h'), alt: KeyToken::Char('h'), contexts: CTX_HELP, policy: ShortcutPolicy::LettersAndComma },
    ShortcutSpec { action: ShortcutAction::Undo, id: "undo", name: "Undo", description: "Undo last change", ctrl: KeyToken::Char('z'), alt: KeyToken::Char('z'), contexts: CTX_NORMAL_ONLY, policy: ShortcutPolicy::LettersAndComma },
    ShortcutSpec { action: ShortcutAction::Redo, id: "redo", name: "Redo", description: "Redo undone change", ctrl: KeyToken::Char('y'), alt: KeyToken::Char('y'), contexts: CTX_NORMAL_ONLY, policy: ShortcutPolicy::LettersAndComma },
    ShortcutSpec { action: ShortcutAction::Search, id: "search", name: "Search", description: "Find in file", ctrl: KeyToken::Char('f'), alt: KeyToken::Char('f'), contexts: CTX_SEARCH, policy: ShortcutPolicy::LettersAndComma },
    ShortcutSpec { action: ShortcutAction::SearchNext, id: "search_next", name: "Search next", description: "Jump to next match", ctrl: KeyToken::Char('n'), alt: KeyToken::Char('n'), contexts: CTX_SEARCH_NEXT, policy: ShortcutPolicy::LettersAndComma },
    ShortcutSpec { action: ShortcutAction::SearchPrev, id: "search_prev", name: "Search previous", description: "Jump to previous match", ctrl: KeyToken::Char('p'), alt: KeyToken::Char('p'), contexts: CTX_SEARCH_PREV, policy: ShortcutPolicy::LettersAndComma },
    ShortcutSpec { action: ShortcutAction::GoToLine, id: "goto_line", name: "Go to line", description: "Jump to line number", ctrl: KeyToken::Char('g'), alt: KeyToken::Char('g'), contexts: CTX_NORMAL_ONLY, policy: ShortcutPolicy::LettersAndComma },
    ShortcutSpec { action: ShortcutAction::ToggleLineNumbers, id: "line_numbers", name: "Toggle line numbers", description: "Cycle line number modes", ctrl: KeyToken::Char('l'), alt: KeyToken::Char('m'), contexts: CTX_NORMAL_ONLY, policy: ShortcutPolicy::LettersAndComma },
    ShortcutSpec { action: ShortcutAction::Palette, id: "palette", name: "Command palette", description: "Open command palette", ctrl: KeyToken::Char('p'), alt: KeyToken::Char('k'), contexts: CTX_PALETTE, policy: ShortcutPolicy::LettersAndComma },
    ShortcutSpec { action: ShortcutAction::SelectAll, id: "select_all", name: "Select all", description: "Select entire file", ctrl: KeyToken::Char('a'), alt: KeyToken::Char('a'), contexts: CTX_NORMAL_ONLY, policy: ShortcutPolicy::LettersAndComma },
    ShortcutSpec { action: ShortcutAction::Copy, id: "copy", name: "Copy", description: "Copy selection to clipboard", ctrl: KeyToken::Char('c'), alt: KeyToken::Char('c'), contexts: CTX_NORMAL_ONLY, policy: ShortcutPolicy::LettersAndComma },
    ShortcutSpec { action: ShortcutAction::Cut, id: "cut", name: "Cut", description: "Cut selection to clipboard", ctrl: KeyToken::Char('x'), alt: KeyToken::Char('x'), contexts: CTX_NORMAL_ONLY, policy: ShortcutPolicy::LettersAndComma },
    ShortcutSpec { action: ShortcutAction::Paste, id: "paste", name: "Paste", description: "Paste from clipboard", ctrl: KeyToken::Char('v'), alt: KeyToken::Char('v'), contexts: CTX_NORMAL_ONLY, policy: ShortcutPolicy::LettersAndComma },
    ShortcutSpec { action: ShortcutAction::GoBackBuffer, id: "go_back", name: "Go back", description: "Return to previous buffer", ctrl: KeyToken::Char('o'), alt: KeyToken::Char('o'), contexts: CTX_NORMAL_ONLY, policy: ShortcutPolicy::LettersAndComma },
    ShortcutSpec { action: ShortcutAction::Replace, id: "replace", name: "Find and replace", description: "Find and replace text", ctrl: KeyToken::Char('r'), alt: KeyToken::Char('r'), contexts: CTX_REPLACE, policy: ShortcutPolicy::LettersAndComma },
    ShortcutSpec { action: ShortcutAction::ReplaceAll, id: "replace_all", name: "Replace all", description: "Replace all matches", ctrl: KeyToken::Char('a'), alt: KeyToken::Char('a'), contexts: CTX_REPLACE_ALL, policy: ShortcutPolicy::LettersAndComma },
    ShortcutSpec { action: ShortcutAction::ToggleCaseSensitive, id: "case_sensitive", name: "Case sensitivity", description: "Toggle case-sensitive search", ctrl: KeyToken::Char('i'), alt: KeyToken::Char('i'), contexts: CTX_CASE_TOGGLE, policy: ShortcutPolicy::LettersAndComma },
    ShortcutSpec { action: ShortcutAction::DuplicateLine, id: "duplicate_line", name: "Duplicate line", description: "Duplicate current line", ctrl: KeyToken::Char('d'), alt: KeyToken::Char('d'), contexts: CTX_NORMAL_ONLY, policy: ShortcutPolicy::LettersAndComma },
    ShortcutSpec { action: ShortcutAction::Settings, id: "settings", name: "Settings", description: "Open settings panel", ctrl: KeyToken::Char('t'), alt: KeyToken::Comma, contexts: CTX_NORMAL_ONLY, policy: ShortcutPolicy::AnyToken },
    ShortcutSpec { action: ShortcutAction::DeleteWordBefore, id: "delete_word_before", name: "Delete word before", description: "Delete word before cursor", ctrl: KeyToken::Char('w'), alt: KeyToken::Char('w'), contexts: CTX_NORMAL_ONLY, policy: ShortcutPolicy::LettersAndComma },
    ShortcutSpec { action: ShortcutAction::DeleteWordAfter, id: "delete_word_after", name: "Delete word after", description: "Delete word after cursor", ctrl: KeyToken::Delete, alt: KeyToken::Delete, contexts: CTX_NORMAL_ONLY, policy: ShortcutPolicy::DeleteOnly },
    ShortcutSpec { action: ShortcutAction::WordLeft, id: "word_left", name: "Word left", description: "Move one word left", ctrl: KeyToken::Left, alt: KeyToken::Left, contexts: CTX_NORMAL_ONLY, policy: ShortcutPolicy::ArrowsOnly },
    ShortcutSpec { action: ShortcutAction::WordRight, id: "word_right", name: "Word right", description: "Move one word right", ctrl: KeyToken::Right, alt: KeyToken::Right, contexts: CTX_NORMAL_ONLY, policy: ShortcutPolicy::ArrowsOnly },
    ShortcutSpec { action: ShortcutAction::UppercaseSelection, id: "uppercase_selection", name: "Uppercase selection", description: "Convert selection to UPPERCASE", ctrl: KeyToken::Char('u'), alt: KeyToken::Char('u'), contexts: CTX_NORMAL_ONLY, policy: ShortcutPolicy::LettersAndComma },
    ShortcutSpec { action: ShortcutAction::LowercaseSelection, id: "lowercase_selection", name: "Lowercase selection", description: "Convert selection to lowercase", ctrl: KeyToken::Char('j'), alt: KeyToken::Char('l'), contexts: CTX_NORMAL_ONLY, policy: ShortcutPolicy::LettersAndComma },
];

pub fn configurable_actions() -> &'static [ShortcutAction] {
    &[
        ShortcutAction::Save,
        ShortcutAction::Quit,
        ShortcutAction::Help,
        ShortcutAction::Undo,
        ShortcutAction::Redo,
        ShortcutAction::Search,
        ShortcutAction::GoToLine,
        ShortcutAction::ToggleLineNumbers,
        ShortcutAction::Palette,
        ShortcutAction::SelectAll,
        ShortcutAction::Copy,
        ShortcutAction::Cut,
        ShortcutAction::Paste,
        ShortcutAction::GoBackBuffer,
        ShortcutAction::Replace,
        ShortcutAction::DuplicateLine,
        ShortcutAction::Settings,
        ShortcutAction::DeleteWordBefore,
        ShortcutAction::DeleteWordAfter,
        ShortcutAction::WordLeft,
        ShortcutAction::WordRight,
        ShortcutAction::UppercaseSelection,
        ShortcutAction::LowercaseSelection,
    ]
}

fn spec(action: ShortcutAction) -> &'static ShortcutSpec {
    SHORTCUT_SPECS
        .iter()
        .find(|spec| spec.action == action)
        .expect("shortcut spec should exist")
}

fn spec_by_id(action_id: &str) -> Option<&'static ShortcutSpec> {
    SHORTCUT_SPECS.iter().find(|spec| spec.id == action_id)
}

impl ShortcutAction {
    pub fn id(self) -> &'static str {
        spec(self).id
    }

    pub fn name(self) -> &'static str {
        spec(self).name
    }

    pub fn description(self) -> &'static str {
        spec(self).description
    }

    pub fn accepts_token(self, token: KeyToken) -> bool {
        match spec(self).policy {
            ShortcutPolicy::AnyToken => true,
            ShortcutPolicy::LettersAndComma => matches!(token, KeyToken::Char(_) | KeyToken::Comma),
            ShortcutPolicy::DeleteOnly => token == KeyToken::Delete,
            ShortcutPolicy::ArrowsOnly => matches!(token, KeyToken::Left | KeyToken::Right),
        }
    }
}

impl ActiveShortcuts {
    pub fn resolve(config: &crate::config::ShortcutsSection) -> Result<Self, Vec<String>> {
        let mut bindings = HashMap::new();
        let mut errors = Vec::new();

        for spec in SHORTCUT_SPECS {
            let token = match config.overrides.get(spec.id) {
                Some(raw) => match KeyToken::parse(raw) {
                    Some(token) if spec.action.accepts_token(token) => token,
                    Some(token) => {
                        errors.push(format!(
                            "Config: shortcuts.overrides.{} does not allow key token {:?}",
                            spec.id, token.config_value()
                        ));
                        continue;
                    }
                    None => {
                        errors.push(format!(
                            "Config: shortcuts.overrides.{} has unsupported key token {:?}",
                            spec.id, raw
                        ));
                        continue;
                    }
                },
                None => match config.profile {
                    ShortcutProfile::Ctrl => spec.ctrl,
                    ShortcutProfile::Alt => spec.alt,
                },
            };
            bindings.insert(spec.action, token);
        }

        for action_id in config.overrides.keys() {
            if spec_by_id(action_id).is_none() {
                errors.push(format!(
                    "Config: shortcuts.overrides.{action_id} is not a known action"
                ));
            }
        }

        if errors.is_empty() {
            let mut seen = HashMap::<(ShortcutContext, KeyToken), ShortcutAction>::new();
            for spec in SHORTCUT_SPECS {
                if let Some(&token) = bindings.get(&spec.action) {
                    for &context in spec.contexts {
                        if let Some(existing) = seen.insert((context, token), spec.action) {
                            errors.push(format!(
                                "Config: shortcut collision in {:?}: {} conflicts with {}",
                                context,
                                existing.id(),
                                spec.action.id(),
                            ));
                        }
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(Self {
                profile: config.profile,
                bindings,
            })
        } else {
            Err(errors)
        }
    }

    pub fn profile(&self) -> ShortcutProfile {
        self.profile
    }

    pub fn token_for(&self, action: ShortcutAction) -> KeyToken {
        *self
            .bindings
            .get(&action)
            .expect("active shortcut should exist")
    }

    pub fn label_for(&self, action: ShortcutAction, style: LabelStyle) -> String {
        let token = self.token_for(action);
        match style {
            LabelStyle::Long => format!("{}{}", self.profile.prefix_long(), token.long_suffix()),
            LabelStyle::Compact => format!("{}{}", self.profile.prefix_compact(), token.compact_suffix()),
        }
    }

    pub fn resolve_action(&self, context: ShortcutContext, event: KeyEvent) -> Option<ShortcutAction> {
        if event.modifiers != self.profile.modifier() {
            return None;
        }

        SHORTCUT_SPECS
            .iter()
            .filter(|spec| spec.contexts.contains(&context))
            .find_map(|spec| {
                let token = self.bindings.get(&spec.action)?;
                token.matches(event.code).then_some(spec.action)
            })
    }
}

pub fn capture_token(event: KeyEvent) -> Option<KeyToken> {
    match event.code {
        KeyCode::Char(',') => Some(KeyToken::Comma),
        KeyCode::Char(ch) if ch.is_ascii_alphabetic() => Some(KeyToken::Char(ch.to_ascii_lowercase())),
        KeyCode::Delete => Some(KeyToken::Delete),
        KeyCode::Left => Some(KeyToken::Left),
        KeyCode::Right => Some(KeyToken::Right),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use crate::config::ShortcutsSection;

    #[test]
    fn ctrl_profile_uses_expected_labels() {
        let shortcuts = ActiveShortcuts::resolve(&ShortcutsSection::default()).expect("shortcuts");
        assert_eq!(shortcuts.label_for(ShortcutAction::Save, LabelStyle::Long), "Ctrl+S");
        assert_eq!(shortcuts.label_for(ShortcutAction::Help, LabelStyle::Compact), "^H");
        assert_eq!(shortcuts.label_for(ShortcutAction::Settings, LabelStyle::Long), "Ctrl+T");
    }

    #[test]
    fn alt_profile_uses_alt_defaults() {
        let shortcuts = ActiveShortcuts::resolve(&crate::config::ShortcutsSection {
            profile: ShortcutProfile::Alt,
            overrides: BTreeMap::new(),
        })
        .expect("shortcuts");

        assert_eq!(shortcuts.label_for(ShortcutAction::Settings, LabelStyle::Long), "Alt+,");
        assert_eq!(shortcuts.label_for(ShortcutAction::Help, LabelStyle::Long), "Alt+H");
    }

    #[test]
    fn invalid_override_token_fails_validation() {
        let mut config = crate::config::ShortcutsSection::default();
        config.overrides.insert("save".to_string(), "tab".to_string());

        let err = ActiveShortcuts::resolve(&config).expect_err("invalid token should fail");
        assert!(err.iter().any(|msg| msg.contains("unsupported key token")));
    }

    #[test]
    fn duplicate_override_is_rejected() {
        let mut config = crate::config::ShortcutsSection::default();
        config.overrides.insert("save".to_string(), "q".to_string());

        let err = ActiveShortcuts::resolve(&config).expect_err("collision should fail");
        assert!(err.iter().any(|msg| msg.contains("shortcut collision")));
    }
}
