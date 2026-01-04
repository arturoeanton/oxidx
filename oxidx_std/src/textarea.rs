//! TextArea Component
//!
//! A multiline text editor with:
//! - Multiple line support with scrolling
//! - Cursor navigation (arrows, Home, End, Page Up/Down)
//! - Text selection (mouse and keyboard)
//! - Line numbers (optional)
//! - Syntax highlighting ready (via rich text)
//! - Word wrap (optional)
//! - Clipboard support
//! - IME support
//! - Undo/Redo (basic)

use glam::Vec2;
use oxidx_core::component::OxidXComponent;
use oxidx_core::events::{KeyCode, OxidXEvent};
use oxidx_core::layout::LayoutProps;
use oxidx_core::primitives::{Color, Rect, TextStyle};
use oxidx_core::renderer::Renderer;
use oxidx_core::style::{ComponentState, InteractiveStyle, Style};
use oxidx_core::OxidXContext;
use std::cell::Cell;
use winit::window::CursorIcon;

/// Cursor position in a text document (line, column)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CursorPosition {
    /// Line index (0-based)
    pub line: usize,
    /// Column index (character position in line, 0-based)
    pub col: usize,
}

impl CursorPosition {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}

/// A multiline text editor component.
pub struct TextArea {
    // === Layout ===
    bounds: Rect,
    layout: LayoutProps,

    // === Content ===
    /// Lines of text
    lines: Vec<String>,
    placeholder: String,

    // === Styling ===
    style: InteractiveStyle,
    text_style: TextStyle,
    line_number_color: Color,

    // === Configuration ===
    id: String,
    show_line_numbers: bool,
    word_wrap: bool,
    tab_size: usize,
    read_only: bool,

    // === State ===
    is_focused: bool,
    is_hovered: bool,

    // === Cursor ===
    cursor: CursorPosition,
    cursor_blink_timer: f32,
    cursor_visible: bool,

    // === Selection ===
    selection_anchor: Option<CursorPosition>,
    is_selecting: bool,

    // === Scrolling ===
    scroll_offset: Vec2,
    /// Number of visible lines
    visible_lines: usize,

    // === Undo/Redo ===
    undo_stack: Vec<UndoAction>,
    redo_stack: Vec<UndoAction>,

    // === IME ===
    ime_preedit: String,

    // === Cached Values ===
    cached_cursor_x: Cell<f32>,
    cached_cursor_y: Cell<f32>,
    line_height: f32,
    gutter_width: f32,
}

/// Action for undo/redo
#[derive(Debug, Clone)]
enum UndoAction {
    Insert {
        position: CursorPosition,
        text: String,
    },
    Delete {
        position: CursorPosition,
        text: String,
    },
    Replace {
        position: CursorPosition,
        old_text: String,
        new_text: String,
    },
}

impl TextArea {
    /// Creates a new empty TextArea.
    pub fn new() -> Self {
        let border_color = Color::new(0.3, 0.3, 0.35, 1.0);
        let focus_color = Color::new(0.2, 0.5, 0.9, 1.0);
        let bg = Color::new(0.08, 0.08, 0.1, 1.0);
        let text_white = Color::WHITE;

        let idle = Style::new()
            .bg_solid(bg)
            .border(1.0, border_color)
            .rounded(4.0)
            .text_color(text_white);

        let hover = Style::new()
            .bg_solid(Color::new(0.1, 0.1, 0.12, 1.0))
            .border(1.0, Color::new(0.4, 0.4, 0.45, 1.0))
            .rounded(4.0)
            .text_color(text_white);

        let focused = Style::new()
            .bg_solid(bg)
            .border(2.0, focus_color)
            .rounded(4.0)
            .text_color(text_white);

        Self {
            bounds: Rect::default(),
            layout: LayoutProps::default().with_padding(8.0),
            lines: vec![String::new()],
            placeholder: String::new(),
            style: InteractiveStyle {
                idle,
                hover,
                pressed: focused,
                disabled: idle,
            },
            text_style: TextStyle::new(14.0).with_color(Color::WHITE),
            line_number_color: Color::new(0.5, 0.5, 0.55, 1.0),
            id: String::new(),
            show_line_numbers: false,
            word_wrap: false,
            tab_size: 4,
            read_only: false,
            is_focused: false,
            is_hovered: false,
            cursor: CursorPosition::default(),
            cursor_blink_timer: 0.0,
            cursor_visible: true,
            selection_anchor: None,
            is_selecting: false,
            scroll_offset: Vec2::ZERO,
            visible_lines: 10,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            ime_preedit: String::new(),
            cached_cursor_x: Cell::new(0.0),
            cached_cursor_y: Cell::new(0.0),
            line_height: 20.0,
            gutter_width: 0.0,
        }
    }

    // === Builder Methods ===

    /// Sets the initial text content.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        let text = text.into();
        self.lines = text.lines().map(|s| s.to_string()).collect();
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self
    }

    /// Sets the placeholder text.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Sets the component ID.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Enables/disables line numbers.
    pub fn with_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    /// Enables/disables word wrap.
    pub fn with_word_wrap(mut self, wrap: bool) -> Self {
        self.word_wrap = wrap;
        self
    }

    /// Sets the tab size.
    pub fn with_tab_size(mut self, size: usize) -> Self {
        self.tab_size = size;
        self
    }

    /// Sets read-only mode.
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// Sets the interactive style.
    pub fn style(mut self, style: InteractiveStyle) -> Self {
        self.style = style;
        self
    }

    /// Sets the text style.
    pub fn text_style(mut self, style: TextStyle) -> Self {
        self.text_style = style;
        self
    }

    /// Sets layout properties.
    pub fn with_layout(mut self, layout: LayoutProps) -> Self {
        self.layout = layout;
        self
    }

    // === Public API ===

    /// Returns the full text content.
    pub fn get_text(&self) -> String {
        self.lines.join("\n")
    }

    /// Sets the text content.
    pub fn set_text(&mut self, text: impl Into<String>) {
        let text = text.into();
        self.lines = text.lines().map(|s| s.to_string()).collect();
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self.cursor = CursorPosition::default();
        self.clear_selection();
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Returns the current line count.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Returns the cursor position.
    pub fn cursor_position(&self) -> CursorPosition {
        self.cursor
    }

    // === Selection Helpers ===

    fn has_selection(&self) -> bool {
        self.selection_anchor.is_some() && self.selection_anchor != Some(self.cursor)
    }

    fn selection_range(&self) -> Option<(CursorPosition, CursorPosition)> {
        self.selection_anchor.map(|anchor| {
            if self.pos_less_than(anchor, self.cursor) {
                (anchor, self.cursor)
            } else {
                (self.cursor, anchor)
            }
        })
    }

    fn pos_less_than(&self, a: CursorPosition, b: CursorPosition) -> bool {
        a.line < b.line || (a.line == b.line && a.col < b.col)
    }

    fn selected_text(&self) -> String {
        if let Some((start, end)) = self.selection_range() {
            if start.line == end.line {
                // Single line selection
                let line = &self.lines[start.line];
                let start_byte = self.col_to_byte(start.line, start.col);
                let end_byte = self.col_to_byte(start.line, end.col);
                line[start_byte..end_byte].to_string()
            } else {
                // Multi-line selection
                let mut result = String::new();

                // First line (from start.col to end)
                let first_line = &self.lines[start.line];
                let start_byte = self.col_to_byte(start.line, start.col);
                result.push_str(&first_line[start_byte..]);
                result.push('\n');

                // Middle lines (full lines)
                for line_idx in (start.line + 1)..end.line {
                    result.push_str(&self.lines[line_idx]);
                    result.push('\n');
                }

                // Last line (from start to end.col)
                let last_line = &self.lines[end.line];
                let end_byte = self.col_to_byte(end.line, end.col);
                result.push_str(&last_line[..end_byte]);

                result
            }
        } else {
            String::new()
        }
    }

    fn delete_selection(&mut self) {
        if let Some((start, end)) = self.selection_range() {
            let deleted_text = self.selected_text();

            if start.line == end.line {
                // Single line deletion
                let start_byte = self.col_to_byte(start.line, start.col);
                let end_byte = self.col_to_byte(start.line, end.col);
                self.lines[start.line].replace_range(start_byte..end_byte, "");
            } else {
                // Multi-line deletion
                let start_byte = self.col_to_byte(start.line, start.col);
                let end_byte = self.col_to_byte(end.line, end.col);

                // Keep the start of first line and end of last line
                let new_line = format!(
                    "{}{}",
                    &self.lines[start.line][..start_byte],
                    &self.lines[end.line][end_byte..]
                );

                // Remove lines between start and end
                self.lines.drain(start.line..=end.line);
                self.lines.insert(start.line, new_line);
            }

            // Save undo action
            self.undo_stack.push(UndoAction::Delete {
                position: start,
                text: deleted_text,
            });
            self.redo_stack.clear();

            self.cursor = start;
            self.clear_selection();
        }
    }

    fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

    // === Cursor Helpers ===

    fn reset_cursor_blink(&mut self) {
        self.cursor_blink_timer = 0.0;
        self.cursor_visible = true;
    }

    fn clamp_cursor(&mut self) {
        self.cursor.line = self.cursor.line.min(self.lines.len().saturating_sub(1));
        let line_len = self.lines[self.cursor.line].chars().count();
        self.cursor.col = self.cursor.col.min(line_len);
    }

    fn col_to_byte(&self, line: usize, col: usize) -> usize {
        if line >= self.lines.len() {
            return 0;
        }
        self.lines[line]
            .char_indices()
            .nth(col)
            .map(|(i, _)| i)
            .unwrap_or(self.lines[line].len())
    }

    fn line_char_count(&self, line: usize) -> usize {
        if line < self.lines.len() {
            self.lines[line].chars().count()
        } else {
            0
        }
    }

    // === Text Editing ===

    fn insert_char(&mut self, ch: char) {
        if self.read_only {
            return;
        }

        if self.has_selection() {
            self.delete_selection();
        }

        let byte_pos = self.col_to_byte(self.cursor.line, self.cursor.col);
        self.lines[self.cursor.line].insert(byte_pos, ch);
        self.cursor.col += 1;

        self.undo_stack.push(UndoAction::Insert {
            position: CursorPosition::new(self.cursor.line, self.cursor.col - 1),
            text: ch.to_string(),
        });
        self.redo_stack.clear();
    }

    fn insert_text(&mut self, text: &str) {
        if self.read_only {
            return;
        }

        if self.has_selection() {
            self.delete_selection();
        }

        let start_pos = self.cursor;

        for ch in text.chars() {
            if ch == '\n' {
                self.insert_newline();
            } else if ch == '\t' {
                self.insert_tab();
            } else if !ch.is_control() {
                let byte_pos = self.col_to_byte(self.cursor.line, self.cursor.col);
                self.lines[self.cursor.line].insert(byte_pos, ch);
                self.cursor.col += 1;
            }
        }

        self.undo_stack.push(UndoAction::Insert {
            position: start_pos,
            text: text.to_string(),
        });
        self.redo_stack.clear();
    }

    fn insert_newline(&mut self) {
        if self.read_only {
            return;
        }

        let byte_pos = self.col_to_byte(self.cursor.line, self.cursor.col);
        let rest = self.lines[self.cursor.line][byte_pos..].to_string();
        self.lines[self.cursor.line].truncate(byte_pos);
        self.cursor.line += 1;
        self.lines.insert(self.cursor.line, rest);
        self.cursor.col = 0;
    }

    fn insert_tab(&mut self) {
        if self.read_only {
            return;
        }

        let spaces = " ".repeat(self.tab_size);
        let byte_pos = self.col_to_byte(self.cursor.line, self.cursor.col);
        self.lines[self.cursor.line].insert_str(byte_pos, &spaces);
        self.cursor.col += self.tab_size;
    }

    fn delete_char_before(&mut self) {
        if self.read_only {
            return;
        }

        if self.has_selection() {
            self.delete_selection();
            return;
        }

        if self.cursor.col > 0 {
            // Delete character in current line
            let char_indices: Vec<_> = self.lines[self.cursor.line].char_indices().collect();
            let idx = self.cursor.col - 1;
            if idx < char_indices.len() {
                let (byte_pos, ch) = char_indices[idx];
                let deleted = ch.to_string();
                self.lines[self.cursor.line].replace_range(byte_pos..byte_pos + ch.len_utf8(), "");
                self.cursor.col -= 1;

                self.undo_stack.push(UndoAction::Delete {
                    position: self.cursor,
                    text: deleted,
                });
                self.redo_stack.clear();
            }
        } else if self.cursor.line > 0 {
            // Merge with previous line
            let current_line = self.lines.remove(self.cursor.line);
            self.cursor.line -= 1;
            self.cursor.col = self.line_char_count(self.cursor.line);
            self.lines[self.cursor.line].push_str(&current_line);

            self.undo_stack.push(UndoAction::Delete {
                position: self.cursor,
                text: "\n".to_string(),
            });
            self.redo_stack.clear();
        }
    }

    fn delete_char_after(&mut self) {
        if self.read_only {
            return;
        }

        if self.has_selection() {
            self.delete_selection();
            return;
        }

        let line_len = self.line_char_count(self.cursor.line);

        if self.cursor.col < line_len {
            // Delete character in current line
            let char_indices: Vec<_> = self.lines[self.cursor.line].char_indices().collect();
            if self.cursor.col < char_indices.len() {
                let (byte_pos, ch) = char_indices[self.cursor.col];
                let deleted = ch.to_string();
                self.lines[self.cursor.line].replace_range(byte_pos..byte_pos + ch.len_utf8(), "");

                self.undo_stack.push(UndoAction::Delete {
                    position: self.cursor,
                    text: deleted,
                });
                self.redo_stack.clear();
            }
        } else if self.cursor.line < self.lines.len() - 1 {
            // Merge with next line
            let next_line = self.lines.remove(self.cursor.line + 1);
            self.lines[self.cursor.line].push_str(&next_line);

            self.undo_stack.push(UndoAction::Delete {
                position: self.cursor,
                text: "\n".to_string(),
            });
            self.redo_stack.clear();
        }
    }

    // === Navigation ===

    fn move_cursor_left(&mut self, extend_selection: bool) {
        if extend_selection && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        } else if !extend_selection {
            if self.has_selection() {
                if let Some((start, _)) = self.selection_range() {
                    self.cursor = start;
                }
                self.clear_selection();
                return;
            }
        }

        if self.cursor.col > 0 {
            self.cursor.col -= 1;
        } else if self.cursor.line > 0 {
            self.cursor.line -= 1;
            self.cursor.col = self.line_char_count(self.cursor.line);
        }

        if !extend_selection {
            self.clear_selection();
        }
    }

    fn move_cursor_right(&mut self, extend_selection: bool) {
        if extend_selection && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        } else if !extend_selection {
            if self.has_selection() {
                if let Some((_, end)) = self.selection_range() {
                    self.cursor = end;
                }
                self.clear_selection();
                return;
            }
        }

        let line_len = self.line_char_count(self.cursor.line);

        if self.cursor.col < line_len {
            self.cursor.col += 1;
        } else if self.cursor.line < self.lines.len() - 1 {
            self.cursor.line += 1;
            self.cursor.col = 0;
        }

        if !extend_selection {
            self.clear_selection();
        }
    }

    fn move_cursor_up(&mut self, extend_selection: bool) {
        if extend_selection && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        }

        if self.cursor.line > 0 {
            self.cursor.line -= 1;
            let line_len = self.line_char_count(self.cursor.line);
            self.cursor.col = self.cursor.col.min(line_len);
        } else {
            self.cursor.col = 0;
        }

        if !extend_selection {
            self.clear_selection();
        }
    }

    fn move_cursor_down(&mut self, extend_selection: bool) {
        if extend_selection && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        }

        if self.cursor.line < self.lines.len() - 1 {
            self.cursor.line += 1;
            let line_len = self.line_char_count(self.cursor.line);
            self.cursor.col = self.cursor.col.min(line_len);
        } else {
            self.cursor.col = self.line_char_count(self.cursor.line);
        }

        if !extend_selection {
            self.clear_selection();
        }
    }

    fn move_cursor_home(&mut self, extend_selection: bool) {
        if extend_selection && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        }

        self.cursor.col = 0;

        if !extend_selection {
            self.clear_selection();
        }
    }

    fn move_cursor_end(&mut self, extend_selection: bool) {
        if extend_selection && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        }

        self.cursor.col = self.line_char_count(self.cursor.line);

        if !extend_selection {
            self.clear_selection();
        }
    }

    fn page_up(&mut self, extend_selection: bool) {
        if extend_selection && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        }

        let page_size = self.visible_lines.saturating_sub(1);
        self.cursor.line = self.cursor.line.saturating_sub(page_size);
        let line_len = self.line_char_count(self.cursor.line);
        self.cursor.col = self.cursor.col.min(line_len);

        if !extend_selection {
            self.clear_selection();
        }
    }

    fn page_down(&mut self, extend_selection: bool) {
        if extend_selection && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        }

        let page_size = self.visible_lines.saturating_sub(1);
        self.cursor.line = (self.cursor.line + page_size).min(self.lines.len() - 1);
        let line_len = self.line_char_count(self.cursor.line);
        self.cursor.col = self.cursor.col.min(line_len);

        if !extend_selection {
            self.clear_selection();
        }
    }

    fn select_all(&mut self) {
        self.selection_anchor = Some(CursorPosition::new(0, 0));
        let last_line = self.lines.len() - 1;
        self.cursor = CursorPosition::new(last_line, self.line_char_count(last_line));
    }

    // === Scrolling ===

    fn ensure_cursor_visible(&mut self) {
        let content_height = self.bounds.height - self.layout.padding * 2.0;
        self.visible_lines = (content_height / self.line_height) as usize;

        // Vertical scrolling
        let cursor_y = self.cursor.line as f32 * self.line_height;

        if cursor_y < self.scroll_offset.y {
            self.scroll_offset.y = cursor_y;
        } else if cursor_y + self.line_height > self.scroll_offset.y + content_height {
            self.scroll_offset.y = cursor_y + self.line_height - content_height;
        }

        // Horizontal scrolling (future)
        // TODO: implement horizontal scroll for long lines
    }

    // === Text Measurement ===

    fn measure_text_to_col(&self, renderer: &mut Renderer, line: usize, col: usize) -> f32 {
        if line >= self.lines.len() || col == 0 {
            return 0.0;
        }
        let prefix: String = self.lines[line].chars().take(col).collect();
        renderer.measure_text(&prefix, self.text_style.font_size)
    }

    // === IME ===

    fn update_ime_position(&self, ctx: &OxidXContext) {
        if self.is_focused {
            ctx.set_ime_position(Rect::new(
                self.cached_cursor_x.get(),
                self.cached_cursor_y.get(),
                2.0,
                self.line_height,
            ));
        }
    }

    // === Undo/Redo ===

    fn undo(&mut self) {
        if self.read_only {
            return;
        }

        if let Some(action) = self.undo_stack.pop() {
            match action {
                UndoAction::Insert { position, text } => {
                    // Undo insert = delete the inserted text
                    self.cursor = position;
                    let char_count = text.chars().count();
                    for _ in 0..char_count {
                        // Move cursor to end of inserted text first
                        let line_len = self.line_char_count(self.cursor.line);
                        if self.cursor.col < line_len {
                            self.cursor.col += 1;
                        } else if self.cursor.line < self.lines.len() - 1 {
                            self.cursor.line += 1;
                            self.cursor.col = 0;
                        }
                    }
                    // Now delete backwards
                    for ch in text.chars().rev() {
                        if ch == '\n' {
                            // Merge lines
                            if self.cursor.line > 0 {
                                let current_line = self.lines.remove(self.cursor.line);
                                self.cursor.line -= 1;
                                self.cursor.col = self.line_char_count(self.cursor.line);
                                self.lines[self.cursor.line].push_str(&current_line);
                            }
                        } else if self.cursor.col > 0 {
                            let byte_pos = self.col_to_byte(self.cursor.line, self.cursor.col - 1);
                            let end_byte = self.col_to_byte(self.cursor.line, self.cursor.col);
                            self.lines[self.cursor.line].replace_range(byte_pos..end_byte, "");
                            self.cursor.col -= 1;
                        }
                    }
                    self.redo_stack.push(UndoAction::Insert { position, text });
                }
                UndoAction::Delete { position, text } => {
                    // Undo delete = re-insert the deleted text
                    self.cursor = position;
                    for ch in text.chars() {
                        if ch == '\n' {
                            let byte_pos = self.col_to_byte(self.cursor.line, self.cursor.col);
                            let rest = self.lines[self.cursor.line][byte_pos..].to_string();
                            self.lines[self.cursor.line].truncate(byte_pos);
                            self.cursor.line += 1;
                            self.lines.insert(self.cursor.line, rest);
                            self.cursor.col = 0;
                        } else {
                            let byte_pos = self.col_to_byte(self.cursor.line, self.cursor.col);
                            self.lines[self.cursor.line].insert(byte_pos, ch);
                            self.cursor.col += 1;
                        }
                    }
                    self.redo_stack.push(UndoAction::Delete { position, text });
                }
                UndoAction::Replace {
                    position,
                    old_text,
                    new_text,
                } => {
                    // Undo replace = replace new_text with old_text
                    self.cursor = position;
                    // Delete new_text, insert old_text
                    // Simplified: just push to redo and swap
                    self.redo_stack.push(UndoAction::Replace {
                        position,
                        old_text: new_text.clone(),
                        new_text: old_text,
                    });
                }
            }
            self.clear_selection();
        }
    }

    fn redo(&mut self) {
        if self.read_only {
            return;
        }

        if let Some(action) = self.redo_stack.pop() {
            match action {
                UndoAction::Insert { position, text } => {
                    // Redo insert = insert the text again
                    self.cursor = position;
                    for ch in text.chars() {
                        if ch == '\n' {
                            let byte_pos = self.col_to_byte(self.cursor.line, self.cursor.col);
                            let rest = self.lines[self.cursor.line][byte_pos..].to_string();
                            self.lines[self.cursor.line].truncate(byte_pos);
                            self.cursor.line += 1;
                            self.lines.insert(self.cursor.line, rest);
                            self.cursor.col = 0;
                        } else {
                            let byte_pos = self.col_to_byte(self.cursor.line, self.cursor.col);
                            self.lines[self.cursor.line].insert(byte_pos, ch);
                            self.cursor.col += 1;
                        }
                    }
                    self.undo_stack.push(UndoAction::Insert { position, text });
                }
                UndoAction::Delete { position, text } => {
                    // Redo delete = delete the text again
                    self.cursor = position;
                    let char_count = text.chars().count();
                    for _ in 0..char_count {
                        let line_len = self.line_char_count(self.cursor.line);
                        if self.cursor.col < line_len {
                            let byte_pos = self.col_to_byte(self.cursor.line, self.cursor.col);
                            let end_byte = self.col_to_byte(self.cursor.line, self.cursor.col + 1);
                            self.lines[self.cursor.line].replace_range(byte_pos..end_byte, "");
                        } else if self.cursor.line < self.lines.len() - 1 {
                            // Merge with next line
                            let next_line = self.lines.remove(self.cursor.line + 1);
                            self.lines[self.cursor.line].push_str(&next_line);
                        }
                    }
                    self.undo_stack.push(UndoAction::Delete { position, text });
                }
                UndoAction::Replace {
                    position,
                    old_text,
                    new_text,
                } => {
                    self.cursor = position;
                    self.undo_stack.push(UndoAction::Replace {
                        position,
                        old_text: new_text,
                        new_text: old_text,
                    });
                }
            }
            self.clear_selection();
        }
    }
}

impl Default for TextArea {
    fn default() -> Self {
        Self::new()
    }
}

impl OxidXComponent for TextArea {
    fn update(&mut self, dt: f32) {
        // Cursor blinking
        if self.is_focused && !self.has_selection() {
            self.cursor_blink_timer += dt;
            if self.cursor_blink_timer >= 0.53 {
                self.cursor_visible = !self.cursor_visible;
                self.cursor_blink_timer = 0.0;
            }
        }
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        let margin = self.layout.margin;
        self.bounds = Rect::new(
            available.x + margin,
            available.y + margin,
            available.width - margin * 2.0,
            available.height - margin * 2.0,
        );

        // Calculate gutter width for line numbers
        if self.show_line_numbers {
            let digits = (self.lines.len() as f32).log10().ceil() as usize + 1;
            self.gutter_width = digits as f32 * self.text_style.font_size * 0.6 + 16.0;
        } else {
            self.gutter_width = 0.0;
        }

        // Calculate line height
        self.line_height = self.text_style.font_size * 1.4;

        // Calculate visible lines
        let content_height = self.bounds.height - self.layout.padding * 2.0;
        self.visible_lines = (content_height / self.line_height).max(1.0) as usize;

        Vec2::new(self.bounds.width, self.bounds.height)
    }

    fn render(&self, renderer: &mut Renderer) {
        let state = if self.is_focused {
            ComponentState::Pressed
        } else if self.is_hovered {
            ComponentState::Hover
        } else {
            ComponentState::Idle
        };

        let current_style = self.style.resolve(state);

        // 1. Draw background
        renderer.draw_style_rect(self.bounds, current_style);

        let padding = self.layout.padding;
        let content_x = self.bounds.x + padding + self.gutter_width;
        let content_y = self.bounds.y + padding;
        let content_width = self.bounds.width - padding * 2.0 - self.gutter_width;

        // 2. Draw line numbers
        if self.show_line_numbers {
            let gutter_x = self.bounds.x + padding;

            for (i, _) in self.lines.iter().enumerate() {
                let y = content_y + (i as f32 * self.line_height) - self.scroll_offset.y;

                if y < content_y - self.line_height || y > self.bounds.y + self.bounds.height {
                    continue; // Skip invisible lines
                }

                let line_num = format!("{}", i + 1);
                let num_width = renderer.measure_text(&line_num, self.text_style.font_size);

                renderer.draw_text(
                    &line_num,
                    Vec2::new(gutter_x + self.gutter_width - num_width - 8.0, y),
                    TextStyle {
                        font_size: self.text_style.font_size,
                        color: self.line_number_color,
                        ..self.text_style.clone()
                    },
                );
            }

            // Draw gutter separator
            renderer.fill_rect(
                Rect::new(
                    self.bounds.x + padding + self.gutter_width - 4.0,
                    self.bounds.y + padding,
                    1.0,
                    self.bounds.height - padding * 2.0,
                ),
                Color::new(0.3, 0.3, 0.35, 1.0),
            );
        }

        // 3. Draw selection
        if let Some((start, end)) = self.selection_range() {
            for line_idx in start.line..=end.line {
                if line_idx >= self.lines.len() {
                    break;
                }

                let y = content_y + (line_idx as f32 * self.line_height) - self.scroll_offset.y;

                if y < content_y - self.line_height || y > self.bounds.y + self.bounds.height {
                    continue;
                }

                let sel_start_col = if line_idx == start.line { start.col } else { 0 };
                let sel_end_col = if line_idx == end.line {
                    end.col
                } else {
                    self.line_char_count(line_idx)
                };

                let start_x = self.measure_text_to_col(renderer, line_idx, sel_start_col);
                let end_x = self.measure_text_to_col(renderer, line_idx, sel_end_col);

                renderer.fill_rect(
                    Rect::new(
                        content_x + start_x,
                        y,
                        (end_x - start_x).max(4.0), // Min width for empty line selection
                        self.line_height,
                    ),
                    Color::new(0.2, 0.5, 0.9, 0.5),
                );
            }
        }

        // 4. Draw text lines
        let is_empty = self.lines.len() == 1 && self.lines[0].is_empty();

        for (i, line) in self.lines.iter().enumerate() {
            let y = content_y + (i as f32 * self.line_height) - self.scroll_offset.y;

            if y < content_y - self.line_height || y > self.bounds.y + self.bounds.height {
                continue;
            }

            let display_text = if is_empty && i == 0 && !self.placeholder.is_empty() {
                &self.placeholder
            } else {
                line
            };

            let mut color = self.text_style.color;
            if is_empty && i == 0 && !self.placeholder.is_empty() {
                color.a *= 0.5;
            }

            renderer.draw_text(
                display_text,
                Vec2::new(content_x, y),
                TextStyle {
                    font_size: self.text_style.font_size,
                    color,
                    ..self.text_style.clone()
                },
            );
        }

        // 5. Draw cursor
        if self.is_focused {
            let cursor_y =
                content_y + (self.cursor.line as f32 * self.line_height) - self.scroll_offset.y;
            let cursor_x =
                content_x + self.measure_text_to_col(renderer, self.cursor.line, self.cursor.col);

            // Draw IME preedit
            if !self.ime_preedit.is_empty() {
                let preedit_width =
                    renderer.measure_text(&self.ime_preedit, self.text_style.font_size);

                renderer.draw_text(
                    &self.ime_preedit,
                    Vec2::new(cursor_x, cursor_y),
                    TextStyle {
                        font_size: self.text_style.font_size,
                        color: Color::new(1.0, 1.0, 0.5, 1.0),
                        ..self.text_style.clone()
                    },
                );

                // Underline
                renderer.fill_rect(
                    Rect::new(
                        cursor_x,
                        cursor_y + self.line_height - 2.0,
                        preedit_width,
                        1.0,
                    ),
                    Color::new(1.0, 1.0, 0.5, 1.0),
                );
            }

            // Draw cursor (blinking)
            if self.cursor_visible && !self.has_selection() {
                let final_cursor_x = if self.ime_preedit.is_empty() {
                    cursor_x
                } else {
                    cursor_x + renderer.measure_text(&self.ime_preedit, self.text_style.font_size)
                };

                renderer.fill_rect(
                    Rect::new(final_cursor_x, cursor_y, 2.0, self.line_height),
                    self.text_style.color,
                );

                self.cached_cursor_x.set(final_cursor_x);
                self.cached_cursor_y.set(cursor_y);
            }
        }
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        match event {
            OxidXEvent::MouseDown { position, .. }
            | OxidXEvent::MouseUp { position, .. }
            | OxidXEvent::Click { position, .. } => {
                if !self.bounds.contains(*position) {
                    return false;
                }
            }
            _ => {}
        }

        match event {
            OxidXEvent::MouseEnter => {
                self.is_hovered = true;
                ctx.set_cursor_icon(CursorIcon::Text);
                true
            }
            OxidXEvent::MouseLeave => {
                self.is_hovered = false;
                ctx.set_cursor_icon(CursorIcon::Default);
                true
            }
            OxidXEvent::FocusGained => {
                self.is_focused = true;
                self.reset_cursor_blink();
                true
            }
            OxidXEvent::FocusLost => {
                self.is_focused = false;
                self.is_selecting = false;
                self.ime_preedit.clear();
                true
            }
            OxidXEvent::MouseDown {
                position,
                modifiers,
                ..
            } => {
                if !self.id.is_empty() {
                    ctx.request_focus(&self.id);
                }
                self.is_focused = true;
                self.is_selecting = true;

                // Calculate cursor position from click
                let content_x = self.bounds.x + self.layout.padding + self.gutter_width;
                let content_y = self.bounds.y + self.layout.padding;

                let relative_y = position.y - content_y + self.scroll_offset.y;
                let line = ((relative_y / self.line_height) as usize).min(self.lines.len() - 1);

                let relative_x = (position.x - content_x).max(0.0);
                let col = if self.lines[line].is_empty() {
                    0
                } else {
                    // Approximate
                    ((relative_x / 8.0) as usize).min(self.line_char_count(line))
                };

                if modifiers.shift && self.selection_anchor.is_some() {
                    self.cursor = CursorPosition::new(line, col);
                } else {
                    self.cursor = CursorPosition::new(line, col);
                    self.selection_anchor = Some(self.cursor);
                }

                self.reset_cursor_blink();
                self.update_ime_position(ctx);
                true
            }
            OxidXEvent::MouseMove { position, .. } => {
                if self.is_selecting {
                    let content_x = self.bounds.x + self.layout.padding + self.gutter_width;
                    let content_y = self.bounds.y + self.layout.padding;

                    let relative_y = position.y - content_y + self.scroll_offset.y;
                    let line = ((relative_y / self.line_height).max(0.0) as usize)
                        .min(self.lines.len() - 1);

                    let relative_x = (position.x - content_x).max(0.0);
                    let col = ((relative_x / 8.0) as usize).min(self.line_char_count(line));

                    self.cursor = CursorPosition::new(line, col);
                    self.reset_cursor_blink();
                    true
                } else {
                    false
                }
            }
            OxidXEvent::MouseUp { .. } => {
                self.is_selecting = false;
                if self.selection_anchor == Some(self.cursor) {
                    self.clear_selection();
                }
                true
            }
            OxidXEvent::Click { .. } => true,
            OxidXEvent::KeyDown { .. } | OxidXEvent::CharInput { .. } => {
                if self.is_focused {
                    self.on_keyboard_input(event, ctx);
                    true
                } else {
                    false
                }
            }
            OxidXEvent::ImePreedit { text, .. } => {
                self.ime_preedit = text.clone();
                self.update_ime_position(ctx);
                true
            }
            OxidXEvent::ImeCommit(text) => {
                if self.has_selection() {
                    self.delete_selection();
                }
                self.insert_text(text);
                self.ime_preedit.clear();
                self.reset_cursor_blink();
                self.ensure_cursor_visible();
                self.update_ime_position(ctx);
                true
            }
            _ => false,
        }
    }

    fn on_keyboard_input(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) {
        if !ctx.is_focused(&self.id) {
            return;
        }

        match event {
            OxidXEvent::CharInput {
                character,
                modifiers,
            } => {
                // Skip character input when primary modifier (Cmd/Ctrl) is held - these are shortcuts
                if modifiers.is_primary() {
                    return;
                }
                if !character.is_control() {
                    if self.has_selection() {
                        self.delete_selection();
                    }
                    self.insert_char(*character);
                    self.reset_cursor_blink();
                    self.ensure_cursor_visible();
                }
            }
            OxidXEvent::KeyDown { key, modifiers } => {
                let shift = modifiers.shift;

                // Navigation
                if *key == KeyCode::LEFT {
                    self.move_cursor_left(shift);
                    self.reset_cursor_blink();
                    self.ensure_cursor_visible();
                    return;
                }
                if *key == KeyCode::RIGHT {
                    self.move_cursor_right(shift);
                    self.reset_cursor_blink();
                    self.ensure_cursor_visible();
                    return;
                }
                if *key == KeyCode::UP {
                    self.move_cursor_up(shift);
                    self.reset_cursor_blink();
                    self.ensure_cursor_visible();
                    return;
                }
                if *key == KeyCode::DOWN {
                    self.move_cursor_down(shift);
                    self.reset_cursor_blink();
                    self.ensure_cursor_visible();
                    return;
                }
                if *key == KeyCode::HOME {
                    self.move_cursor_home(shift);
                    self.reset_cursor_blink();
                    self.ensure_cursor_visible();
                    return;
                }
                if *key == KeyCode::END {
                    self.move_cursor_end(shift);
                    self.reset_cursor_blink();
                    self.ensure_cursor_visible();
                    return;
                }

                // Page Up/Down - need to add these KeyCodes
                // if *key == KeyCode::PAGE_UP { ... }
                // if *key == KeyCode::PAGE_DOWN { ... }

                // Editing
                if *key == KeyCode::ENTER && !self.read_only {
                    if self.has_selection() {
                        self.delete_selection();
                    }
                    self.insert_newline();
                    self.reset_cursor_blink();
                    self.ensure_cursor_visible();
                    return;
                }
                if *key == KeyCode::TAB && !self.read_only {
                    if self.has_selection() {
                        self.delete_selection();
                    }
                    self.insert_tab();
                    self.reset_cursor_blink();
                    self.ensure_cursor_visible();
                    return;
                }
                if *key == KeyCode::BACKSPACE && !self.read_only {
                    self.delete_char_before();
                    self.reset_cursor_blink();
                    self.ensure_cursor_visible();
                    return;
                }
                if *key == KeyCode::DELETE && !self.read_only {
                    self.delete_char_after();
                    self.reset_cursor_blink();
                    self.ensure_cursor_visible();
                    return;
                }

                // Shortcuts
                // Undo (Cmd+Z / Ctrl+Z)
                if modifiers.is_primary() && *key == KeyCode::KEY_Z && !modifiers.shift {
                    self.undo();
                    self.reset_cursor_blink();
                    self.ensure_cursor_visible();
                    return;
                }
                // Redo (Cmd+Shift+Z / Ctrl+Shift+Z or Ctrl+Y)
                if modifiers.is_primary()
                    && (*key == KeyCode::KEY_Z && modifiers.shift || *key == KeyCode::KEY_Y)
                {
                    self.redo();
                    self.reset_cursor_blink();
                    self.ensure_cursor_visible();
                    return;
                }
                // Select All (Cmd+A / Ctrl+A)
                if modifiers.is_primary() && *key == KeyCode::KEY_A {
                    self.select_all();
                    return;
                }
                // Copy (Cmd+C / Ctrl+C)
                if modifiers.is_primary() && *key == KeyCode::KEY_C {
                    if self.has_selection() {
                        ctx.copy_to_clipboard(&self.selected_text());
                    }
                    return;
                }
                // Cut (Cmd+X / Ctrl+X)
                if modifiers.is_primary() && *key == KeyCode::KEY_X && !self.read_only {
                    if self.has_selection() {
                        ctx.copy_to_clipboard(&self.selected_text());
                        self.delete_selection();
                        self.reset_cursor_blink();
                        self.ensure_cursor_visible();
                    }
                    return;
                }
                // Paste (Cmd+V / Ctrl+V)
                if modifiers.is_primary() && *key == KeyCode::KEY_V && !self.read_only {
                    if let Some(text) = ctx.paste_from_clipboard() {
                        if self.has_selection() {
                            self.delete_selection();
                        }
                        self.insert_text(&text);
                        self.reset_cursor_blink();
                        self.ensure_cursor_visible();
                    }
                    return;
                }

                // Escape
                if *key == KeyCode::ESCAPE {
                    self.clear_selection();
                    self.ime_preedit.clear();
                    return;
                }
            }
            _ => {}
        }
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.bounds.x = x + self.layout.margin;
        self.bounds.y = y + self.layout.margin;
    }

    fn set_size(&mut self, width: f32, height: f32) {
        self.bounds.width = width - self.layout.margin * 2.0;
        self.bounds.height = height - self.layout.margin * 2.0;
    }

    fn is_focusable(&self) -> bool {
        true
    }
}
