pub mod keybindings;

pub use keybindings::{map_key, AppAction};

use anyhow::Result;
use crossterm::event::{Event, KeyEventKind, MouseButton, MouseEventKind};

use crate::editor::buffer::{
    byte_for_display_col, grapheme_slice, next_grapheme_boundary, prev_grapheme_boundary,
};
use crate::editor::history::{HistoryEntry, TextChange};
use crate::editor::{Cursor, EditorMode, EditorState, LineNumberMode, SearchMatch, TextBuffer};
use crate::shortcuts::{ActiveShortcuts, KeyToken, ShortcutAction, ShortcutProfile};

/// Translate a crossterm Event into an AppAction and apply it to the editor state.
pub fn handle_event(event: Event, state: &mut EditorState) -> Result<()> {
    let action = match event {
        Event::Key(key_event) => {
            if key_event.kind != KeyEventKind::Press {
                return Ok(());
            }
            map_key(key_event, state)
        }
        Event::Resize(_, _) => return Ok(()),
        Event::Mouse(mouse_event) => {
            if state.mode == EditorMode::Normal
                && matches!(mouse_event.kind, MouseEventKind::Down(MouseButton::Left))
            {
                handle_mouse_click(mouse_event.column, mouse_event.row, state);
            }
            return Ok(());
        }
        _ => return Ok(()),
    };

    apply_action(state, action)
}

/// Apply a semantic action to the editor state.
fn apply_action(state: &mut EditorState, action: AppAction) -> Result<()> {
    use AppAction::*;

    state.status_message = None;
    clamp_cursor_to_buffer(state);

    match action {
        Noop => {}

        ShowHelp => {
            state.pre_help_mode = state.mode;
            state.mode = EditorMode::Help;
        }

        CloseHelp => {
            state.mode = state.pre_help_mode;
        }

        RequestQuit => {
            if state.buffer.dirty {
                state.mode = EditorMode::ConfirmQuit;
            } else {
                state.should_quit = true;
            }
        }

        ForceQuit => {
            state.should_quit = true;
        }

        CancelQuit => {
            state.mode = EditorMode::Normal;
        }

        Save => {
            if state.buffer.file_path.is_none() {
                // No path yet — enter SaveAs mode so the user can type a filename.
                state.saveas_input = String::new();
                state.mode = EditorMode::SaveAs;
                return Ok(());
            }
            let was_confirm_quit = state.mode == EditorMode::ConfirmQuit;
            match state.buffer.save() {
                Ok(()) => {
                    state.mode = EditorMode::Normal;
                    if was_confirm_quit {
                        state.should_quit = true;
                    } else {
                        state.status_message = Some("Saved.".to_string());
                    }
                }
                Err(err) => {
                    state.mode = EditorMode::Normal;
                    state.status_message = Some(format!("Save error: {err}"));
                }
            }
        }

        SaveAsChar(c) => {
            state.saveas_input.push(c);
        }

        SaveAsBackspace => {
            state.saveas_input.pop();
        }

        CancelSaveAs => {
            state.saveas_input.clear();
            state.mode = EditorMode::Normal;
        }

        SaveAsSubmit => {
            let raw = state.saveas_input.trim().to_string();
            if raw.is_empty() {
                state.status_message = Some("Save cancelled — no filename entered.".to_string());
                state.mode = EditorMode::Normal;
                return Ok(());
            }
            // Expand leading ~ to the home directory.
            let expanded = if raw.starts_with("~/") || raw == "~" {
                let home = std::env::var_os("HOME")
                    .map(std::path::PathBuf::from)
                    .unwrap_or_default();
                home.join(raw.trim_start_matches("~/"))
            } else {
                std::path::PathBuf::from(&raw)
            };
            state.buffer.file_path = Some(expanded);
            match state.buffer.save() {
                Ok(()) => {
                    state.mode = EditorMode::Normal;
                    state.status_message = Some(format!(
                        "Saved: {}",
                        state.buffer.file_path.as_ref()
                            .and_then(|p| p.file_name())
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default()
                    ));
                }
                Err(err) => {
                    // Undo the path assignment so the user can try again.
                    state.buffer.file_path = None;
                    state.mode = EditorMode::SaveAs;
                    state.status_message = Some(format!("Save error: {err}"));
                }
            }
        }

        InsertChar(ch) => {
            let cursor_before = state.cursor;
            let text = if ch == '\t' && state.config.editor.expand_tabs {
                " ".repeat(state.config.editor.tab_width as usize)
            } else {
                ch.to_string()
            };
            let change = build_change(
                (cursor_before.row, cursor_before.col),
                String::new(),
                text,
                cursor_before,
            );
            apply_and_record_change(state, change, true);
        }

        DeleteBefore => {
            let cursor_before = state.cursor;
            let row = cursor_before.row;
            let col = cursor_before.col;

            if col > 0 {
                let line = state.buffer.line(row);
                let start_col = prev_grapheme_boundary(line, col);
                let old_text = line[start_col..col].to_string();
                let change = build_change(
                    (row, start_col),
                    old_text,
                    String::new(),
                    cursor_before,
                );
                apply_and_record_change(state, change, false);
            } else if row > 0 {
                let previous_len = state.buffer.line(row - 1).len();
                let change = build_change(
                    (row - 1, previous_len),
                    "\n".to_string(),
                    String::new(),
                    cursor_before,
                );
                apply_and_record_change(state, change, false);
            }
        }

        DeleteAt => {
            let cursor_before = state.cursor;
            let row = cursor_before.row;
            let col = cursor_before.col;
            let line = state.buffer.line(row);

            if col < line.len() {
                let end = next_grapheme_boundary(line, col);
                let old_text = line[col..end].to_string();
                let change = build_change((row, col), old_text, String::new(), cursor_before);
                apply_and_record_change(state, change, false);
            } else if row + 1 < state.buffer.line_count() {
                let change = build_change((row, col), "\n".to_string(), String::new(), cursor_before);
                apply_and_record_change(state, change, false);
            }
        }

        InsertNewline => {
            let cursor_before = state.cursor;
            let indent = if state.config.editor.auto_indent {
                let line = state.buffer.line(cursor_before.row);
                let up_to_cursor = &line[..state.buffer.clamp_column(cursor_before.row, cursor_before.col)];
                up_to_cursor
                    .chars()
                    .take_while(|c| *c == ' ' || *c == '\t')
                    .collect::<String>()
            } else {
                String::new()
            };
            let change = build_change(
                (cursor_before.row, cursor_before.col),
                String::new(),
                format!("\n{indent}"),
                cursor_before,
            );
            apply_and_record_change(state, change, false);
        }

        MoveUp => {
            state.selection_anchor = None;
            state.cursor.move_up(&state.buffer);
        }
        MoveDown => {
            state.selection_anchor = None;
            state.cursor.move_down(&state.buffer);
        }
        MoveLeft => {
            state.selection_anchor = None;
            state.cursor.move_left(&state.buffer);
        }
        MoveRight => {
            state.selection_anchor = None;
            state.cursor.move_right(&state.buffer);
        }
        Home => {
            state.selection_anchor = None;
            state.cursor.home();
        }
        End => {
            state.selection_anchor = None;
            state.cursor.end(&state.buffer);
        }
        PageUp => {
            state.selection_anchor = None;
            let vh = state.viewport_height.max(1);
            state.cursor.page_up(&state.buffer, vh);
        }
        PageDown => {
            state.selection_anchor = None;
            let vh = state.viewport_height.max(1);
            state.cursor.page_down(&state.buffer, vh);
        }

        Undo => {
            if let Some(entry) = state.history.undo_stack.pop() {
                apply_history_entry_reverse(state, &entry);
                state.history.redo_stack.push(entry);
            }
        }

        Redo => {
            if let Some(entry) = state.history.redo_stack.pop() {
                apply_history_entry_forward(state, &entry);
                state.history.undo_stack.push(entry);
            }
        }

        StartSearch => {
            state.mode = EditorMode::Search;
            state.search_query.clear();
            state.search_results.clear();
            state.search_result_idx = 0;
        }

        CloseSearch => {
            state.mode = EditorMode::Normal;
        }

        SearchChar(c) => {
            state.search_query.push(c);
            refresh_search_results(state, true);
        }

        SearchBackspace => {
            state.search_query.pop();
            refresh_search_results(state, true);
        }

        SearchNext => {
            if !state.search_results.is_empty() {
                state.search_result_idx =
                    (state.search_result_idx + 1) % state.search_results.len();
                move_cursor_to_search_match(state);
            }
        }

        SearchPrev => {
            if !state.search_results.is_empty() {
                let len = state.search_results.len();
                state.search_result_idx = (state.search_result_idx + len - 1) % len;
                move_cursor_to_search_match(state);
            }
        }

        OpenGoToLine => {
            state.mode = EditorMode::GoToLine;
            state.goto_input.clear();
        }

        GoToLineChar(c) => {
            state.goto_input.push(c);
        }

        GoToLineBackspace => {
            state.goto_input.pop();
        }

        GoToLineSubmit => {
            if let Ok(value) = state.goto_input.parse::<usize>() {
                let target = value.max(1).min(state.buffer.line_count());
                state.cursor = Cursor {
                    row: target - 1,
                    col: 0,
                };
            }
            state.mode = EditorMode::Normal;
        }

        CloseGoToLine => {
            state.mode = EditorMode::Normal;
        }

        ToggleLineNumbers => {
            state.line_number_mode = match state.line_number_mode {
                LineNumberMode::Off => LineNumberMode::Absolute,
                LineNumberMode::Absolute => LineNumberMode::Relative,
                LineNumberMode::Relative => LineNumberMode::Off,
            };
        }

        SelectAll => {
            state.selection_anchor = Some(Cursor { row: 0, col: 0 });
            let last_row = state.buffer.line_count().saturating_sub(1);
            let last_col = state.buffer.line(last_row).len();
            state.cursor = Cursor {
                row: last_row,
                col: last_col,
            };
        }

        Copy => {
            if let Some((start, end)) = state.selection_range() {
                let text = state.buffer.text_in_range(start, end);
                let strategy = state.config.clipboard.strategy.clone();
                let result = crate::clipboard::copy_to_clipboard(&text, &strategy);
                state.clipboard = text;
                state.status_message = clipboard_copy_status(&strategy, result);
            }
        }

        Cut => {
            if let Some((start, end)) = state.selection_range() {
                let text = state.buffer.text_in_range(start, end);
                let strategy = state.config.clipboard.strategy.clone();
                let result = crate::clipboard::copy_to_clipboard(&text, &strategy);
                state.clipboard = text.clone();
                state.status_message = clipboard_copy_status(&strategy, result);

                let change = build_change(
                    start,
                    text,
                    String::new(),
                    state.cursor,
                );
                apply_and_record_change(state, change, false);
                state.selection_anchor = None;
            }
        }

        Paste => {
            let strategy = state.config.clipboard.strategy.clone();
            let (text, status) = match crate::clipboard::paste_from_clipboard(&strategy) {
                crate::clipboard::ClipboardPasteResult::System(text) if !text.is_empty() => {
                    (text, None)
                }
                _ => {
                    let fallback = state.clipboard.clone();
                    let status = clipboard_paste_status(&strategy, fallback.is_empty());
                    (fallback, status)
                }
            };
            state.status_message = status;
            if !text.is_empty() {
                let cursor_before = state.cursor;
                let change = build_change(
                    (cursor_before.row, cursor_before.col),
                    String::new(),
                    text,
                    cursor_before,
                );
                apply_and_record_change(state, change, false);
            }
        }

        ClearSelection => {
            state.selection_anchor = None;
        }

        ShiftUp => {
            if state.selection_anchor.is_none() {
                state.selection_anchor = Some(state.cursor);
            }
            state.cursor.move_up(&state.buffer);
        }
        ShiftDown => {
            if state.selection_anchor.is_none() {
                state.selection_anchor = Some(state.cursor);
            }
            state.cursor.move_down(&state.buffer);
        }
        ShiftLeft => {
            if state.selection_anchor.is_none() {
                state.selection_anchor = Some(state.cursor);
            }
            state.cursor.move_left(&state.buffer);
        }
        ShiftRight => {
            if state.selection_anchor.is_none() {
                state.selection_anchor = Some(state.cursor);
            }
            state.cursor.move_right(&state.buffer);
        }
        ShiftHome => {
            if state.selection_anchor.is_none() {
                state.selection_anchor = Some(state.cursor);
            }
            state.cursor.home();
        }
        ShiftEnd => {
            if state.selection_anchor.is_none() {
                state.selection_anchor = Some(state.cursor);
            }
            state.cursor.end(&state.buffer);
        }

        OpenCommandPalette => {
            state.mode = EditorMode::CommandPalette;
            state.palette_query.clear();
            state.palette_matches = (0..crate::commands::COMMANDS.len()).collect();
            state.palette_cursor = 0;
        }

        ClosePalette => {
            state.mode = EditorMode::Normal;
        }

        PaletteChar(c) => {
            state.palette_query.push(c);
            state.palette_matches = crate::commands::filter_commands(&state.palette_query);
            state.palette_cursor = 0;
        }

        PaletteBackspace => {
            state.palette_query.pop();
            state.palette_matches = crate::commands::filter_commands(&state.palette_query);
            state.palette_cursor = 0;
        }

        PaletteUp => {
            if !state.palette_matches.is_empty() {
                if state.palette_cursor == 0 {
                    // Wrap: jump to the last item.
                    state.palette_cursor = state.palette_matches.len() - 1;
                } else {
                    state.palette_cursor -= 1;
                }
            }
        }

        PaletteDown => {
            if !state.palette_matches.is_empty() {
                if state.palette_cursor + 1 >= state.palette_matches.len() {
                    // Wrap: jump back to the first item.
                    state.palette_cursor = 0;
                } else {
                    state.palette_cursor += 1;
                }
            }
        }

        PaletteSubmit => {
            let idx = state.palette_matches.get(state.palette_cursor).copied();
            state.mode = EditorMode::Normal;
            if let Some(command_index) = idx
                && let Some(command) = crate::commands::COMMANDS.get(command_index)
            {
                return apply_action(state, command.action);
            }
        }

        OpenConfig => {
            if let Some(path) = crate::config::config_path() {
                match crate::editor::buffer::TextBuffer::from_file(&path) {
                    Ok(buffer) => {
                        let previous_buffer = std::mem::replace(&mut state.buffer, buffer);
                        let previous_cursor = state.cursor;
                        state.previous_buffer = Some((previous_buffer, previous_cursor));
                        state.cursor = Cursor::new();
                        state.history = crate::editor::history::History::new();
                        state.selection_anchor = None;
                        state.mode = EditorMode::Normal;
                        state.status_message =
                            Some("Editing config — Ctrl+O to go back".to_string());
                        sync_search_results_after_buffer_change(state);
                    }
                    Err(err) => {
                        state.status_message = Some(format!("Cannot open config: {err}"));
                    }
                }
            }
        }

        GoBackBuffer => {
            if let Some((previous_buffer, previous_cursor)) = state.previous_buffer.take() {
                let name = previous_buffer.display_name();
                state.buffer = previous_buffer;
                state.cursor = previous_cursor;
                state.history = crate::editor::history::History::new();
                state.selection_anchor = None;
                state.mode = EditorMode::Normal;
                state.status_message = Some(format!("Returned to {name}"));
                sync_search_results_after_buffer_change(state);
            }
        }

        StartReplace => {
            state.mode = EditorMode::Replace;
            if state.search_query.is_empty() {
                state.search_results.clear();
                state.search_result_idx = 0;
            }
            state.replace_query.clear();
        }

        ReplaceChar(c) => {
            state.replace_query.push(c);
        }

        ReplaceBackspace => {
            state.replace_query.pop();
        }

        ReplaceSubmit => {
            if let Some(current_match) = current_search_match(state) {
                let index = state.search_result_idx;
                let old_text = state
                    .buffer
                    .text_in_range((current_match.row, current_match.start), (current_match.row, current_match.end));
                let change = build_change_with_cursor(
                    (current_match.row, current_match.start),
                    old_text,
                    state.replace_query.clone(),
                    state.cursor,
                    Cursor {
                        row: current_match.row,
                        col: current_match.start,
                    },
                );
                apply_and_record_change(state, change, false);
                state.search_result_idx = index;
                sync_search_results_after_buffer_change(state);
                if !state.search_results.is_empty() {
                    state.search_result_idx =
                        state.search_result_idx.min(state.search_results.len() - 1);
                    move_cursor_to_search_match(state);
                }
            }
        }

        ReplaceAll => {
            if state.search_query.is_empty() || state.search_results.is_empty() {
                state.status_message = Some("No matches to replace".to_string());
            } else {
                let cursor_before = state.cursor;
                let replacements = state.search_results.len();
                let mut ordered = state.search_results.clone();
                ordered.sort_by(|left, right| (right.row, right.start).cmp(&(left.row, left.start)));
                let mut changes = Vec::with_capacity(ordered.len());

                for found in ordered {
                    let old_text = state
                        .buffer
                        .text_in_range((found.row, found.start), (found.row, found.end));
                    changes.push(build_change_with_cursor(
                        (found.row, found.start),
                        old_text,
                        state.replace_query.clone(),
                        cursor_before,
                        cursor_before,
                    ));
                }

                if !changes.is_empty() {
                    for change in &changes {
                        state
                            .buffer
                            .apply_change(change.start, &change.old_text, &change.new_text);
                    }
                    state.cursor = cursor_before;
                    clamp_cursor_to_buffer(state);
                    let cursor_after = state.cursor;
                    state
                        .history
                        .push_batch_no_merge(changes, cursor_before, cursor_after);
                }

                refresh_search_results(state, true);
                state.mode = EditorMode::Normal;
                state.status_message = Some(format!("Replaced {replacements} occurrences"));
            }
        }

        CloseReplace => {
            state.mode = EditorMode::Normal;
            state.search_results.clear();
            state.search_result_idx = 0;
        }

        SearchSelect => {
            if let Some(found) = current_search_match(state) {
                state.selection_anchor = Some(Cursor {
                    row: found.row,
                    col: found.start,
                });
                state.cursor = Cursor {
                    row: found.row,
                    col: found.end,
                };
            }
            state.mode = EditorMode::Normal;
        }

        ToggleCaseSensitive => {
            state.search_case_sensitive = !state.search_case_sensitive;
            refresh_search_results(state, true);
        }

        UppercaseSelection => {
            apply_selection_transform(state, |text| text.to_uppercase());
        }

        LowercaseSelection => {
            apply_selection_transform(state, |text| text.to_lowercase());
        }

        WordLeft => {
            state.selection_anchor = None;
            let (row, col) = word_left_pos(&state.buffer, state.cursor.row, state.cursor.col);
            state.cursor = Cursor { row, col };
        }

        WordRight => {
            state.selection_anchor = None;
            let (row, col) = word_right_pos(&state.buffer, state.cursor.row, state.cursor.col);
            state.cursor = Cursor { row, col };
        }

        ShiftWordLeft => {
            if state.selection_anchor.is_none() {
                state.selection_anchor = Some(state.cursor);
            }
            let (row, col) = word_left_pos(&state.buffer, state.cursor.row, state.cursor.col);
            state.cursor = Cursor { row, col };
        }

        ShiftWordRight => {
            if state.selection_anchor.is_none() {
                state.selection_anchor = Some(state.cursor);
            }
            let (row, col) = word_right_pos(&state.buffer, state.cursor.row, state.cursor.col);
            state.cursor = Cursor { row, col };
        }

        DeleteWordBefore => {
            let cursor_before = state.cursor;
            let (target_row, target_col) =
                word_left_pos(&state.buffer, cursor_before.row, cursor_before.col);
            if (target_row, target_col) != (cursor_before.row, cursor_before.col) {
                let old_text = state
                    .buffer
                    .text_in_range((target_row, target_col), (cursor_before.row, cursor_before.col));
                let change = build_change_with_cursor(
                    (target_row, target_col),
                    old_text,
                    String::new(),
                    cursor_before,
                    Cursor { row: target_row, col: target_col },
                );
                apply_and_record_change(state, change, false);
            }
        }

        DeleteWordAfter => {
            let cursor_before = state.cursor;
            let (target_row, target_col) =
                word_right_pos(&state.buffer, cursor_before.row, cursor_before.col);
            if (target_row, target_col) != (cursor_before.row, cursor_before.col) {
                let old_text = state
                    .buffer
                    .text_in_range((cursor_before.row, cursor_before.col), (target_row, target_col));
                let change = build_change_with_cursor(
                    (cursor_before.row, cursor_before.col),
                    old_text,
                    String::new(),
                    cursor_before,
                    Cursor { row: cursor_before.row, col: cursor_before.col },
                );
                apply_and_record_change(state, change, false);
            }
        }

        // ── Duplicate line ────────────────────────────────────────────
        DuplicateLine => {
            let row = state.cursor.row;
            let orig_col = state.cursor.col;
            let line = state.buffer.line(row).to_string();
            let insert_col = line.len();
            let change = build_change(
                (row, insert_col),
                String::new(),
                format!("\n{line}"),
                state.cursor,
            );
            apply_and_record_change(state, change, false);
            // Move to the new duplicate row, same column (clamped).
            state.cursor.row = row + 1;
            state.cursor.col = orig_col;
            clamp_cursor_to_buffer(state);
        }

        // ── Settings overlay ──────────────────────────────────────────
        OpenSettings => {
            state.settings_cursor = 0;
            state.keybinding_capture = false;
            state.command_alias_input.clear();
            state.mode = EditorMode::Settings;
        }

        CloseSettings => {
            if let Err(e) = crate::config::save_config(&state.config) {
                state.status_message = Some(format!("Settings save error: {e}"));
            } else {
                state.mode = EditorMode::Normal;
                state.status_message = Some("Settings saved.".to_string());
            }
        }

        SettingsUp => {
            state.settings_cursor = state.settings_cursor.saturating_sub(1);
        }

        SettingsDown => {
            state.settings_cursor =
                (state.settings_cursor + 1).min(crate::ui::settings_overlay::SETTINGS_ITEM_COUNT - 1);
        }

        SettingsToggle => {
            apply_settings_action(state, 0);
        }

        SettingsAdjust(delta) => {
            apply_settings_action(state, delta);
        }

        CloseKeybindings => {
            state.keybinding_capture = false;
            state.mode = EditorMode::Settings;
        }

        KeybindingsUp => {
            state.keybindings_cursor = state.keybindings_cursor.saturating_sub(1);
        }

        KeybindingsDown => {
            let max = crate::shortcuts::configurable_actions().len().saturating_sub(1);
            state.keybindings_cursor = (state.keybindings_cursor + 1).min(max);
        }

        StartKeybindingCapture => {
            state.keybinding_capture = true;
            if let Some(action) = state.selected_keybinding_action() {
                state.status_message = Some(format!(
                    "Press a supported key token for {}",
                    action.name()
                ));
            }
        }

        CancelKeybindingCapture => {
            state.keybinding_capture = false;
            state.status_message = Some("Shortcut capture cancelled.".to_string());
        }

        CaptureKeyToken(token) => {
            state.keybinding_capture = false;
            assign_selected_shortcut(state, token);
        }

        ResetSelectedKeybinding => {
            reset_selected_shortcut(state);
        }

        ResetShortcutOverrides => {
            try_update_shortcuts_config(state, |shortcuts| {
                shortcuts.overrides.clear();
            });
            state.status_message = Some(format!(
                "Shortcut overrides cleared. Active profile: {}.",
                state.active_shortcuts.profile().name()
            ));
        }

        CloseCommandAlias => {
            state.command_alias_input.clear();
            state.mode = EditorMode::Settings;
            state.status_message = Some("Command alias edit cancelled.".to_string());
        }

        CommandAliasChar(c) => {
            state.command_alias_input.push(c);
        }

        CommandAliasBackspace => {
            state.command_alias_input.pop();
        }

        CommandAliasSubmit => {
            submit_command_alias(state);
        }
    }

    Ok(())
}

fn apply_selection_transform<F>(state: &mut EditorState, transform: F)
where
    F: FnOnce(&str) -> String,
{
    if let Some((start, end)) = state.selection_range() {
        let old_text = state.buffer.text_in_range(start, end);
        let new_text = transform(&old_text);
        if old_text != new_text {
            let change = build_change_with_cursor(
                start,
                old_text,
                new_text,
                state.cursor,
                Cursor {
                    row: start.0,
                    col: start.1,
                },
            );
            apply_and_record_change(state, change, false);
            state.selection_anchor = None;
        }
    }
}

fn apply_and_record_change(state: &mut EditorState, change: TextChange, allow_merge: bool) {
    apply_change_forward(state, &change);
    if allow_merge {
        state.history.push(change);
    } else {
        state.history.push_no_merge(change);
    }
}

fn apply_history_entry_forward(state: &mut EditorState, entry: &HistoryEntry) {
    match entry {
        HistoryEntry::Single(change) => apply_change_forward(state, change),
        HistoryEntry::Batch {
            changes,
            cursor_after,
            ..
        } => {
            for change in changes {
                state
                    .buffer
                    .apply_change(change.start, &change.old_text, &change.new_text);
            }
            state.cursor = *cursor_after;
            clamp_cursor_to_buffer(state);
            sync_search_results_after_buffer_change(state);
        }
    }
}

fn apply_history_entry_reverse(state: &mut EditorState, entry: &HistoryEntry) {
    match entry {
        HistoryEntry::Single(change) => apply_change_reverse(state, change),
        HistoryEntry::Batch {
            changes,
            cursor_before,
            ..
        } => {
            for change in changes.iter().rev() {
                state
                    .buffer
                    .apply_change(change.start, &change.new_text, &change.old_text);
            }
            state.cursor = *cursor_before;
            clamp_cursor_to_buffer(state);
            sync_search_results_after_buffer_change(state);
        }
    }
}

fn apply_change_forward(state: &mut EditorState, change: &TextChange) {
    state
        .buffer
        .apply_change(change.start, &change.old_text, &change.new_text);
    state.cursor = change.cursor_after;
    clamp_cursor_to_buffer(state);
    sync_search_results_after_buffer_change(state);
}

fn apply_change_reverse(state: &mut EditorState, change: &TextChange) {
    state
        .buffer
        .apply_change(change.start, &change.new_text, &change.old_text);
    state.cursor = change.cursor_before;
    clamp_cursor_to_buffer(state);
    sync_search_results_after_buffer_change(state);
}

fn build_change(
    start: (usize, usize),
    old_text: String,
    new_text: String,
    cursor_before: Cursor,
) -> TextChange {
    let (row, col) = TextBuffer::position_after(start, &new_text);
    build_change_with_cursor(
        start,
        old_text,
        new_text,
        cursor_before,
        Cursor { row, col },
    )
}

fn build_change_with_cursor(
    start: (usize, usize),
    old_text: String,
    new_text: String,
    cursor_before: Cursor,
    cursor_after: Cursor,
) -> TextChange {
    TextChange {
        start,
        old_text,
        new_text,
        cursor_before,
        cursor_after,
    }
}

fn refresh_search_results(state: &mut EditorState, jump_to_first: bool) {
    state.search_results = crate::editor::search::find_all(
        &state.buffer.lines,
        &state.search_query,
        state.search_case_sensitive,
    );

    if state.search_results.is_empty() {
        state.search_result_idx = 0;
        return;
    }

    if jump_to_first || state.search_result_idx >= state.search_results.len() {
        state.search_result_idx = 0;
    }
    move_cursor_to_search_match(state);
}

fn sync_search_results_after_buffer_change(state: &mut EditorState) {
    if state.search_query.is_empty() {
        state.search_results.clear();
        state.search_result_idx = 0;
        return;
    }

    state.search_results = crate::editor::search::find_all(
        &state.buffer.lines,
        &state.search_query,
        state.search_case_sensitive,
    );
    if state.search_results.is_empty() {
        state.search_result_idx = 0;
    } else {
        state.search_result_idx = state.search_result_idx.min(state.search_results.len() - 1);
    }
}

fn move_cursor_to_search_match(state: &mut EditorState) {
    if let Some(found) = current_search_match(state) {
        state.cursor = Cursor {
            row: found.row,
            col: found.start,
        };
    }
}

fn current_search_match(state: &EditorState) -> Option<SearchMatch> {
    state.search_results.get(state.search_result_idx).copied()
}

// ── Settings helpers ───────────────────────────────────────────────────

/// Toggle or adjust the setting at `state.settings_cursor`.
/// `delta == 0` → toggle bool/open action; `delta == ±1` → adjust numeric/cycle enum.
fn apply_settings_action(state: &mut EditorState, delta: i8) {
    match state.settings_cursor {
        0 => state.config.editor.line_numbers = !state.config.editor.line_numbers,
        1 => state.config.editor.relative_numbers = !state.config.editor.relative_numbers,
        2 => state.config.editor.syntax_highlight = !state.config.editor.syntax_highlight,
        3 => state.config.editor.auto_indent = !state.config.editor.auto_indent,
        4 => state.config.editor.expand_tabs = !state.config.editor.expand_tabs,
        5 => {
            let tw = state.config.editor.tab_width as i16 + delta as i16;
            state.config.editor.tab_width = tw.clamp(1, 8) as u8;
        }
        6 => state.config.editor.indent_guides = !state.config.editor.indent_guides,
        7 => state.config.editor.indent_errors = !state.config.editor.indent_errors,
        8 => {
            const STRATEGIES: &[&str] = &["auto", "internal", "osc52"];
            let cur = STRATEGIES
                .iter()
                .position(|&s| s == state.config.clipboard.strategy)
                .unwrap_or(0);
            let next = ((cur as i16 + delta as i16).rem_euclid(STRATEGIES.len() as i16)) as usize;
            state.config.clipboard.strategy = STRATEGIES[next].to_string();
        }
        9 if delta != 0 => {
            let previous = state.active_shortcuts.profile();
            let next_profile = match state.config.shortcuts.profile {
                ShortcutProfile::Ctrl => ShortcutProfile::Alt,
                ShortcutProfile::Alt => ShortcutProfile::Ctrl,
            };
            try_update_shortcuts_config(state, |shortcuts| {
                shortcuts.profile = next_profile;
            });
            if state.active_shortcuts.profile() != previous {
                state.status_message = Some(format!(
                    "Shortcut profile switched to {}.",
                    state.active_shortcuts.profile().name()
                ));
            }
            return;
        }
        10 if delta == 0 => {
            state.mode = EditorMode::Keybindings;
            state.keybindings_cursor = 0;
            state.keybinding_capture = false;
            state.status_message = Some("Customize shortcuts in the overlay.".to_string());
            return;
        }
        11 if delta == 0 => {
            try_update_shortcuts_config(state, |shortcuts| {
                shortcuts.overrides.clear();
            });
            state.status_message = Some(format!(
                "Shortcut overrides reset for {} profile.",
                state.active_shortcuts.profile().name()
            ));
            return;
        }
        12 if delta == 0 => {
            state.command_alias_input = state.config.command.alias.clone();
            state.mode = EditorMode::CommandAlias;
            state.status_message = Some("Edit your personal command alias.".to_string());
            return;
        }
        13 if delta == 0 => {
            let current_exe = match std::env::current_exe() {
                Ok(path) => path,
                Err(err) => {
                    state.status_message = Some(format!("Alias install error: {err}"));
                    return;
                }
            };
            match crate::command_alias::install_alias(&state.config.command, &current_exe) {
                Ok(message) => state.status_message = Some(message),
                Err(err) => state.status_message = Some(format!("Alias install error: {err}")),
            }
            return;
        }
        14 if delta == 0 => {
            let current_exe = match std::env::current_exe() {
                Ok(path) => path,
                Err(err) => {
                    state.status_message = Some(format!("Alias remove error: {err}"));
                    return;
                }
            };
            match crate::command_alias::remove_alias(&state.config.command, &current_exe) {
                Ok(message) => state.status_message = Some(message),
                Err(err) => state.status_message = Some(format!("Alias remove error: {err}")),
            }
            return;
        }
        _ => {}
    }
    // Apply line-number mode immediately (matches apply_config behaviour).
    state.line_number_mode = crate::editor::EditorState::line_number_mode_for(&state.config);
}

fn submit_command_alias(state: &mut EditorState) {
    let candidate = state.command_alias_input.trim().to_string();
    match try_update_command_alias_config(state, candidate.clone()) {
        Ok(()) => {
            state.command_alias_input.clear();
            state.mode = EditorMode::Settings;
            state.status_message = if candidate.is_empty() {
                Some("Command alias cleared.".to_string())
            } else {
                Some(format!("Command alias set to '{candidate}'. Install it from Settings when ready."))
            };
        }
        Err(err) => {
            state.status_message = Some(err);
        }
    }
}

fn assign_selected_shortcut(state: &mut EditorState, token: KeyToken) {
    let Some(action) = state.selected_keybinding_action() else {
        return;
    };

    if !action.accepts_token(token) {
        state.status_message = Some(format!(
            "{} only accepts {}.",
            action.name(),
            shortcut_policy_hint(action),
        ));
        return;
    }

    let action_id = action.id().to_string();
    let token_value = token.config_value();
    try_update_shortcuts_config(state, |shortcuts| {
        shortcuts.overrides.insert(action_id.clone(), token_value.clone());
    });
    state.status_message = Some(format!(
        "{} → {}",
        action.name(),
        state.active_shortcuts.label_for(action, crate::shortcuts::LabelStyle::Long),
    ));
}

fn reset_selected_shortcut(state: &mut EditorState) {
    let Some(action) = state.selected_keybinding_action() else {
        return;
    };
    let removed = state
        .config
        .shortcuts
        .overrides
        .contains_key(action.id());
    try_update_shortcuts_config(state, |shortcuts| {
        shortcuts.overrides.remove(action.id());
    });
    if removed {
        state.status_message = Some(format!("Reset {} to profile default.", action.name()));
    } else {
        state.status_message = Some(format!("{} already uses the profile default.", action.name()));
    }
}

fn try_update_shortcuts_config<F>(state: &mut EditorState, mut mutate: F)
where
    F: FnMut(&mut crate::config::ShortcutsSection),
{
    let mut candidate = state.config.clone();
    mutate(&mut candidate.shortcuts);

    match ActiveShortcuts::resolve(&candidate.shortcuts) {
        Ok(_) => state.apply_config(candidate),
        Err(errors) => {
            state.status_message = Some(errors.join("; "));
        }
    }
}

fn try_update_command_alias_config(state: &mut EditorState, alias: String) -> std::result::Result<(), String> {
    let mut candidate = state.config.clone();
    candidate.command.alias = alias;
    crate::command_alias::validate_command_section(&candidate.command)
        .map_err(|err| err.to_string())?;
    state.apply_config(candidate);
    Ok(())
}

fn shortcut_policy_hint(action: ShortcutAction) -> &'static str {
    match action {
        ShortcutAction::DeleteWordAfter => "Delete",
        ShortcutAction::WordLeft | ShortcutAction::WordRight => "Left or Right",
        _ => "a-z or comma",
    }
}

// ── Mouse helpers ──────────────────────────────────────────────────────

/// Translate a left-click at absolute terminal coordinates into a cursor move.
fn handle_mouse_click(screen_col: u16, screen_row: u16, state: &mut EditorState) {
    // Ignore clicks outside the editor area.
    if screen_row < state.editor_area_top
        || screen_row >= state.editor_area_bottom
        || screen_col < state.editor_area_left
        || screen_col >= state.editor_area_right
    {
        return;
    }
    let rel_row = (screen_row - state.editor_area_top) as usize;
    let buf_row = (state.viewport.offset_row + rel_row)
        .min(state.buffer.line_count().saturating_sub(1));

    let rel_col = (screen_col - state.editor_area_left) as usize;
    let line = state.buffer.line(buf_row);
    let byte_pos = byte_for_display_col(line, rel_col).min(line.len());

    state.cursor.row = buf_row;
    state.cursor.col = byte_pos;
    state.selection_anchor = None; // clear any selection on click
    state.cursor.clamp(&state.buffer);
}

/// Move one word to the left: skip whitespace left, then skip non-whitespace left.
/// Crosses line boundaries when the cursor is at column 0.
fn word_left_pos(buffer: &TextBuffer, row: usize, col: usize) -> (usize, usize) {
    if buffer.line_count() == 0 {
        return (0, 0);
    }
    let row = row.min(buffer.line_count().saturating_sub(1));
    let col = buffer.clamp_column(row, col);
    if col == 0 {
        if row == 0 {
            return (0, 0);
        }
        let prev_row = row - 1;
        let prev_len = buffer.line(prev_row).len();
        if prev_len == 0 {
            return (prev_row, 0);
        }
        return word_left_pos(buffer, prev_row, prev_len);
    }
    let line = buffer.line(row);
    let mut pos = col;
    while pos > 0 {
        let prev = prev_grapheme_boundary(line, pos);
        if grapheme_slice(line, prev, pos).chars().all(char::is_whitespace) {
            pos = prev;
        } else {
            break;
        }
    }
    while pos > 0 {
        let prev = prev_grapheme_boundary(line, pos);
        if grapheme_slice(line, prev, pos).chars().all(char::is_whitespace) {
            break;
        }
        pos = prev;
    }
    (row, pos)
}

/// Move one word to the right: skip non-whitespace right, then skip whitespace right.
/// Crosses line boundaries when the cursor is at end of line.
fn word_right_pos(buffer: &TextBuffer, row: usize, col: usize) -> (usize, usize) {
    if buffer.line_count() == 0 {
        return (0, 0);
    }
    let row = row.min(buffer.line_count().saturating_sub(1));
    let line = buffer.line(row);
    let col = buffer.clamp_column(row, col);
    if col >= line.len() {
        if row + 1 >= buffer.line_count() {
            return (row, col);
        }
        return (row + 1, 0);
    }
    let mut pos = col;
    while pos < line.len() {
        let next = next_grapheme_boundary(line, pos);
        if grapheme_slice(line, pos, next).chars().all(char::is_whitespace) {
            break;
        }
        pos = next;
    }
    while pos < line.len() {
        let next = next_grapheme_boundary(line, pos);
        if !grapheme_slice(line, pos, next).chars().all(char::is_whitespace) {
            break;
        }
        pos = next;
    }
    (row, pos)
}

fn clamp_cursor_to_buffer(state: &mut EditorState) {
    (state.cursor.row, state.cursor.col) =
        state.buffer.clamp_position(state.cursor.row, state.cursor.col);
}

fn clipboard_copy_status(
    strategy: &str,
    result: crate::clipboard::ClipboardCopyResult,
) -> Option<String> {
    match (strategy, result) {
        ("auto", crate::clipboard::ClipboardCopyResult::Osc52) => {
            Some("System clipboard unavailable; used OSC-52 copy fallback.".to_string())
        }
        ("auto", crate::clipboard::ClipboardCopyResult::Unavailable) => {
            Some("System clipboard unavailable; kept copy in internal clipboard.".to_string())
        }
        ("osc52", crate::clipboard::ClipboardCopyResult::Unavailable) => {
            Some("OSC-52 copy failed; kept copy in internal clipboard.".to_string())
        }
        _ => None,
    }
}

fn clipboard_paste_status(strategy: &str, internal_empty: bool) -> Option<String> {
    if strategy == "internal" {
        return None;
    }

    if strategy == "osc52" {
        return if internal_empty {
            Some("OSC-52 mode does not support paste; internal clipboard is empty.".to_string())
        } else {
            Some("OSC-52 mode does not support paste; used internal clipboard.".to_string())
        };
    }

    if internal_empty {
        Some("System clipboard unavailable; nothing to paste.".to_string())
    } else {
        Some("System clipboard unavailable; used internal clipboard.".to_string())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::EditorConfig;
    use crate::editor::TextBuffer;
    use crossterm::event::{KeyModifiers, MouseEvent};

    fn state_with_lines(lines: &[&str]) -> EditorState {
        let mut state = EditorState::new_empty();
        state.buffer = TextBuffer::from_lines(lines.iter().map(|line| line.to_string()).collect());
        state.config = EditorConfig::default();
        state.editor_area_left = 2;
        state.editor_area_top = 1;
        state.editor_area_right = 22;
        state.editor_area_bottom = 4;
        state
    }

    #[test]
    fn undo_redo_multiline_paste_round_trip() {
        let mut state = state_with_lines(&["hello"]);
        state.cursor = Cursor { row: 0, col: 5 };
        state.clipboard = "\nworld".to_string();
        // Force internal clipboard so the test is not affected by the system clipboard.
        state.config.clipboard.strategy = "internal".to_string();

        apply_action(&mut state, AppAction::Paste).expect("paste");
        assert_eq!(state.buffer.lines, vec!["hello".to_string(), "world".to_string()]);

        apply_action(&mut state, AppAction::Undo).expect("undo");
        assert_eq!(state.buffer.lines, vec!["hello".to_string()]);

        apply_action(&mut state, AppAction::Redo).expect("redo");
        assert_eq!(state.buffer.lines, vec!["hello".to_string(), "world".to_string()]);
    }

    #[test]
    fn undo_redo_newline_round_trip() {
        let mut state = state_with_lines(&["abcd"]);
        state.cursor = Cursor { row: 0, col: 2 };

        apply_action(&mut state, AppAction::InsertNewline).expect("newline");
        assert_eq!(state.buffer.lines, vec!["ab".to_string(), "cd".to_string()]);

        apply_action(&mut state, AppAction::Undo).expect("undo");
        assert_eq!(state.buffer.lines, vec!["abcd".to_string()]);

        apply_action(&mut state, AppAction::Redo).expect("redo");
        assert_eq!(state.buffer.lines, vec!["ab".to_string(), "cd".to_string()]);
    }

    #[test]
    fn undo_redo_join_lines_from_backspace_round_trip() {
        let mut state = state_with_lines(&["foo", "bar"]);
        state.cursor = Cursor { row: 1, col: 0 };

        apply_action(&mut state, AppAction::DeleteBefore).expect("join");
        assert_eq!(state.buffer.lines, vec!["foobar".to_string()]);

        apply_action(&mut state, AppAction::Undo).expect("undo");
        assert_eq!(state.buffer.lines, vec!["foo".to_string(), "bar".to_string()]);

        apply_action(&mut state, AppAction::Redo).expect("redo");
        assert_eq!(state.buffer.lines, vec!["foobar".to_string()]);
    }

    #[test]
    fn undo_redo_join_lines_from_delete_round_trip() {
        let mut state = state_with_lines(&["foo", "bar"]);
        state.cursor = Cursor { row: 0, col: 3 };

        apply_action(&mut state, AppAction::DeleteAt).expect("join");
        assert_eq!(state.buffer.lines, vec!["foobar".to_string()]);

        apply_action(&mut state, AppAction::Undo).expect("undo");
        assert_eq!(state.buffer.lines, vec!["foo".to_string(), "bar".to_string()]);

        apply_action(&mut state, AppAction::Redo).expect("redo");
        assert_eq!(state.buffer.lines, vec!["foobar".to_string()]);
    }

    #[test]
    fn undo_redo_multiline_cut_round_trip() {
        let mut state = state_with_lines(&["ab", "cd"]);
        state.selection_anchor = Some(Cursor { row: 0, col: 1 });
        state.cursor = Cursor { row: 1, col: 1 };

        apply_action(&mut state, AppAction::Cut).expect("cut");
        assert_eq!(state.buffer.lines, vec!["ad".to_string()]);

        apply_action(&mut state, AppAction::Undo).expect("undo");
        assert_eq!(state.buffer.lines, vec!["ab".to_string(), "cd".to_string()]);

        apply_action(&mut state, AppAction::Redo).expect("redo");
        assert_eq!(state.buffer.lines, vec!["ad".to_string()]);
    }

    #[test]
    fn search_navigation_uses_match_offsets() {
        let mut state = state_with_lines(&["zażółć test zażółć"]);

        apply_action(&mut state, AppAction::StartSearch).expect("start search");
        for ch in "zażółć".chars() {
            apply_action(&mut state, AppAction::SearchChar(ch)).expect("search char");
        }

        assert_eq!(state.cursor, Cursor { row: 0, col: 0 });

        apply_action(&mut state, AppAction::SearchNext).expect("next");
        // "zażółć" = 10 bytes (z=1, a=1, ż=2, ó=2, ł=2, ć=2); " test " = 6 bytes → second match at byte 16.
        assert_eq!(state.cursor, Cursor { row: 0, col: 16 });

        apply_action(&mut state, AppAction::SearchPrev).expect("prev");
        assert_eq!(state.cursor, Cursor { row: 0, col: 0 });
    }

    #[test]
    fn replace_all_clamps_cursor_to_valid_boundary() {
        let mut state = state_with_lines(&["zażółć zażółć"]);
        state.cursor = Cursor { row: 0, col: "zażółć zażółć".len() };
        state.search_query = "zażółć".to_string();
        state.search_results = crate::editor::search::find_all(&state.buffer.lines, "zażółć", false);
        state.replace_query = "x".to_string();

        apply_action(&mut state, AppAction::ReplaceAll).expect("replace all");

        let line = state.buffer.line(0);
        assert!(state.cursor.col <= line.len());
        assert!(line.is_char_boundary(state.cursor.col));
    }

    #[test]
    fn replace_all_is_undoable_as_one_batch() {
        let mut state = state_with_lines(&["alpha beta alpha"]);
        state.search_query = "alpha".to_string();
        state.search_results = crate::editor::search::find_all(&state.buffer.lines, "alpha", true);
        state.replace_query = "x".to_string();

        apply_action(&mut state, AppAction::ReplaceAll).expect("replace all");
        assert_eq!(state.buffer.lines, vec!["x beta x".to_string()]);

        apply_action(&mut state, AppAction::Undo).expect("undo");
        assert_eq!(state.buffer.lines, vec!["alpha beta alpha".to_string()]);
    }

    #[test]
    fn insert_newline_clamps_invalid_utf8_cursor_before_slicing_indent() {
        let mut state = state_with_lines(&["ąż test"]);
        state.cursor = Cursor { row: 0, col: 1 };
        state.config.editor.auto_indent = true;

        apply_action(&mut state, AppAction::InsertNewline).expect("newline");

        assert_eq!(state.buffer.lines, vec!["".to_string(), "ąż test".to_string()]);
    }

    #[test]
    fn delete_before_removes_whole_grapheme_cluster() {
        let mut state = state_with_lines(&["A👨‍👩‍👧‍👦B"]);
        state.cursor = Cursor {
            row: 0,
            col: "A👨‍👩‍👧‍👦".len(),
        };

        apply_action(&mut state, AppAction::DeleteBefore).expect("delete before");

        assert_eq!(state.buffer.lines, vec!["AB".to_string()]);
        assert_eq!(state.cursor, Cursor { row: 0, col: 1 });
    }

    // ── Word navigation helpers ───────────────────────────────────────────

    fn buffer(v: &[&str]) -> TextBuffer {
        TextBuffer::from_lines(v.iter().map(|s| s.to_string()).collect())
    }

    #[test]
    fn word_left_from_middle_of_word() {
        // "hello world" — cursor at 'r' of "world" (col 8) → start of "world" (col 6)
        let buffer = buffer(&["hello world"]);
        assert_eq!(word_left_pos(&buffer, 0, 8), (0, 6));
    }

    #[test]
    fn word_left_from_start_of_word_skips_to_prev() {
        // cursor at 'w' (col 6) → start of "hello" (col 0)
        let buffer = buffer(&["hello world"]);
        assert_eq!(word_left_pos(&buffer, 0, 6), (0, 0));
    }

    #[test]
    fn word_left_at_buffer_start_stays() {
        let buffer = buffer(&["hello"]);
        assert_eq!(word_left_pos(&buffer, 0, 0), (0, 0));
    }

    #[test]
    fn word_left_crosses_line_boundary() {
        // line 0: "hello", line 1: "" cursor col 0 → (0, 0) because prev line is empty
        let buffer = buffer(&["hello", ""]);
        assert_eq!(word_left_pos(&buffer, 1, 0), (0, 0));
    }

    #[test]
    fn word_left_crosses_line_boundary_to_word() {
        // line 0: "hello world", line 1: cursor col 0 → start of "world" on line 0
        let buffer = buffer(&["hello world", "next"]);
        assert_eq!(word_left_pos(&buffer, 1, 0), (0, 6));
    }

    #[test]
    fn word_right_from_middle_of_word() {
        // "hello world" — cursor at 'e' (col 1) → start of "world" (col 6)
        let buffer = buffer(&["hello world"]);
        assert_eq!(word_right_pos(&buffer, 0, 1), (0, 6));
    }

    #[test]
    fn word_right_from_whitespace() {
        // cursor at space (col 5) → start of "world" (col 6)
        let buffer = buffer(&["hello world"]);
        assert_eq!(word_right_pos(&buffer, 0, 5), (0, 6));
    }

    #[test]
    fn word_right_at_end_of_line_crosses_boundary() {
        // cursor at end of line 0 → (1, 0)
        let buffer = buffer(&["hello", "world"]);
        assert_eq!(word_right_pos(&buffer, 0, 5), (1, 0));
    }

    #[test]
    fn word_right_at_buffer_end_stays() {
        let buffer = buffer(&["hello"]);
        assert_eq!(word_right_pos(&buffer, 0, 5), (0, 5));
    }

    // ── Word navigation actions ───────────────────────────────────────────

    #[test]
    fn word_left_action_moves_cursor() {
        let mut state = state_with_lines(&["hello world"]);
        state.cursor = Cursor { row: 0, col: 8 }; // inside "world"
        apply_action(&mut state, AppAction::WordLeft).expect("word left");
        assert_eq!(state.cursor, Cursor { row: 0, col: 6 });
        assert!(state.selection_anchor.is_none());
    }

    #[test]
    fn word_right_action_moves_cursor() {
        let mut state = state_with_lines(&["hello world"]);
        state.cursor = Cursor { row: 0, col: 0 }; // start of "hello"
        apply_action(&mut state, AppAction::WordRight).expect("word right");
        assert_eq!(state.cursor, Cursor { row: 0, col: 6 }); // start of "world"
    }

    #[test]
    fn shift_word_right_extends_selection() {
        let mut state = state_with_lines(&["hello world"]);
        state.cursor = Cursor { row: 0, col: 0 };
        apply_action(&mut state, AppAction::ShiftWordRight).expect("shift word right");
        assert_eq!(state.selection_anchor, Some(Cursor { row: 0, col: 0 }));
        assert_eq!(state.cursor, Cursor { row: 0, col: 6 });
    }

    #[test]
    fn delete_word_before_removes_previous_word() {
        let mut state = state_with_lines(&["hello world"]);
        state.cursor = Cursor { row: 0, col: 11 }; // end of "world"
        apply_action(&mut state, AppAction::DeleteWordBefore).expect("delete word before");
        assert_eq!(state.buffer.lines, vec!["hello ".to_string()]);
        assert_eq!(state.cursor, Cursor { row: 0, col: 6 });
    }

    #[test]
    fn delete_word_after_removes_next_word() {
        let mut state = state_with_lines(&["hello world"]);
        state.cursor = Cursor { row: 0, col: 6 }; // start of "world"
        apply_action(&mut state, AppAction::DeleteWordAfter).expect("delete word after");
        assert_eq!(state.buffer.lines, vec!["hello ".to_string()]);
        assert_eq!(state.cursor, Cursor { row: 0, col: 6 });
    }

    #[test]
    fn delete_word_before_is_undoable() {
        let mut state = state_with_lines(&["hello world"]);
        state.cursor = Cursor { row: 0, col: 11 };
        apply_action(&mut state, AppAction::DeleteWordBefore).expect("delete");
        apply_action(&mut state, AppAction::Undo).expect("undo");
        assert_eq!(state.buffer.lines, vec!["hello world".to_string()]);
    }

    #[test]
    fn delete_word_before_at_buffer_start_does_nothing() {
        let mut state = state_with_lines(&["hello"]);
        state.cursor = Cursor { row: 0, col: 0 };
        apply_action(&mut state, AppAction::DeleteWordBefore).expect("noop");
        assert_eq!(state.buffer.lines, vec!["hello".to_string()]);
    }

    #[test]
    fn delete_word_after_at_buffer_end_does_nothing() {
        let mut state = state_with_lines(&["hello"]);
        state.cursor = Cursor { row: 0, col: 5 };
        apply_action(&mut state, AppAction::DeleteWordAfter).expect("noop");
        assert_eq!(state.buffer.lines, vec!["hello".to_string()]);
    }

    // ── Auto-indent tests ─────────────────────────────────────────────────

    #[test]
    fn auto_indent_preserves_leading_spaces() {
        let mut state = state_with_lines(&["    hello"]);
        state.cursor = Cursor { row: 0, col: 9 }; // end of line
        state.config.editor.auto_indent = true;
        apply_action(&mut state, AppAction::InsertNewline).expect("newline");
        assert_eq!(state.buffer.lines[1], "    ");
    }

    #[test]
    fn auto_indent_empty_line_no_indent() {
        let mut state = state_with_lines(&[""]);
        state.cursor = Cursor { row: 0, col: 0 };
        state.config.editor.auto_indent = true;
        apply_action(&mut state, AppAction::InsertNewline).expect("newline");
        assert_eq!(state.buffer.lines[1], "");
    }

    #[test]
    fn auto_indent_disabled_no_indent() {
        let mut state = state_with_lines(&["    hello"]);
        state.cursor = Cursor { row: 0, col: 9 };
        state.config.editor.auto_indent = false;
        apply_action(&mut state, AppAction::InsertNewline).expect("newline");
        assert_eq!(state.buffer.lines[1], "");
    }

    // ── Expand-tabs tests ─────────────────────────────────────────────────

    #[test]
    fn expand_tabs_inserts_spaces() {
        let mut state = state_with_lines(&[""]);
        state.cursor = Cursor { row: 0, col: 0 };
        state.config.editor.expand_tabs = true;
        state.config.editor.tab_width = 4;
        apply_action(&mut state, AppAction::InsertChar('\t')).expect("tab");
        assert_eq!(state.buffer.lines[0], "    ");
    }

    #[test]
    fn expand_tabs_disabled_inserts_tab() {
        let mut state = state_with_lines(&[""]);
        state.cursor = Cursor { row: 0, col: 0 };
        state.config.editor.expand_tabs = false;
        apply_action(&mut state, AppAction::InsertChar('\t')).expect("tab");
        assert_eq!(state.buffer.lines[0], "\t");
    }

    // ── Duplicate-line tests ──────────────────────────────────────────────

    #[test]
    fn duplicate_line_copies_content() {
        let mut state = state_with_lines(&["hello"]);
        state.cursor = Cursor { row: 0, col: 2 };
        apply_action(&mut state, AppAction::DuplicateLine).expect("dup");
        assert_eq!(state.buffer.lines, vec!["hello".to_string(), "hello".to_string()]);
    }

    #[test]
    fn duplicate_line_cursor_on_new_line() {
        let mut state = state_with_lines(&["hello"]);
        state.cursor = Cursor { row: 0, col: 3 };
        apply_action(&mut state, AppAction::DuplicateLine).expect("dup");
        assert_eq!(state.cursor.row, 1);
        assert_eq!(state.cursor.col, 3);
    }

    #[test]
    fn duplicate_line_is_undoable() {
        let mut state = state_with_lines(&["hello"]);
        state.cursor = Cursor { row: 0, col: 0 };
        apply_action(&mut state, AppAction::DuplicateLine).expect("dup");
        assert_eq!(state.buffer.line_count(), 2);
        apply_action(&mut state, AppAction::Undo).expect("undo");
        assert_eq!(state.buffer.line_count(), 1);
        assert_eq!(state.buffer.lines[0], "hello");
    }

    // ── Indent-error detection tests ──────────────────────────────────────

    #[test]
    fn has_indent_error_yaml_wrong_indent() {
        use crate::syntax::{has_indent_error, Language};
        assert!(has_indent_error("   key: val", 4, Language::Yaml)); // 3 spaces, not % 4
    }

    #[test]
    fn has_indent_error_yaml_correct_indent() {
        use crate::syntax::{has_indent_error, Language};
        assert!(!has_indent_error("    key: val", 4, Language::Yaml)); // 4 spaces = ok
    }

    #[test]
    fn has_indent_error_mixed_tabs_spaces() {
        use crate::syntax::{has_indent_error, Language};
        assert!(has_indent_error("\t  key: val", 4, Language::Yaml)); // mix
    }

    #[test]
    fn has_indent_error_rust_not_checked() {
        use crate::syntax::{has_indent_error, Language};
        assert!(!has_indent_error("   fn foo()", 4, Language::Rust)); // Rust not checked
    }

    #[test]
    fn mouse_click_inside_editor_moves_cursor() {
        let mut state = state_with_lines(&["hello"]);

        handle_event(
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: 4,
                row: 1,
                modifiers: KeyModifiers::NONE,
            }),
            &mut state,
        )
        .expect("mouse event");

        assert_eq!(state.cursor, Cursor { row: 0, col: 2 });
    }

    #[test]
    fn mouse_click_on_wide_grapheme_uses_grapheme_boundaries() {
        let mut state = state_with_lines(&["A👨‍👩‍👧‍👦B"]);

        handle_event(
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: 4,
                row: 1,
                modifiers: KeyModifiers::NONE,
            }),
            &mut state,
        )
        .expect("mouse event");

        assert_eq!(
            state.cursor,
            Cursor {
                row: 0,
                col: "A👨‍👩‍👧‍👦".len(),
            }
        );
    }

    #[test]
    fn ctrl_t_opens_settings_overlay() {
        let mut state = state_with_lines(&["hello"]);

        handle_event(
            Event::Key(crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char('t'),
                KeyModifiers::CONTROL,
            )),
            &mut state,
        )
        .expect("ctrl+t");

        assert_eq!(state.mode, EditorMode::Settings);
    }

    #[test]
    fn settings_open_command_alias_overlay() {
        let mut state = state_with_lines(&["hello"]);
        state.mode = EditorMode::Settings;
        state.settings_cursor = 12;

        apply_action(&mut state, AppAction::SettingsToggle).expect("open alias overlay");

        assert_eq!(state.mode, EditorMode::CommandAlias);
    }

    #[test]
    fn command_alias_submit_updates_config() {
        let mut state = state_with_lines(&["hello"]);
        state.mode = EditorMode::CommandAlias;
        state.command_alias_input = "cct".to_string();

        apply_action(&mut state, AppAction::CommandAliasSubmit).expect("submit alias");

        assert_eq!(state.mode, EditorMode::Settings);
        assert_eq!(state.config.command.alias, "cct");
    }

    #[test]
    fn mouse_click_outside_editor_does_not_move_cursor() {
        let mut state = state_with_lines(&["hello"]);
        state.cursor = Cursor { row: 0, col: 4 };

        handle_event(
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: 4,
                row: 4,
                modifiers: KeyModifiers::NONE,
            }),
            &mut state,
        )
        .expect("mouse event");

        assert_eq!(state.cursor, Cursor { row: 0, col: 4 });
    }

    #[test]
    fn mouse_click_in_overlay_mode_does_not_move_cursor() {
        let mut state = state_with_lines(&["hello"]);
        state.mode = EditorMode::Help;
        state.cursor = Cursor { row: 0, col: 1 };

        handle_event(
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: 6,
                row: 2,
                modifiers: KeyModifiers::NONE,
            }),
            &mut state,
        )
        .expect("mouse event");

        assert_eq!(state.cursor, Cursor { row: 0, col: 1 });
    }

    #[test]
    fn clipboard_copy_status_reports_osc52_fallback_for_auto() {
        let status = clipboard_copy_status(
            "auto",
            crate::clipboard::ClipboardCopyResult::Osc52,
        );

        assert_eq!(
            status.as_deref(),
            Some("System clipboard unavailable; used OSC-52 copy fallback.")
        );
    }

    #[test]
    fn clipboard_paste_status_reports_internal_fallback_for_auto() {
        let status = clipboard_paste_status("auto", false);

        assert_eq!(
            status.as_deref(),
            Some("System clipboard unavailable; used internal clipboard.")
        );
    }
}
