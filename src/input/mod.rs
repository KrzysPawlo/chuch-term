pub mod keybindings;

pub use keybindings::{map_key, AppAction};

use anyhow::Result;
use crossterm::event::{Event, KeyEventKind};

use crate::editor::buffer::{next_char_boundary, prev_char_boundary};
use crate::editor::history::TextChange;
use crate::editor::{Cursor, EditorMode, EditorState, LineNumberMode, SearchMatch, TextBuffer};

/// Translate a crossterm Event into an AppAction and apply it to the editor state.
pub fn handle_event(event: Event, state: &mut EditorState) -> Result<()> {
    let action = match event {
        Event::Key(key_event) => {
            if key_event.kind != KeyEventKind::Press {
                return Ok(());
            }
            map_key(key_event, state.mode)
        }
        Event::Resize(_, _) => return Ok(()),
        _ => return Ok(()),
    };

    apply_action(state, action)
}

/// Apply a semantic action to the editor state.
fn apply_action(state: &mut EditorState, action: AppAction) -> Result<()> {
    use AppAction::*;

    state.status_message = None;

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

        InsertChar(ch) => {
            let cursor_before = state.cursor;
            let change = build_change(
                (cursor_before.row, cursor_before.col),
                String::new(),
                ch.to_string(),
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
                let start_col = prev_char_boundary(line, col);
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
                let end = next_char_boundary(line, col);
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
            let change = build_change(
                (cursor_before.row, cursor_before.col),
                String::new(),
                "\n".to_string(),
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
            if let Some(change) = state.history.undo_stack.pop() {
                apply_change_reverse(state, &change);
                state.history.redo_stack.push(change);
            }
        }

        Redo => {
            if let Some(change) = state.history.redo_stack.pop() {
                apply_change_forward(state, &change);
                state.history.undo_stack.push(change);
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
                crate::clipboard::copy_to_clipboard(&text, &strategy);
                state.clipboard = text;
            }
        }

        Cut => {
            if let Some((start, end)) = state.selection_range() {
                let text = state.buffer.text_in_range(start, end);
                let strategy = state.config.clipboard.strategy.clone();
                crate::clipboard::copy_to_clipboard(&text, &strategy);
                state.clipboard = text.clone();

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
            let text = crate::clipboard::paste_from_clipboard(&strategy)
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| state.clipboard.clone());
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
            if state.palette_cursor > 0 {
                state.palette_cursor -= 1;
            }
        }

        PaletteDown => {
            if !state.palette_matches.is_empty()
                && state.palette_cursor + 1 < state.palette_matches.len()
            {
                state.palette_cursor += 1;
            }
        }

        PaletteSubmit => {
            let idx = state.palette_matches.get(state.palette_cursor).copied();
            state.mode = EditorMode::Normal;
            if let Some(command_index) = idx {
                if let Some(command) = crate::commands::COMMANDS.get(command_index) {
                    return apply_action(state, command.action.clone());
                }
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
                let old_text = state.buffer.full_text();
                let new_text = build_replace_all_text(
                    &state.buffer,
                    &state.search_results,
                    &state.replace_query,
                );
                let replacements = state.search_results.len();

                if old_text != new_text {
                    let change =
                        build_change_with_cursor((0, 0), old_text, new_text, cursor_before, cursor_before);
                    apply_and_record_change(state, change, false);
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
            let (row, col) = word_left_pos(&state.buffer.lines, state.cursor.row, state.cursor.col);
            state.cursor = Cursor { row, col };
        }

        WordRight => {
            state.selection_anchor = None;
            let (row, col) = word_right_pos(&state.buffer.lines, state.cursor.row, state.cursor.col);
            state.cursor = Cursor { row, col };
        }

        ShiftWordLeft => {
            if state.selection_anchor.is_none() {
                state.selection_anchor = Some(state.cursor);
            }
            let (row, col) = word_left_pos(&state.buffer.lines, state.cursor.row, state.cursor.col);
            state.cursor = Cursor { row, col };
        }

        ShiftWordRight => {
            if state.selection_anchor.is_none() {
                state.selection_anchor = Some(state.cursor);
            }
            let (row, col) = word_right_pos(&state.buffer.lines, state.cursor.row, state.cursor.col);
            state.cursor = Cursor { row, col };
        }

        DeleteWordBefore => {
            let cursor_before = state.cursor;
            let (target_row, target_col) =
                word_left_pos(&state.buffer.lines, cursor_before.row, cursor_before.col);
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
                word_right_pos(&state.buffer.lines, cursor_before.row, cursor_before.col);
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

fn build_replace_all_text(
    buffer: &TextBuffer,
    matches: &[SearchMatch],
    replacement: &str,
) -> String {
    let mut full_text = buffer.full_text();
    let mut absolute_ranges: Vec<(usize, usize)> = matches
        .iter()
        .map(|found| {
            (
                buffer.absolute_offset(found.row, found.start),
                buffer.absolute_offset(found.row, found.end),
            )
        })
        .collect();
    absolute_ranges.sort_by(|a, b| b.0.cmp(&a.0));

    for (start, end) in absolute_ranges {
        full_text.replace_range(start..end, replacement);
    }

    full_text
}

/// Move one word to the left: skip whitespace left, then skip non-whitespace left.
/// Crosses line boundaries when the cursor is at column 0.
fn word_left_pos(lines: &[String], row: usize, col: usize) -> (usize, usize) {
    if col == 0 {
        if row == 0 {
            return (0, 0);
        }
        let prev_row = row - 1;
        let prev_len = lines[prev_row].len();
        if prev_len == 0 {
            return (prev_row, 0);
        }
        return word_left_pos(lines, prev_row, prev_len);
    }
    let line = &lines[row];
    let mut pos = col;
    // Skip whitespace left
    while pos > 0 {
        let prev = prev_char_boundary(line, pos);
        if line[prev..pos].chars().next().is_some_and(|c| c.is_whitespace()) {
            pos = prev;
        } else {
            break;
        }
    }
    // Skip non-whitespace left
    while pos > 0 {
        let prev = prev_char_boundary(line, pos);
        if line[prev..pos].chars().next().is_some_and(|c| c.is_whitespace()) {
            break;
        }
        pos = prev;
    }
    (row, pos)
}

/// Move one word to the right: skip non-whitespace right, then skip whitespace right.
/// Crosses line boundaries when the cursor is at end of line.
fn word_right_pos(lines: &[String], row: usize, col: usize) -> (usize, usize) {
    let line = &lines[row];
    if col >= line.len() {
        if row + 1 >= lines.len() {
            return (row, col);
        }
        return (row + 1, 0);
    }
    let mut pos = col;
    // Skip non-whitespace right
    while pos < line.len() {
        if line[pos..].chars().next().is_some_and(|c| c.is_whitespace()) {
            break;
        }
        pos = next_char_boundary(line, pos);
    }
    // Skip whitespace right
    while pos < line.len() {
        if line[pos..].chars().next().is_some_and(|c| !c.is_whitespace()) {
            break;
        }
        pos = next_char_boundary(line, pos);
    }
    (row, pos)
}

fn clamp_cursor_to_buffer(state: &mut EditorState) {
    let max_row = state.buffer.line_count().saturating_sub(1);
    state.cursor.row = state.cursor.row.min(max_row);

    let line = state.buffer.line(state.cursor.row);
    let mut col = state.cursor.col.min(line.len());
    while col > 0 && !line.is_char_boundary(col) {
        col -= 1;
    }
    state.cursor.col = col;
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::EditorConfig;
    use crate::editor::TextBuffer;

    fn state_with_lines(lines: &[&str]) -> EditorState {
        let mut state = EditorState::new_empty();
        state.buffer = TextBuffer {
            lines: lines.iter().map(|line| line.to_string()).collect(),
            dirty: false,
            file_path: None,
        };
        state.config = EditorConfig::default();
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

    // ── Word navigation helpers ───────────────────────────────────────────

    fn lines(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn word_left_from_middle_of_word() {
        // "hello world" — cursor at 'r' of "world" (col 8) → start of "world" (col 6)
        let ls = lines(&["hello world"]);
        assert_eq!(word_left_pos(&ls, 0, 8), (0, 6));
    }

    #[test]
    fn word_left_from_start_of_word_skips_to_prev() {
        // cursor at 'w' (col 6) → start of "hello" (col 0)
        let ls = lines(&["hello world"]);
        assert_eq!(word_left_pos(&ls, 0, 6), (0, 0));
    }

    #[test]
    fn word_left_at_buffer_start_stays() {
        let ls = lines(&["hello"]);
        assert_eq!(word_left_pos(&ls, 0, 0), (0, 0));
    }

    #[test]
    fn word_left_crosses_line_boundary() {
        // line 0: "hello", line 1: "" cursor col 0 → (0, 0) because prev line is empty
        let ls = lines(&["hello", ""]);
        assert_eq!(word_left_pos(&ls, 1, 0), (0, 0));
    }

    #[test]
    fn word_left_crosses_line_boundary_to_word() {
        // line 0: "hello world", line 1: cursor col 0 → start of "world" on line 0
        let ls = lines(&["hello world", "next"]);
        assert_eq!(word_left_pos(&ls, 1, 0), (0, 6));
    }

    #[test]
    fn word_right_from_middle_of_word() {
        // "hello world" — cursor at 'e' (col 1) → start of "world" (col 6)
        let ls = lines(&["hello world"]);
        assert_eq!(word_right_pos(&ls, 0, 1), (0, 6));
    }

    #[test]
    fn word_right_from_whitespace() {
        // cursor at space (col 5) → start of "world" (col 6)
        let ls = lines(&["hello world"]);
        assert_eq!(word_right_pos(&ls, 0, 5), (0, 6));
    }

    #[test]
    fn word_right_at_end_of_line_crosses_boundary() {
        // cursor at end of line 0 → (1, 0)
        let ls = lines(&["hello", "world"]);
        assert_eq!(word_right_pos(&ls, 0, 5), (1, 0));
    }

    #[test]
    fn word_right_at_buffer_end_stays() {
        let ls = lines(&["hello"]);
        assert_eq!(word_right_pos(&ls, 0, 5), (0, 5));
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
}
