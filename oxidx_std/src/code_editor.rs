//! CodeEditor Component
//!
//! A full-featured code editor with:
//! - Syntax highlighting (Rust, extensible)
//! - Line numbers with gutter
//! - Minimap (VS Code style)
//! - Multiple line support with scrolling
//! - Cursor navigation and text selection
//! - Word wrap (optional)
//! - Clipboard support
//! - IME support
//! - Undo/Redo

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

// Import SyntaxDefinition for dynamic language support
use oxidx_core::syntax::{SyntaxDefinition, SyntaxError};

/// Syntax theme for code highlighting.
#[derive(Debug, Clone)]
pub struct SyntaxTheme {
    /// Keywords (fn, let, pub, struct, impl, use, etc.)
    pub keyword: Color,
    /// Strings (text inside quotes)
    pub string: Color,
    /// Comments (after //)
    pub comment: Color,
    /// Numbers (digits)
    pub number: Color,
    /// Normal text
    pub normal: Color,
    /// Types (uppercase identifiers like String, Vec, Option)
    pub type_name: Color,
    /// Macros (ending with !)
    pub macro_call: Color,
    /// Function names
    pub function: Color,
    /// Whether syntax highlighting is enabled
    pub enabled: bool,
}

impl Default for SyntaxTheme {
    fn default() -> Self {
        Self::dark_rust()
    }
}

impl SyntaxTheme {
    /// VS Code Dark / Monokai-style theme for Rust
    /// Note: syntax highlighting is disabled by default. Use with_syntax_highlighting(true) to enable.
    pub fn dark_rust() -> Self {
        Self {
            keyword: Color::new(0.78, 0.47, 0.82, 1.0),   // Purple
            string: Color::new(0.8, 0.68, 0.47, 1.0),     // Orange/Yellow
            comment: Color::new(0.45, 0.55, 0.45, 1.0),   // Green-gray
            number: Color::new(0.71, 0.84, 0.67, 1.0),    // Light green
            normal: Color::new(0.85, 0.85, 0.9, 1.0),     // Light gray
            type_name: Color::new(0.4, 0.75, 0.85, 1.0),  // Cyan
            macro_call: Color::new(0.4, 0.75, 0.85, 1.0), // Cyan
            function: Color::new(0.88, 0.88, 0.62, 1.0),  // Yellow
            enabled: false,                               // Disabled by default for performance
        }
    }

    /// No syntax highlighting
    pub fn none() -> Self {
        Self {
            keyword: Color::WHITE,
            string: Color::WHITE,
            comment: Color::WHITE,
            number: Color::WHITE,
            normal: Color::WHITE,
            type_name: Color::WHITE,
            macro_call: Color::WHITE,
            function: Color::WHITE,
            enabled: false,
        }
    }
}

/// A text span with its color for syntax highlighting
#[derive(Debug, Clone)]
pub struct TextSpan {
    pub text: String,
    pub color: Color,
}

/// Rust keywords for syntax highlighting
const RUST_KEYWORDS: &[&str] = &[
    "fn", "let", "mut", "pub", "struct", "impl", "use", "mod", "crate", "super", "self", "Self",
    "const", "static", "enum", "trait", "type", "where", "for", "while", "loop", "if", "else",
    "match", "return", "break", "continue", "as", "in", "ref", "move", "async", "await", "dyn",
    "unsafe", "extern", "true", "false", "Some", "None", "Ok", "Err",
];

/// JS keywords for syntax highlighting
const JS_KEYWORDS: &[&str] = &[
    "function",
    "const",
    "let",
    "var",
    "if",
    "else",
    "for",
    "while",
    "do",
    "switch",
    "case",
    "break",
    "continue",
    "return",
    "try",
    "catch",
    "finally",
    "throw",
    "new",
    "this",
    "super",
    "class",
    "extends",
    "import",
    "export",
    "default",
    "from",
    "async",
    "await",
    "typeof",
    "instanceof",
    "in",
    "of",
    "void",
    "delete",
    "true",
    "false",
    "null",
    "undefined",
    "NaN",
];

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

/// A full-featured code editor.
///
/// Includes:
/// - Syntax highlighting
/// - Line numbering
/// - Cursor blinking and movement
/// - Selection ranges
/// - Undo/Redo stack
/// - Basic keyboard shortcuts
pub struct CodeEditor {
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
    language: String,

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
    /// Show horizontal scrollbar
    show_scrollbar_x: bool,
    /// Show vertical scrollbar
    show_scrollbar_y: bool,
    /// Is dragging the scrollbar thumb
    is_dragging_scrollbar: bool,

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

    // === Syntax Highlighting ===
    syntax_theme: SyntaxTheme,
    /// Cached tokenized lines (invalidated on text change)
    cached_tokens: Vec<Vec<TextSpan>>,
    /// Whether cache needs to be rebuilt
    tokens_dirty: bool,

    // === Minimap (VS Code style) ===
    /// Show minimap on right side
    show_minimap: bool,
    /// Width of minimap in pixels
    minimap_width: f32,
    /// Is dragging the minimap viewport
    is_dragging_minimap: bool,

    // === Dynamic Syntax Definition ===
    /// Optional loaded syntax definition for dynamic language support
    syntax_definition: Option<SyntaxDefinition>,
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
    #[allow(dead_code)]
    Replace {
        position: CursorPosition,
        old_text: String,
        new_text: String,
    },
}

impl CodeEditor {
    /// Creates a new empty CodeEditor with default dark theme settings.
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
            show_line_numbers: true,
            word_wrap: false,
            tab_size: 4,
            read_only: false,
            language: "rust".to_string(),
            is_focused: false,
            is_hovered: false,
            cursor: CursorPosition::default(),
            cursor_blink_timer: 0.0,
            cursor_visible: true,
            selection_anchor: None,
            is_selecting: false,
            scroll_offset: Vec2::ZERO,
            visible_lines: 10,
            show_scrollbar_x: false,
            show_scrollbar_y: true,
            is_dragging_scrollbar: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            ime_preedit: String::new(),
            cached_cursor_x: Cell::new(0.0),
            cached_cursor_y: Cell::new(0.0),
            line_height: 20.0,
            gutter_width: 0.0,
            syntax_theme: SyntaxTheme::dark_rust(),
            cached_tokens: Vec::new(),
            tokens_dirty: true,
            show_minimap: false,
            minimap_width: 80.0,
            is_dragging_minimap: false,
            // Dynamic syntax
            syntax_definition: None,
        }
    }

    // === Builder Methods ===

    /// Sets the initial text content.
    ///
    /// Clears undo history and resets cursor position.
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

    /// Sets the syntax theme for code highlighting.
    pub fn with_syntax_theme(mut self, theme: SyntaxTheme) -> Self {
        self.syntax_theme = theme;
        self
    }

    /// Enables syntax highlighting with the default Rust theme.
    pub fn with_syntax_highlighting(mut self, enabled: bool) -> Self {
        self.syntax_theme.enabled = enabled;
        self
    }

    /// Sets the syntax language (e.g. "rust", "js").
    pub fn syntax(mut self, lang: &str) -> Self {
        self.language = lang.to_string();
        self.syntax_theme.enabled = true;
        self.tokens_dirty = true;
        self
    }

    /// Loads a syntax definition from a JSON file.
    ///
    /// When loaded, the tokenizer will use the keywords and types from
    /// the definition instead of the hardcoded RUST_KEYWORDS.
    pub fn load_syntax_from_file(mut self, path: &str) -> Result<Self, SyntaxError> {
        let definition = SyntaxDefinition::from_file(path)?;
        self.syntax_definition = Some(definition);
        self.syntax_theme.enabled = true; // Auto-enable when loading syntax
        self.tokens_dirty = true;
        Ok(self)
    }

    /// Sets a syntax definition directly (for embedded definitions).
    pub fn with_syntax_definition(mut self, definition: SyntaxDefinition) -> Self {
        self.syntax_definition = Some(definition);
        self.syntax_theme.enabled = true;
        self.tokens_dirty = true;
        self
    }

    /// Enables/disables word wrap.
    pub fn with_word_wrap(mut self, wrap: bool) -> Self {
        self.word_wrap = wrap;
        self
    }

    /// Shows/hides the horizontal scrollbar.
    pub fn with_scrollbar_x(mut self, show: bool) -> Self {
        self.show_scrollbar_x = show;
        self
    }

    /// Shows/hides the vertical scrollbar.
    pub fn with_scrollbar_y(mut self, show: bool) -> Self {
        self.show_scrollbar_y = show;
        self
    }

    /// Enables/disables the minimap (VS Code style code preview).
    pub fn with_minimap(mut self, show: bool) -> Self {
        self.show_minimap = show;
        self
    }

    /// Sets the minimap width (default: 80 pixels).
    pub fn with_minimap_width(mut self, width: f32) -> Self {
        self.minimap_width = width;
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

    fn _clamp_cursor(&mut self) {
        self.cursor.line = self.cursor.line.min(self.lines.len().saturating_sub(1));
        let line_len = self.lines[self.cursor.line].chars().count();
        self.cursor.col = self.cursor.col.min(line_len);
    }

    /// Tokenizes a line of text into colored spans for syntax highlighting.
    fn tokenize_line(&self, line: &str) -> Vec<TextSpan> {
        if !self.syntax_theme.enabled || line.is_empty() {
            return vec![TextSpan {
                text: line.to_string(),
                color: self.syntax_theme.normal,
            }];
        }

        let mut spans = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            // Check for line comment
            if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '/' {
                let rest: String = chars[i..].iter().collect();
                spans.push(TextSpan {
                    text: rest,
                    color: self.syntax_theme.comment,
                });
                break;
            }

            // Check for string literal
            if chars[i] == '"' {
                let start = i;
                i += 1;
                while i < chars.len() && chars[i] != '"' {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        i += 1; // Skip escaped char
                    }
                    i += 1;
                }
                if i < chars.len() {
                    i += 1; // Include closing quote
                }
                let text: String = chars[start..i].iter().collect();
                spans.push(TextSpan {
                    text,
                    color: self.syntax_theme.string,
                });
                continue;
            }

            // Check for char literal
            if chars[i] == '\'' && i + 2 < chars.len() {
                let start = i;
                i += 1;
                if chars[i] == '\\' {
                    i += 2;
                } else {
                    i += 1;
                }
                if i < chars.len() && chars[i] == '\'' {
                    i += 1;
                    let text: String = chars[start..i].iter().collect();
                    spans.push(TextSpan {
                        text,
                        color: self.syntax_theme.string,
                    });
                    continue;
                }
                i = start + 1; // Not a char literal, backtrack
            }

            // Check for number
            if chars[i].is_ascii_digit() {
                let start = i;
                while i < chars.len()
                    && (chars[i].is_ascii_alphanumeric() || chars[i] == '_' || chars[i] == '.')
                {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                spans.push(TextSpan {
                    text,
                    color: self.syntax_theme.number,
                });
                continue;
            }

            // Check for identifier/keyword
            if chars[i].is_alphabetic() || chars[i] == '_' {
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();

                // Check for macro
                if i < chars.len() && chars[i] == '!' {
                    i += 1;
                    let text = format!("{}!", word);
                    spans.push(TextSpan {
                        text,
                        color: self.syntax_theme.macro_call,
                    });
                    continue;
                }

                // Check if keyword using dynamic definition or fallback
                let is_keyword = if let Some(ref def) = self.syntax_definition {
                    def.is_keyword(&word)
                } else {
                    match self.language.as_str() {
                        "js" | "javascript" => JS_KEYWORDS.contains(&word.as_str()),
                        _ => RUST_KEYWORDS.contains(&word.as_str()),
                    }
                };

                // Check if type using dynamic definition
                let is_type = if let Some(ref def) = self.syntax_definition {
                    def.is_type(&word)
                } else {
                    word.chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false)
                };

                let color = if is_keyword {
                    self.syntax_theme.keyword
                } else if is_type {
                    self.syntax_theme.type_name
                } else {
                    self.syntax_theme.normal
                };

                spans.push(TextSpan { text: word, color });
                continue;
            }

            // Whitespace and other characters
            let start = i;
            while i < chars.len()
                && !chars[i].is_alphanumeric()
                && chars[i] != '_'
                && chars[i] != '"'
                && chars[i] != '\''
                && chars[i] != '/'
            {
                i += 1;
            }
            if i > start {
                let text: String = chars[start..i].iter().collect();
                spans.push(TextSpan {
                    text,
                    color: self.syntax_theme.normal,
                });
            } else {
                // Single char fallback
                spans.push(TextSpan {
                    text: chars[i].to_string(),
                    color: self.syntax_theme.normal,
                });
                i += 1;
            }
        }

        spans
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
        self.tokens_dirty = true; // Invalidate syntax cache
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
        self.tokens_dirty = true; // Invalidate syntax cache
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
        self.tokens_dirty = true; // Invalidate cache when lines change
    }

    fn insert_tab(&mut self) {
        if self.read_only {
            return;
        }

        let spaces = " ".repeat(self.tab_size);
        let byte_pos = self.col_to_byte(self.cursor.line, self.cursor.col);
        self.lines[self.cursor.line].insert_str(byte_pos, &spaces);
        self.cursor.col += self.tab_size;
        self.tokens_dirty = true; // Invalidate cache
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
                self.tokens_dirty = true; // Invalidate cache
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
            self.tokens_dirty = true; // Invalidate cache when lines merge
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
                self.tokens_dirty = true; // Invalidate cache
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
            self.tokens_dirty = true; // Invalidate cache when lines merge
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

    fn _page_up(&mut self, extend_selection: bool) {
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

    fn _page_down(&mut self, extend_selection: bool) {
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

impl Default for CodeEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl OxidXComponent for CodeEditor {
    fn update(&mut self, dt: f32) {
        // Cursor blinking
        if self.is_focused && !self.has_selection() {
            self.cursor_blink_timer += dt;
            if self.cursor_blink_timer >= 0.53 {
                self.cursor_visible = !self.cursor_visible;
                self.cursor_blink_timer = 0.0;
            }
        }

        // Rebuild syntax highlighting cache if needed
        if self.tokens_dirty && self.syntax_theme.enabled {
            self.cached_tokens.clear();
            for line in &self.lines {
                self.cached_tokens.push(self.tokenize_line(line));
            }
            self.tokens_dirty = false;
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
        //let content_width = self.bounds.width - padding * 2.0 - self.gutter_width;

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

        // 4. Draw text lines with syntax highlighting
        let is_empty = self.lines.len() == 1 && self.lines[0].is_empty();
        let visible_top = content_y;
        let visible_bottom = self.bounds.y + self.bounds.height - padding;

        for (i, line) in self.lines.iter().enumerate() {
            let y = content_y + (i as f32 * self.line_height) - self.scroll_offset.y;

            // Skip lines outside visible area (strict clipping)
            if y + self.line_height < visible_top || y > visible_bottom {
                continue;
            }

            // Handle placeholder
            if is_empty && i == 0 && !self.placeholder.is_empty() {
                let mut color = self.text_style.color;
                color.a *= 0.5;
                renderer.draw_text(
                    &self.placeholder,
                    Vec2::new(content_x, y),
                    TextStyle {
                        font_size: self.text_style.font_size,
                        color,
                        ..self.text_style.clone()
                    },
                );
                continue;
            }

            // Use cached tokens for syntax highlighting (much faster!)
            if self.syntax_theme.enabled && !line.is_empty() && i < self.cached_tokens.len() {
                let spans = &self.cached_tokens[i];
                let mut x = content_x;

                for span in spans {
                    renderer.draw_text(
                        &span.text,
                        Vec2::new(x, y),
                        TextStyle {
                            font_size: self.text_style.font_size,
                            color: span.color,
                            ..self.text_style.clone()
                        },
                    );
                    x += renderer.measure_text(&span.text, self.text_style.font_size);
                }
            } else {
                // Fallback: simple text rendering
                renderer.draw_text(
                    line,
                    Vec2::new(content_x, y),
                    TextStyle {
                        font_size: self.text_style.font_size,
                        color: self.text_style.color,
                        ..self.text_style.clone()
                    },
                );
            }
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

                // Only draw cursor if within visible area
                if cursor_y >= content_y && cursor_y < self.bounds.y + self.bounds.height {
                    renderer.fill_rect(
                        Rect::new(final_cursor_x, cursor_y, 2.0, self.line_height),
                        self.text_style.color,
                    );
                }

                self.cached_cursor_x.set(final_cursor_x);
                self.cached_cursor_y.set(cursor_y);
            }
        }

        // 6. Draw scrollbars
        let scrollbar_width = 8.0;
        let content_height = self.lines.len() as f32 * self.line_height;
        let visible_height = self.bounds.height - padding * 2.0;

        // Vertical scrollbar
        if self.show_scrollbar_y && content_height > visible_height {
            let track_x = self.bounds.x + self.bounds.width - scrollbar_width - 2.0;
            let track_y = self.bounds.y + padding;
            let track_height = visible_height;

            // Track background
            renderer.fill_rect(
                Rect::new(track_x, track_y, scrollbar_width, track_height),
                Color::new(0.15, 0.15, 0.18, 0.8),
            );

            // Thumb
            let max_scroll = (content_height - visible_height).max(1.0);
            let thumb_height = (visible_height / content_height * track_height).max(20.0);
            let thumb_y =
                track_y + (self.scroll_offset.y / max_scroll) * (track_height - thumb_height);

            renderer.fill_rect(
                Rect::new(track_x, thumb_y, scrollbar_width, thumb_height),
                Color::new(0.4, 0.4, 0.45, 0.9),
            );
        }

        // 7. Draw Minimap (VS Code style)
        if self.show_minimap {
            let minimap_x = self.bounds.x + self.bounds.width - self.minimap_width - 4.0;
            let minimap_y = self.bounds.y + padding;
            let minimap_height = visible_height;

            // Minimap background
            renderer.fill_rect(
                Rect::new(minimap_x, minimap_y, self.minimap_width, minimap_height),
                Color::new(0.08, 0.08, 0.1, 0.95),
            );

            // Calculate minimap scale
            let total_lines = self.lines.len() as f32;
            let minimap_line_height = 2.0; // Each line is 2px tall in minimap
            let minimap_content_height = total_lines * minimap_line_height;

            // Calculate what portion of content is visible
            /*let scale = if minimap_content_height > minimap_height {
                minimap_height / minimap_content_height
            } else {
                1.0
            }; // */

            // Calculate minimap scroll offset to keep viewport visible
            let minimap_scroll = if minimap_content_height > minimap_height {
                let scroll_ratio = self.scroll_offset.y / content_height.max(1.0);
                scroll_ratio * (minimap_content_height - minimap_height)
            } else {
                0.0
            };

            // Draw minimap lines
            for (i, line) in self.lines.iter().enumerate() {
                let line_y = minimap_y + (i as f32 * minimap_line_height) - minimap_scroll;

                // Skip lines outside minimap area
                if line_y < minimap_y - minimap_line_height || line_y > minimap_y + minimap_height {
                    continue;
                }

                if line.is_empty() {
                    continue;
                }

                // Use syntax tokens if available for coloring
                if self.syntax_theme.enabled && i < self.cached_tokens.len() {
                    let mut x = minimap_x + 2.0;
                    for span in &self.cached_tokens[i] {
                        let char_width = 1.0; // Each char is ~1px in minimap
                        let span_width =
                            (span.text.len() as f32 * char_width).min(self.minimap_width - 4.0);

                        if span_width > 0.5 {
                            let mut color = span.color;
                            color.a *= 0.8;
                            renderer.fill_rect(
                                Rect::new(x, line_y, span_width, minimap_line_height - 0.5),
                                color,
                            );
                        }
                        x += span_width;
                        if x > minimap_x + self.minimap_width - 2.0 {
                            break;
                        }
                    }
                } else {
                    // Simple gray line representation
                    let line_width = (line.len() as f32 * 0.8).min(self.minimap_width - 4.0);
                    renderer.fill_rect(
                        Rect::new(
                            minimap_x + 2.0,
                            line_y,
                            line_width,
                            minimap_line_height - 0.5,
                        ),
                        Color::new(0.6, 0.6, 0.65, 0.6),
                    );
                }
            }

            // Draw viewport indicator (the visible area rectangle)
            let viewport_ratio = visible_height / content_height;
            let viewport_height = (minimap_height * viewport_ratio)
                .max(20.0)
                .min(minimap_height);
            let viewport_y = minimap_y
                + (self.scroll_offset.y / content_height.max(1.0))
                    * (minimap_height - viewport_height);

            renderer.fill_rect(
                Rect::new(minimap_x, viewport_y, self.minimap_width, viewport_height),
                Color::new(0.5, 0.5, 0.55, 0.25),
            );

            // Viewport border
            renderer.fill_rect(
                Rect::new(minimap_x, viewport_y, 2.0, viewport_height),
                Color::new(0.6, 0.6, 0.7, 0.5),
            );
        }
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        // Sync focus state from the singleton - engine is the source of truth
        if !self.id.is_empty() {
            self.is_focused = ctx.is_focused(&self.id);
        }

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
            // MouseWheel: scroll the content
            OxidXEvent::MouseWheel { delta, position } => {
                if self.bounds.contains(*position) {
                    // Calculate max scroll
                    let content_height = self.lines.len() as f32 * self.line_height;
                    let visible_height = self.bounds.height - self.layout.padding * 2.0;
                    let max_scroll = (content_height - visible_height).max(0.0);

                    // Apply scroll - delta.y is positive when scrolling down
                    self.scroll_offset.y = (self.scroll_offset.y - delta.y).clamp(0.0, max_scroll);
                    return true;
                }
                false
            }
            // FocusGained: reset cursor blink when we gain focus
            OxidXEvent::FocusGained { id } if id == &self.id => {
                self.reset_cursor_blink();
                true
            }
            // FocusLost: cleanup when we lose focus
            OxidXEvent::FocusLost { id } if id == &self.id => {
                self.is_selecting = false;
                self.ime_preedit.clear();
                true
            }
            OxidXEvent::MouseDown {
                position,
                modifiers,
                ..
            } => {
                let padding = self.layout.padding;
                let content_height = self.lines.len() as f32 * self.line_height;
                let visible_height = self.bounds.height - padding * 2.0;

                // Check if clicking on minimap
                if self.show_minimap {
                    let minimap_x = self.bounds.x + self.bounds.width - self.minimap_width - 4.0;

                    if position.x >= minimap_x && position.x <= self.bounds.x + self.bounds.width {
                        let minimap_y = self.bounds.y + padding;
                        let max_scroll = (content_height - visible_height).max(0.0);

                        // Center the viewport on click position
                        let click_ratio =
                            ((position.y - minimap_y) / visible_height).clamp(0.0, 1.0);
                        self.scroll_offset.y = (click_ratio * content_height
                            - visible_height / 2.0)
                            .clamp(0.0, max_scroll);
                        self.is_dragging_minimap = true;
                        return true;
                    }
                }

                // Check if clicking on scrollbar
                let scrollbar_width = 8.0;

                if self.show_scrollbar_y && content_height > visible_height {
                    let scrollbar_x = self.bounds.x + self.bounds.width - scrollbar_width - 2.0;

                    // If click is on scrollbar area
                    if position.x >= scrollbar_x
                        && position.x <= scrollbar_x + scrollbar_width + 2.0
                    {
                        let track_y = self.bounds.y + padding;
                        let track_height = visible_height;
                        let max_scroll = (content_height - visible_height).max(1.0);

                        // Calculate scroll position from click
                        let click_ratio = ((position.y - track_y) / track_height).clamp(0.0, 1.0);
                        self.scroll_offset.y = click_ratio * max_scroll;
                        self.is_dragging_scrollbar = true;
                        return true;
                    }
                }

                if !self.id.is_empty() {
                    ctx.request_focus(&self.id);
                }
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
                // Minimap drag has priority
                if self.is_dragging_minimap {
                    let padding = self.layout.padding;
                    let content_height = self.lines.len() as f32 * self.line_height;
                    let visible_height = self.bounds.height - padding * 2.0;
                    let minimap_y = self.bounds.y + padding;
                    let max_scroll = (content_height - visible_height).max(0.0);

                    let click_ratio = ((position.y - minimap_y) / visible_height).clamp(0.0, 1.0);
                    self.scroll_offset.y = (click_ratio * content_height - visible_height / 2.0)
                        .clamp(0.0, max_scroll);
                    return true;
                }

                // Scrollbar drag has priority
                if self.is_dragging_scrollbar {
                    let padding = self.layout.padding;
                    let content_height = self.lines.len() as f32 * self.line_height;
                    let visible_height = self.bounds.height - padding * 2.0;
                    let track_y = self.bounds.y + padding;
                    let track_height = visible_height;
                    let max_scroll = (content_height - visible_height).max(1.0);

                    let click_ratio = ((position.y - track_y) / track_height).clamp(0.0, 1.0);
                    self.scroll_offset.y = click_ratio * max_scroll;
                    return true;
                }

                // Text selection
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
                // Release minimap drag
                if self.is_dragging_minimap {
                    self.is_dragging_minimap = false;
                    return true;
                }
                // Release scrollbar drag
                if self.is_dragging_scrollbar {
                    self.is_dragging_scrollbar = false;
                    return true;
                }
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
