//! Label Component
//!
//! A text display component with:
//! - Multiple text styles (heading, body, caption)
//! - Text alignment (left, center, right)
//! - Text wrapping support
//! - Rich text with inline styling
//! - Selectable text option
//! - Truncation with ellipsis

use glam::Vec2;
use oxidx_core::component::OxidXComponent;
use oxidx_core::events::{KeyCode, OxidXEvent};
use oxidx_core::layout::LayoutProps;
use oxidx_core::primitives::{Color, Rect, TextAlign, TextStyle};
use oxidx_core::renderer::Renderer;
use oxidx_core::OxidXContext;
use std::cell::Cell;

/// Text style preset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LabelStyle {
    /// Large heading (24px)
    Heading1,
    /// Medium heading (20px)
    Heading2,
    /// Small heading (16px bold)
    Heading3,
    /// Normal body text (14px)
    #[default]
    Body,
    /// Smaller text (12px)
    Caption,
    /// Monospace code text (14px)
    Code,
}

/// Text overflow behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextOverflow {
    /// Allow text to overflow bounds
    #[default]
    Visible,
    /// Clip text at bounds
    Clip,
    /// Show ellipsis (...) when truncated
    Ellipsis,
    /// Wrap text to multiple lines
    Wrap,
}

/// A text label component for displaying text.
pub struct Label {
    // === Content ===
    text: String,

    // === Styling ===
    style: TextStyle,
    label_style: LabelStyle,
    layout: LayoutProps,

    // === Behavior ===
    overflow: TextOverflow,
    max_lines: Option<usize>,
    line_height_multiplier: f32,

    // === State ===
    bounds: Rect,
    id: String,

    // === Selectable Text ===
    is_selectable: bool,
    is_focused: bool,
    is_selecting: bool,
    selection_start: Option<usize>,
    selection_end: Option<usize>,

    // === Cached Values ===
    /// Cached measured text size
    #[allow(dead_code)]
    cached_text_size: Cell<Vec2>,
    /// Whether cache is valid
    cache_valid: Cell<bool>,
}

impl Label {
    /// Creates a new label with the given text.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: TextStyle::new(14.0).with_color(Color::WHITE),
            label_style: LabelStyle::Body,
            layout: LayoutProps::default(),
            overflow: TextOverflow::Visible,
            max_lines: None,
            line_height_multiplier: 1.4,
            bounds: Rect::default(),
            id: String::new(),
            is_selectable: false,
            is_focused: false,
            is_selecting: false,
            selection_start: None,
            selection_end: None,
            cached_text_size: Cell::new(Vec2::ZERO),
            cache_valid: Cell::new(false),
        }
    }

    // === Builder Methods ===

    /// Sets the text content.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self.cache_valid.set(false);
        self
    }

    /// Sets the font size.
    pub fn with_size(mut self, size: f32) -> Self {
        self.style.font_size = size;
        self.cache_valid.set(false);
        self
    }

    /// Sets the text color.
    pub fn with_color(mut self, color: Color) -> Self {
        self.style.color = color;
        self
    }

    /// Sets the text alignment.
    pub fn with_align(mut self, align: TextAlign) -> Self {
        self.style.align = align;
        self
    }

    /// Sets the label style preset.
    pub fn with_style(mut self, label_style: LabelStyle) -> Self {
        self.label_style = label_style;
        self.apply_label_style();
        self
    }

    /// Sets layout properties.
    pub fn with_layout(mut self, layout: LayoutProps) -> Self {
        self.layout = layout;
        self
    }

    /// Sets the overflow behavior.
    pub fn with_overflow(mut self, overflow: TextOverflow) -> Self {
        self.overflow = overflow;
        self
    }

    /// Sets the maximum number of lines (for wrapping).
    pub fn with_max_lines(mut self, max_lines: usize) -> Self {
        self.max_lines = Some(max_lines);
        self
    }

    /// Sets the line height multiplier.
    pub fn with_line_height(mut self, multiplier: f32) -> Self {
        self.line_height_multiplier = multiplier;
        self
    }

    /// Makes the text selectable.
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.is_selectable = selectable;
        self
    }

    /// Sets the component ID.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    // === Mutable Setters ===

    /// Sets the text content (mutable reference).
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
        self.cache_valid.set(false);
    }

    /// Sets the font size (mutable reference).
    pub fn set_size(&mut self, size: f32) {
        self.style.font_size = size;
        self.cache_valid.set(false);
    }

    /// Sets the text color (mutable reference).
    pub fn set_color(&mut self, color: Color) {
        self.style.color = color;
    }

    /// Sets the alignment (mutable reference).
    pub fn set_align(&mut self, align: TextAlign) {
        self.style.align = align;
    }

    // === Public API ===

    /// Returns the current text.
    pub fn get_text(&self) -> &str {
        &self.text
    }

    /// Returns whether text is currently selected.
    pub fn has_selection(&self) -> bool {
        self.selection_start.is_some()
            && self.selection_end.is_some()
            && self.selection_start != self.selection_end
    }

    /// Returns the selected text.
    pub fn selected_text(&self) -> String {
        if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
            let (s, e) = if start < end {
                (start, end)
            } else {
                (end, start)
            };
            self.text.chars().skip(s).take(e - s).collect()
        } else {
            String::new()
        }
    }

    // === Helper Methods ===

    /// Applies the label style preset.
    fn apply_label_style(&mut self) {
        match self.label_style {
            LabelStyle::Heading1 => {
                self.style.font_size = 24.0;
            }
            LabelStyle::Heading2 => {
                self.style.font_size = 20.0;
            }
            LabelStyle::Heading3 => {
                self.style.font_size = 16.0;
            }
            LabelStyle::Body => {
                self.style.font_size = 14.0;
            }
            LabelStyle::Caption => {
                self.style.font_size = 12.0;
                self.style.color = Color::new(0.7, 0.7, 0.7, 1.0);
            }
            LabelStyle::Code => {
                self.style.font_size = 14.0;
                self.style.font_family = Some("monospace".to_string());
            }
        }
        self.cache_valid.set(false);
    }

    /// Measures text and returns size.
    fn _measure_text(&self, renderer: &mut Renderer) -> Vec2 {
        if self.cache_valid.get() {
            return self.cached_text_size.get();
        }

        let width = renderer.measure_text(&self.text, self.style.font_size);
        let height = self.style.font_size * self.line_height_multiplier;

        let size = Vec2::new(width, height);
        self.cached_text_size.set(size);
        self.cache_valid.set(true);
        size
    }

    /// Truncates text with ellipsis if needed.
    fn truncate_text(&self, renderer: &mut Renderer, max_width: f32) -> String {
        if self.text.is_empty() {
            return String::new();
        }

        let full_width = renderer.measure_text(&self.text, self.style.font_size);
        if full_width <= max_width {
            return self.text.clone();
        }

        let ellipsis = "...";
        let ellipsis_width = renderer.measure_text(ellipsis, self.style.font_size);
        let available = max_width - ellipsis_width;

        if available <= 0.0 {
            return ellipsis.to_string();
        }

        // Binary search for the right length
        let mut result = String::new();
        for ch in self.text.chars() {
            result.push(ch);
            let width = renderer.measure_text(&result, self.style.font_size);
            if width > available {
                result.pop();
                break;
            }
        }

        result + ellipsis
    }

    /// Converts click X position to character index.
    fn _click_to_char_index(&self, click_x: f32, renderer: &mut Renderer) -> usize {
        let text_start_x = self.bounds.x + self.layout.padding;
        let relative_x = click_x - text_start_x;

        if relative_x <= 0.0 || self.text.is_empty() {
            return 0;
        }

        let char_count = self.text.chars().count();
        let mut prev_width = 0.0;

        for i in 1..=char_count {
            let prefix: String = self.text.chars().take(i).collect();
            let width = renderer.measure_text(&prefix, self.style.font_size);
            let mid = (prev_width + width) / 2.0;

            if relative_x < mid {
                return i - 1;
            }
            if relative_x < width {
                return i;
            }
            prev_width = width;
        }

        char_count
    }

    /// Clears the selection.
    fn clear_selection(&mut self) {
        self.selection_start = None;
        self.selection_end = None;
    }
}

impl Default for Label {
    fn default() -> Self {
        Self::new("")
    }
}

impl OxidXComponent for Label {
    fn update(&mut self, _dt: f32) {
        // Labels don't animate by default
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        let margin = self.layout.margin;
        let padding = self.layout.padding;

        self.bounds = Rect::new(
            available.x + margin,
            available.y + margin,
            available.width - margin * 2.0,
            available.height - margin * 2.0,
        );

        // Approximate size (will be refined in render)
        let char_width = self.style.font_size * 0.6;
        let width = (self.text.len() as f32 * char_width + padding * 2.0).min(available.width);
        let height = self.style.font_size * self.line_height_multiplier + padding * 2.0;

        Vec2::new(width + margin * 2.0, height + margin * 2.0)
    }

    fn render(&self, renderer: &mut Renderer) {
        let padding = self.layout.padding;
        let content_width = self.bounds.width - padding * 2.0;

        // Handle text wrapping
        if self.overflow == TextOverflow::Wrap {
            // Greedy word-wrap algorithm
            let line_height = self.style.font_size * self.line_height_multiplier;
            let mut y = self.bounds.y + padding;
            let mut lines_rendered = 0;

            let words: Vec<&str> = self.text.split_whitespace().collect();
            let mut current_line = String::new();
            //let space_width = renderer.measure_text(" ", self.style.font_size);

            for word in words {
                //let word_width = renderer.measure_text(word, self.style.font_size);
                let test_line = if current_line.is_empty() {
                    word.to_string()
                } else {
                    format!("{} {}", current_line, word)
                };
                let test_width = renderer.measure_text(&test_line, self.style.font_size);

                if test_width > content_width && !current_line.is_empty() {
                    // Render current line
                    let text_x = match self.style.align {
                        TextAlign::Left => self.bounds.x + padding,
                        TextAlign::Center => {
                            let w = renderer.measure_text(&current_line, self.style.font_size);
                            self.bounds.x + (self.bounds.width - w) / 2.0
                        }
                        TextAlign::Right => {
                            let w = renderer.measure_text(&current_line, self.style.font_size);
                            self.bounds.x + self.bounds.width - padding - w
                        }
                    };
                    renderer.draw_text(&current_line, Vec2::new(text_x, y), self.style.clone());
                    y += line_height;
                    lines_rendered += 1;

                    // Check max lines
                    if let Some(max) = self.max_lines {
                        if lines_rendered >= max {
                            return;
                        }
                    }

                    current_line = word.to_string();
                } else {
                    current_line = test_line;
                }
            }

            // Render last line
            if !current_line.is_empty() {
                let text_x = match self.style.align {
                    TextAlign::Left => self.bounds.x + padding,
                    TextAlign::Center => {
                        let w = renderer.measure_text(&current_line, self.style.font_size);
                        self.bounds.x + (self.bounds.width - w) / 2.0
                    }
                    TextAlign::Right => {
                        let w = renderer.measure_text(&current_line, self.style.font_size);
                        self.bounds.x + self.bounds.width - padding - w
                    }
                };
                renderer.draw_text(&current_line, Vec2::new(text_x, y), self.style.clone());
            }

            // Draw focus indicator if focused
            if self.is_focused && self.is_selectable {
                renderer.stroke_rect(self.bounds, Color::new(0.4, 0.6, 1.0, 0.5), 1.0);
            }
            return;
        }

        // Non-wrapping modes
        let display_text = match self.overflow {
            TextOverflow::Visible | TextOverflow::Clip => self.text.clone(),
            TextOverflow::Ellipsis => self.truncate_text(renderer, content_width),
            TextOverflow::Wrap => unreachable!(), // handled above
        };

        // Calculate text position based on alignment
        let text_width = renderer.measure_text(&display_text, self.style.font_size);
        let text_x = match self.style.align {
            TextAlign::Left => self.bounds.x + padding,
            TextAlign::Center => self.bounds.x + (self.bounds.width - text_width) / 2.0,
            TextAlign::Right => self.bounds.x + self.bounds.width - padding - text_width,
        };
        let text_y = self.bounds.y + padding;

        // Draw selection background if selectable and has selection
        if self.is_selectable && self.has_selection() {
            if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
                let (s, e) = if start < end {
                    (start, end)
                } else {
                    (end, start)
                };

                let prefix_start: String = self.text.chars().take(s).collect();
                let prefix_end: String = self.text.chars().take(e).collect();

                let start_x = renderer.measure_text(&prefix_start, self.style.font_size);
                let end_x = renderer.measure_text(&prefix_end, self.style.font_size);

                let selection_rect = Rect::new(
                    text_x + start_x,
                    text_y,
                    end_x - start_x,
                    self.style.font_size * self.line_height_multiplier,
                );
                renderer.fill_rect(selection_rect, Color::new(0.2, 0.5, 0.9, 0.5));
            }
        }

        // Draw text
        renderer.draw_text(&display_text, Vec2::new(text_x, text_y), self.style.clone());

        // Draw focus indicator if focused
        if self.is_focused && self.is_selectable {
            renderer.stroke_rect(self.bounds, Color::new(0.4, 0.6, 1.0, 0.5), 1.0);
        }
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        if !self.is_selectable {
            return false;
        }

        // Sync focus state from the singleton - engine is the source of truth
        if !self.id.is_empty() {
            self.is_focused = ctx.is_focused(&self.id);
        }

        match event {
            OxidXEvent::MouseDown { position, .. } => {
                if !self.bounds.contains(*position) {
                    return false;
                }

                if !self.id.is_empty() {
                    ctx.request_focus(&self.id);
                }
                self.is_selecting = true;

                // Approximate position (refined in render)
                let text_start_x = self.bounds.x + self.layout.padding;
                let relative_x = (position.x - text_start_x).max(0.0);
                let approx_pos = if self.text.is_empty() {
                    0
                } else {
                    ((relative_x / 8.0) as usize).min(self.text.chars().count())
                };

                self.selection_start = Some(approx_pos);
                self.selection_end = Some(approx_pos);
                true
            }
            OxidXEvent::MouseMove { position, .. } => {
                if self.is_selecting {
                    let text_start_x = self.bounds.x + self.layout.padding;
                    let relative_x = (position.x - text_start_x).max(0.0);
                    let approx_pos = if self.text.is_empty() {
                        0
                    } else {
                        ((relative_x / 8.0) as usize).min(self.text.chars().count())
                    };
                    self.selection_end = Some(approx_pos);
                    true
                } else {
                    false
                }
            }
            OxidXEvent::MouseUp { .. } => {
                self.is_selecting = false;
                if self.selection_start == self.selection_end {
                    self.clear_selection();
                }
                true
            }
            // FocusLost: cleanup when we lose focus
            OxidXEvent::FocusLost { id } if id == &self.id => {
                self.clear_selection();
                true
            }
            _ => false,
        }
    }

    fn on_keyboard_input(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) {
        if !self.is_selectable || !ctx.is_focused(&self.id) {
            return;
        }

        match event {
            OxidXEvent::KeyDown { key, modifiers } => {
                // Ctrl+A: Select All
                if modifiers.ctrl && *key == KeyCode::KEY_A {
                    self.selection_start = Some(0);
                    self.selection_end = Some(self.text.chars().count());
                    return;
                }

                // Ctrl+C: Copy
                if modifiers.ctrl && *key == KeyCode::KEY_C {
                    if self.has_selection() {
                        ctx.copy_to_clipboard(&self.selected_text());
                    } else {
                        ctx.copy_to_clipboard(&self.text);
                    }
                    return;
                }

                // Escape: Clear selection
                if *key == KeyCode::ESCAPE {
                    self.clear_selection();
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
        self.is_selectable
    }
}
