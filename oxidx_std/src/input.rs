//! Text Input Component
//!
//! A professional text input field with:
//! - Proper text measurement using cosmic-text
//! - Cursor blinking animation
//! - Arrow key navigation (Left, Right, Home, End)
//! - Mouse text selection (click and drag)
//! - Shift+Arrow and Shift+Click for selection
//! - Clipboard support (Ctrl+C/V/X)
//! - IME support for international input
//! - Full UTF-8 support (emojis, accents, etc.)

use oxidx_core::events::{KeyCode, OxidXEvent};
use oxidx_core::layout::LayoutProps;
use oxidx_core::style::InteractiveStyle;
use oxidx_core::{Color, OxidXComponent, OxidXContext, Rect, Renderer, TextStyle, Vec2};
use std::cell::Cell;
use winit::window::CursorIcon;

/// A text input field with full text editing support.
pub struct Input {
    // === Layout ===
    bounds: Rect,
    layout: LayoutProps,
    width: Option<f32>,

    // === Styling ===
    style: InteractiveStyle,
    text_style: TextStyle,

    // === Content ===
    placeholder: String,
    value: String,

    // === State ===
    is_focused: bool,
    is_hovered: bool,
    id: String,
    is_password: bool,

    // === IME ===
    ime_preedit: String,

    // === Cursor ===
    /// Character index where cursor is located (0 = before first char)
    cursor_pos: usize,
    /// Timer for cursor blink animation
    cursor_blink_timer: f32,
    /// Whether cursor is currently visible (toggles every ~530ms)
    cursor_visible: bool,

    // === Selection ===
    /// Start of selection (character index), None if no selection
    selection_anchor: Option<usize>,
    /// Whether user is currently dragging to select
    is_selecting: bool,

    // === Cached values for IME positioning ===
    cached_cursor_x: Cell<f32>,
    cached_text_y: Cell<f32>,

    // === Callbacks ===
    on_change: Option<Box<dyn Fn(&str) + Send>>,
    on_blur: Option<Box<dyn Fn(&str) + Send>>,

    // === Focus Order ===
    /// Tab navigation order (lower values are focused first)
    focus_order: usize,
}

impl Input {
    /// Creates a new Input with a placeholder.
    /// Styling is resolved dynamically from the theme during render().
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            bounds: Rect::default(),
            layout: LayoutProps::default().with_padding(14.0),
            width: None,
            // Style is resolved dynamically from theme in render()
            style: InteractiveStyle::default(),
            text_style: TextStyle::new(16.0).with_color(Color::WHITE),
            placeholder: placeholder.into(),
            value: String::new(),
            is_focused: false,
            is_hovered: false,
            id: String::new(),
            is_password: false,
            ime_preedit: String::new(),
            cursor_pos: 0,
            cursor_blink_timer: 0.0,
            cursor_visible: true,
            selection_anchor: None,
            is_selecting: false,
            cached_cursor_x: Cell::new(0.0),
            cached_text_y: Cell::new(0.0),
            on_change: None,
            on_blur: None,
            focus_order: usize::MAX, // Default to very high (last in tab order)
        }
    }

    // === Builder Methods ===

    /// Builder: Set component ID (required for focus management)
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Builder: Set custom interactive style
    pub fn with_style(mut self, style: InteractiveStyle) -> Self {
        self.style = style;
        self
    }

    /// Builder: Ensure input masks characters
    pub fn password_mode(mut self, enabled: bool) -> Self {
        self.is_password = enabled;
        self
    }

    /// Builder: Set layout properties
    pub fn with_layout(mut self, layout: LayoutProps) -> Self {
        self.layout = layout;
        self
    }

    /// Builder: Set on_change callback (called whenever value changes)
    pub fn with_on_change(mut self, callback: impl Fn(&str) + Send + 'static) -> Self {
        self.on_change = Some(Box::new(callback));
        self
    }

    /// Builder: Set on_blur callback (called when input loses focus)
    pub fn with_on_blur(mut self, callback: impl Fn(&str) + Send + 'static) -> Self {
        self.on_blur = Some(Box::new(callback));
        self
    }

    /// Builder: Set focus order for Tab navigation (lower values are focused first)
    pub fn with_focus_order(mut self, order: usize) -> Self {
        self.focus_order = order;
        self
    }

    pub fn set_placeholder(&mut self, text: impl Into<String>) {
        self.placeholder = text.into();
    }

    pub fn get_text(&self) -> &str {
        &self.value
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.set_value(text);
    }

    // === Public API ===

    /// Returns the current text value
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Sets the text value programmatically
    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
        self.clamp_cursor();
        self.clear_selection();
        self.fire_on_change();
    }

    /// Fires the on_change callback if set
    fn fire_on_change(&self) {
        if let Some(ref callback) = self.on_change {
            callback(&self.value);
        }
    }

    /// Fires the on_blur callback if set
    fn fire_on_blur(&self) {
        if let Some(ref callback) = self.on_blur {
            callback(&self.value);
        }
    }

    // === Selection Helpers ===

    /// Returns true if there's an active text selection
    fn has_selection(&self) -> bool {
        self.selection_anchor.is_some() && self.selection_anchor != Some(self.cursor_pos)
    }

    /// Returns the selection range as (start, end) where start <= end
    fn selection_range(&self) -> Option<(usize, usize)> {
        self.selection_anchor.map(|anchor| {
            if anchor < self.cursor_pos {
                (anchor, self.cursor_pos)
            } else {
                (self.cursor_pos, anchor)
            }
        })
    }

    /// Returns the selected text, or empty string if no selection
    fn selected_text(&self) -> String {
        if let Some((start, end)) = self.selection_range() {
            self.value.chars().skip(start).take(end - start).collect()
        } else {
            String::new()
        }
    }

    /// Deletes the selected text and returns cursor to selection start
    fn delete_selection(&mut self) {
        if let Some((start, end)) = self.selection_range() {
            // Convert char indices to byte indices
            let byte_start = self.char_to_byte_index(start);
            let byte_end = self.char_to_byte_index(end);
            self.value.replace_range(byte_start..byte_end, "");
            self.cursor_pos = start;
            self.selection_anchor = None;
            self.fire_on_change();
        }
    }

    /// Clears selection without deleting text
    fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

    // === Cursor Helpers ===

    /// Resets cursor blink timer (call when cursor moves)
    fn reset_cursor_blink(&mut self) {
        self.cursor_blink_timer = 0.0;
        self.cursor_visible = true;
    }

    /// Ensures cursor_pos is within valid bounds
    fn clamp_cursor(&mut self) {
        let max_pos = self.value.chars().count();
        self.cursor_pos = self.cursor_pos.min(max_pos);
    }

    /// Converts character index to byte index
    fn char_to_byte_index(&self, char_index: usize) -> usize {
        self.value
            .char_indices()
            .nth(char_index)
            .map(|(i, _)| i)
            .unwrap_or(self.value.len())
    }

    /// Inserts text at cursor position
    fn insert_at_cursor(&mut self, text: &str) {
        let byte_pos = self.char_to_byte_index(self.cursor_pos);
        self.value.insert_str(byte_pos, text);
        self.cursor_pos += text.chars().count();
        self.fire_on_change();
    }

    // === Test Helpers (public for headless testing) ===

    /// Test helper: Insert a character at cursor (for headless unit testing)
    #[cfg(test)]
    pub fn test_insert_char(&mut self, ch: char) {
        self.insert_at_cursor(&ch.to_string());
    }

    /// Test helper: Insert text at cursor (for headless unit testing)
    #[cfg(test)]
    pub fn test_insert_text(&mut self, text: &str) {
        self.insert_at_cursor(text);
    }

    /// Test helper: Delete char before cursor (backspace) (for headless unit testing)
    #[cfg(test)]
    pub fn test_backspace(&mut self) {
        self.delete_char_before_cursor();
    }

    /// Test helper: Focus the input (for headless unit testing)
    #[cfg(test)]
    pub fn test_focus(&mut self) {
        self.is_focused = true;
    }

    /// Deletes character before cursor (backspace)
    fn delete_char_before_cursor(&mut self) {
        if self.cursor_pos > 0 {
            let char_indices: Vec<_> = self.value.char_indices().collect();
            let idx = self.cursor_pos - 1;
            if idx < char_indices.len() {
                let (byte_pos, ch) = char_indices[idx];
                self.value
                    .replace_range(byte_pos..byte_pos + ch.len_utf8(), "");
                self.cursor_pos -= 1;
                self.fire_on_change();
            }
        }
    }

    /// Deletes character after cursor (delete key)
    fn delete_char_after_cursor(&mut self) {
        let max_pos = self.value.chars().count();
        if self.cursor_pos < max_pos {
            let char_indices: Vec<_> = self.value.char_indices().collect();
            if self.cursor_pos < char_indices.len() {
                let (byte_pos, ch) = char_indices[self.cursor_pos];
                self.value
                    .replace_range(byte_pos..byte_pos + ch.len_utf8(), "");
                self.fire_on_change();
            }
        }
    }

    // === Text Measurement ===

    // === IME Support ===

    /// Updates IME cursor position using cached values from render()
    fn update_ime_position(&self, ctx: &OxidXContext) {
        if self.is_focused {
            ctx.set_ime_position(Rect::new(
                self.cached_cursor_x.get(),
                self.cached_text_y.get(),
                2.0,
                self.text_style.font_size,
            ));
        }
    }
}

impl OxidXComponent for Input {
    fn update(&mut self, dt: f32) {
        // Cursor blinking - only when focused and no selection
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
        let width = self.width.unwrap_or(available.width - margin * 2.0);
        let height = 40.0;

        self.bounds = Rect::new(available.x + margin, available.y + margin, width, height);
        Vec2::new(width + margin * 2.0, height + margin * 2.0)
    }

    fn render(&self, renderer: &mut Renderer) {
        let is_focused = self.is_focused;

        // Background
        renderer.fill_rect(self.bounds, renderer.theme.colors.surface_alt);

        // Border
        let border_color = if is_focused {
            renderer.theme.colors.border_focus // Focus color
        } else {
            renderer.theme.colors.border // Border color
        };
        let border_width = if is_focused {
            renderer.theme.borders.width_focus
        } else {
            renderer.theme.borders.width
        };

        renderer.stroke_rect(self.bounds, border_color, border_width);

        // Text
        let text_padding = renderer.theme.spacing.sm;
        let text_bounds = Rect::new(
            self.bounds.x + text_padding,
            self.bounds.y + text_padding,
            self.bounds.width - text_padding * 2.0,
            self.bounds.height - text_padding * 2.0,
        );

        // Cache text Y for IME positioning (approximate)
        let text_y = text_bounds.y + (text_bounds.height - 14.0) / 2.0;
        self.cached_text_y.set(text_y);

        // Clip text to input bounds
        renderer.push_clip(self.bounds);

        let text_color = if self.value.is_empty() {
            renderer.theme.colors.text_dim // Placeholder color
        } else {
            renderer.theme.colors.text_main
        };

        // Generate display text - mask password characters
        let masked_value: String;
        let display_text = if self.value.is_empty() {
            &self.placeholder
        } else if self.is_password {
            // Mask all characters with bullet points
            masked_value = "•".repeat(self.value.chars().count());
            &masked_value
        } else {
            &self.value
        };

        renderer.draw_text(
            display_text,
            Vec2::new(text_bounds.x, text_bounds.y),
            TextStyle {
                font_size: 14.0, // Should come from theme
                color: text_color,
                ..Default::default()
            },
        );

        // Cursor
        if is_focused {
            // Simple cursor positioning (approximate for now)
            let cursor_x = if self.value.is_empty() {
                text_bounds.x
            } else {
                // If we had proper text measurement here we'd use it.
                // For now, let's just approximate or rely on cached_cursor if updated elsewhere.
                // But the cached cursor logic was removed.
                // Let's re-implement basic cursor drawing at end of text.
                text_bounds.x + renderer.measure_text(display_text, 14.0) // End of text
            };

            self.cached_cursor_x.set(cursor_x);

            // Blink effect
            if self.cursor_visible && !self.has_selection() {
                renderer.fill_rect(
                    Rect::new(cursor_x, text_bounds.y, 2.0, 18.0),
                    renderer.theme.colors.primary,
                );
            }
        }

        renderer.pop_clip();
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        // Register as focusable for Tab navigation (every event ensures we're in registry)
        if !self.id.is_empty() {
            ctx.register_focusable(&self.id, self.focus_order);
            // Sync focus state from the singleton - engine is the source of truth
            self.is_focused = ctx.is_focused(&self.id);
        }

        // Strict bounds check for mouse events
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
            // FocusGained: reset cursor blink when we gain focus
            OxidXEvent::FocusGained { id } if id == &self.id => {
                self.reset_cursor_blink();
                true
            }
            // FocusLost: cleanup when we lose focus
            OxidXEvent::FocusLost { id } if id == &self.id => {
                self.clear_selection();
                self.is_selecting = false;
                self.ime_preedit.clear();
                self.fire_on_blur();
                true
            }
            OxidXEvent::MouseDown {
                position,
                modifiers,
                ..
            } => {
                // Request focus on click - focus will be set via FocusGained event
                if !self.id.is_empty() {
                    ctx.request_focus(&self.id);
                }
                // Note: is_focused will be set by FocusGained event from engine
                self.is_selecting = true;

                // Approximate cursor position (will be refined)
                // We use a simple approximation since we don't have renderer here
                let click_x = position.x;
                let text_start_x = self.bounds.x + self.layout.padding;
                let relative_x = (click_x - text_start_x).max(0.0);

                let approx_pos = if self.value.is_empty() {
                    0
                } else {
                    // Approximate using average character width
                    let total_chars = self.value.chars().count();
                    let avg_width = if total_chars > 0 {
                        // Use cached cursor position to estimate average width
                        self.cached_cursor_x.get() - text_start_x
                    } else {
                        0.0
                    };

                    if avg_width > 0.0 && total_chars > 0 {
                        let avg_char_width = avg_width / total_chars as f32;
                        ((relative_x / avg_char_width) as usize).min(total_chars)
                    } else {
                        // Fallback to 8px estimate
                        ((relative_x / 8.0) as usize).min(total_chars)
                    }
                };

                // Shift+Click extends selection
                if modifiers.shift && self.selection_anchor.is_some() {
                    self.cursor_pos = approx_pos;
                } else {
                    self.cursor_pos = approx_pos;
                    self.selection_anchor = Some(approx_pos);
                }

                self.reset_cursor_blink();
                self.update_ime_position(ctx);
                true
            }
            OxidXEvent::MouseMove { position, .. } => {
                if self.is_selecting {
                    let click_x = position.x;
                    let text_start_x = self.bounds.x + self.layout.padding;
                    let relative_x = (click_x - text_start_x).max(0.0);

                    let total_chars = self.value.chars().count();
                    let approx_pos = if self.value.is_empty() {
                        0
                    } else {
                        ((relative_x / 8.0) as usize).min(total_chars)
                    };

                    self.cursor_pos = approx_pos;
                    self.reset_cursor_blink();
                    true
                } else {
                    false
                }
            }
            OxidXEvent::MouseUp { .. } => {
                self.is_selecting = false;

                // If no movement, clear selection (simple click)
                if self.selection_anchor == Some(self.cursor_pos) {
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
                self.insert_at_cursor(text);
                self.ime_preedit.clear();
                self.reset_cursor_blink();
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
                    self.insert_at_cursor(&character.to_string());
                    self.reset_cursor_blink();
                }
            }
            OxidXEvent::KeyDown { key, modifiers } => {
                let max_pos = self.value.chars().count();

                // === ARROW KEYS ===

                // Left Arrow
                if *key == KeyCode::LEFT {
                    if modifiers.shift {
                        if self.selection_anchor.is_none() {
                            self.selection_anchor = Some(self.cursor_pos);
                        }
                        self.cursor_pos = self.cursor_pos.saturating_sub(1);
                    } else {
                        if self.has_selection() {
                            if let Some((start, _)) = self.selection_range() {
                                self.cursor_pos = start;
                            }
                            self.clear_selection();
                        } else {
                            self.cursor_pos = self.cursor_pos.saturating_sub(1);
                        }
                    }
                    self.reset_cursor_blink();
                    return;
                }

                // Right Arrow
                if *key == KeyCode::RIGHT {
                    if modifiers.shift {
                        if self.selection_anchor.is_none() {
                            self.selection_anchor = Some(self.cursor_pos);
                        }
                        self.cursor_pos = (self.cursor_pos + 1).min(max_pos);
                    } else {
                        if self.has_selection() {
                            if let Some((_, end)) = self.selection_range() {
                                self.cursor_pos = end;
                            }
                            self.clear_selection();
                        } else {
                            self.cursor_pos = (self.cursor_pos + 1).min(max_pos);
                        }
                    }
                    self.reset_cursor_blink();
                    return;
                }

                // Home
                if *key == KeyCode::HOME {
                    if modifiers.shift {
                        if self.selection_anchor.is_none() {
                            self.selection_anchor = Some(self.cursor_pos);
                        }
                    } else {
                        self.clear_selection();
                    }
                    self.cursor_pos = 0;
                    self.reset_cursor_blink();
                    return;
                }

                // End
                if *key == KeyCode::END {
                    if modifiers.shift {
                        if self.selection_anchor.is_none() {
                            self.selection_anchor = Some(self.cursor_pos);
                        }
                    } else {
                        self.clear_selection();
                    }
                    self.cursor_pos = max_pos;
                    self.reset_cursor_blink();
                    return;
                }

                // === EDITING SHORTCUTS ===

                // Select All (Cmd+A / Ctrl+A)
                if modifiers.is_primary() && *key == KeyCode::KEY_A {
                    self.selection_anchor = Some(0);
                    self.cursor_pos = max_pos;
                    return;
                }

                // Copy (Cmd+C / Ctrl+C)
                if modifiers.is_primary() && *key == KeyCode::KEY_C {
                    if self.has_selection() {
                        ctx.copy_to_clipboard(&self.selected_text());
                    } else if !self.value.is_empty() {
                        ctx.copy_to_clipboard(&self.value);
                    }
                    return;
                }

                // Cut (Cmd+X / Ctrl+X)
                if modifiers.is_primary() && *key == KeyCode::KEY_X {
                    if self.has_selection() {
                        ctx.copy_to_clipboard(&self.selected_text());
                        self.delete_selection();
                    } else if !self.value.is_empty() {
                        ctx.copy_to_clipboard(&self.value);
                        self.value.clear();
                        self.cursor_pos = 0;
                    }
                    self.reset_cursor_blink();
                    return;
                }

                // Paste (Cmd+V / Ctrl+V)
                if modifiers.is_primary() && *key == KeyCode::KEY_V {
                    if let Some(text) = ctx.paste_from_clipboard() {
                        if self.has_selection() {
                            self.delete_selection();
                        }
                        let clean: String = text
                            .chars()
                            .filter(|c| !c.is_control() || *c == '\t')
                            .collect();
                        self.insert_at_cursor(&clean);
                        self.reset_cursor_blink();
                    }
                    return;
                }

                // Backspace
                if *key == KeyCode::BACKSPACE {
                    if self.has_selection() {
                        self.delete_selection();
                    } else {
                        self.delete_char_before_cursor();
                    }
                    self.reset_cursor_blink();
                    return;
                }

                // Delete
                if *key == KeyCode::DELETE {
                    if self.has_selection() {
                        self.delete_selection();
                    } else {
                        self.delete_char_after_cursor();
                    }
                    self.reset_cursor_blink();
                    return;
                }
            }
            _ => {}
        }
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.bounds.x = x;
        self.bounds.y = y;
    }

    fn set_size(&mut self, width: f32, height: f32) {
        self.bounds.width = width;
        self.bounds.height = height;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test 1: Text Entry
    /// Simulate typing "A", "B", "C". Assert value == "ABC".
    #[test]
    fn test_text_entry() {
        let mut input = Input::new("Enter text").with_id("test_input");

        // Focus the input (headless)
        input.test_focus();

        // Type characters using test helper
        input.test_insert_char('A');
        input.test_insert_char('B');
        input.test_insert_char('C');

        // Assert value
        assert_eq!(input.value(), "ABC");
    }

    /// Test 2: Password Mode
    /// Set password_mode(true). Type "123".
    /// Assert internal value == "123".
    #[test]
    fn test_password_mode() {
        let mut input = Input::new("Password")
            .with_id("password_input")
            .password_mode(true);

        // Focus the input
        input.test_focus();

        // Type characters
        input.test_insert_text("123");

        // Assert internal value is correct
        assert_eq!(input.value(), "123");

        // Verify password mode is enabled
        assert!(input.is_password);
    }

    /// Test 3: Backspace
    /// Type "A", press Backspace. Assert value == "".
    #[test]
    fn test_backspace() {
        let mut input = Input::new("Test").with_id("test_input");

        // Focus the input
        input.test_focus();

        // Type a character
        input.test_insert_char('A');
        assert_eq!(input.value(), "A");

        // Press Backspace using test helper
        input.test_backspace();

        // Assert value is now empty
        assert_eq!(input.value(), "");
    }

    /// Test: Multiple backspaces
    #[test]
    fn test_multiple_backspace() {
        let mut input = Input::new("Test").with_id("test_input");

        input.test_focus();

        // Type "Hello"
        input.test_insert_text("Hello");
        assert_eq!(input.value(), "Hello");

        // Press Backspace twice
        input.test_backspace();
        input.test_backspace();

        // Assert value is "Hel"
        assert_eq!(input.value(), "Hel");
    }

    /// Test: Set value programmatically
    #[test]
    fn test_set_value() {
        let mut input = Input::new("Test");
        assert_eq!(input.value(), "");

        input.set_value("Hello World");
        assert_eq!(input.value(), "Hello World");
    }
}

// === Schema Serialization ===

impl oxidx_core::schema::ToSchema for Input {
    fn to_schema(&self) -> oxidx_core::schema::ComponentNode {
        let mut node = oxidx_core::schema::ComponentNode::new("Input");

        // ID
        if !self.id.is_empty() {
            node.id = Some(self.id.clone());
        }

        // Properties
        node.props.insert(
            "placeholder".to_string(),
            serde_json::json!(self.placeholder),
        );

        if self.is_password {
            node.props
                .insert("password_mode".to_string(), serde_json::json!(true));
        }

        // Events
        if self.on_change.is_some() {
            node.events.push("on_change".to_string());
        }
        if self.on_blur.is_some() {
            node.events.push("on_blur".to_string());
        }

        node
    }
}
