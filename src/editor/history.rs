use crate::editor::cursor::Cursor;

const MAX_UNDO_HISTORY: usize = 10_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextChange {
    pub start: (usize, usize),
    pub old_text: String,
    pub new_text: String,
    pub cursor_before: Cursor,
    pub cursor_after: Cursor,
}

pub struct History {
    pub undo_stack: Vec<TextChange>,
    pub redo_stack: Vec<TextChange>,
    pub merge_enabled: bool,
}

impl History {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            merge_enabled: true,
        }
    }

    pub fn push(&mut self, change: TextChange) {
        if self.merge_enabled
            && let Some(prev) = self.undo_stack.last_mut()
            && can_merge(prev, &change)
        {
            prev.new_text.push_str(&change.new_text);
            prev.cursor_after = change.cursor_after;
            self.redo_stack.clear();
            return;
        }

        self.undo_stack.push(change);
        if self.undo_stack.len() > MAX_UNDO_HISTORY {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
        self.merge_enabled = true;
    }

    pub fn push_no_merge(&mut self, change: TextChange) {
        let saved = self.merge_enabled;
        self.merge_enabled = false;
        self.push(change);
        self.merge_enabled = saved;
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

fn can_merge(previous: &TextChange, next: &TextChange) -> bool {
    previous.old_text.is_empty()
        && next.old_text.is_empty()
        && !previous.new_text.contains('\n')
        && !next.new_text.contains('\n')
        && previous.start.0 == next.start.0
        && previous.cursor_after == next.cursor_before
        && crate::editor::buffer::TextBuffer::position_after(previous.start, &previous.new_text)
            == next.start
}
