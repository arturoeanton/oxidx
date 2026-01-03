//! Text Input Component
//!
//! A text input field with:
//! - Clipboard support (Ctrl+C/V)
//! - Selection (Ctrl+A)
//! - Text cursor on hover

use oxidx_core::events::{KeyCode, OxidXEvent};
use oxidx_core::layout::LayoutProps;
use oxidx_core::style::{ComponentState, InteractiveStyle, Style};
use oxidx_core::{Color, OxidXComponent, OxidXContext, Rect, Renderer, TextStyle, Vec2};
use winit::window::CursorIcon;

/// A text input field with styling, layout, clipboard, and cursor support.
pub struct Input {
    bounds: Rect,
    style: InteractiveStyle,
    layout: LayoutProps,
    placeholder: String,
    value: String,
    is_focused: bool,
    is_hovered: bool,
    text_style: TextStyle,
    id: String,
    /// Whether all text is selected
    is_selected: bool,
    /// IME pre-edit text (composition)
    ime_preedit: String,
}

impl Input {
    /// Creates a new Input with a placeholder.
    pub fn new(placeholder: impl Into<String>) -> Self {
        // Default styling
        let border_color = Color::new(0.3, 0.3, 0.35, 1.0);
        let focus_color = Color::new(0.2, 0.5, 0.9, 1.0);
        let bg = Color::new(0.1, 0.1, 0.12, 1.0);
        let text_white = Color::WHITE;

        let idle = Style::new()
            .bg_solid(bg)
            .border(1.0, border_color)
            .rounded(4.0)
            .text_color(text_white);

        let hover = Style::new()
            .bg_solid(Color::new(0.12, 0.12, 0.15, 1.0))
            .border(1.0, Color::WHITE)
            .rounded(4.0)
            .text_color(text_white);

        let focused = Style::new()
            .bg_solid(bg)
            .border(2.0, focus_color)
            .rounded(4.0)
            .shadow(Vec2::new(0.0, 0.0), 8.0, focus_color)
            .text_color(text_white);

        let style = InteractiveStyle {
            idle,
            hover,
            pressed: focused,
            disabled: idle,
        };

        Self {
            bounds: Rect::default(),
            style,
            layout: LayoutProps::default().with_padding(10.0),
            placeholder: placeholder.into(),
            value: String::new(),
            is_focused: false,
            is_hovered: false,
            text_style: TextStyle::new(14.0).with_color(Color::WHITE),
            id: String::new(),
            is_selected: false,
            ime_preedit: String::new(),
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    pub fn with_style(mut self, style: InteractiveStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_layout(mut self, layout: LayoutProps) -> Self {
        self.layout = layout;
        self
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    /// Helper to update IME cursor position
    fn update_ime_position(&self, ctx: &OxidXContext) {
        if self.is_focused {
            let text_pos_y = self.bounds.y + (self.bounds.height - self.text_style.font_size) / 2.0;
            let padding = self.layout.padding;

            // Calculate cursor x (same logic as render)
            let base_width = self.value.len() as f32 * 8.0;
            let preedit_width = self.ime_preedit.len() as f32 * 8.0;
            let cursor_x = self.bounds.x + padding + base_width + preedit_width;

            ctx.set_ime_position(Rect::new(
                cursor_x,
                text_pos_y,
                2.0,
                self.text_style.font_size,
            ));
        }
    }
}

impl OxidXComponent for Input {
    fn update(&mut self, _dt: f32) {}

    fn layout(&mut self, available: Rect) -> Vec2 {
        let margin = self.layout.margin;
        self.bounds = Rect::new(
            available.x + margin,
            available.y + margin,
            available.width - margin * 2.0,
            40.0,
        );
        Vec2::new(available.width, 40.0 + margin * 2.0)
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

        // Render background/border
        renderer.draw_style_rect(self.bounds, current_style);

        // Render selection background if selected
        let text = if self.value.is_empty() {
            &self.placeholder
        } else {
            &self.value
        };

        let text_pos = Vec2::new(
            self.bounds.x + self.layout.padding,
            self.bounds.y + (self.bounds.height - self.text_style.font_size) / 2.0,
        );

        // Draw selection background
        if self.is_selected && !self.value.is_empty() {
            let selection_width = self.value.len() as f32 * 8.0; // Hacky monospace
            let selection_rect = Rect::new(
                text_pos.x,
                text_pos.y,
                selection_width,
                self.text_style.font_size,
            );
            renderer.fill_rect(selection_rect, Color::new(0.2, 0.5, 0.9, 0.5));
        }

        // Render Text
        let mut text_color = current_style.text_color;
        if self.value.is_empty() {
            text_color.a *= 0.5;
        }

        renderer.draw_text(
            text,
            text_pos,
            TextStyle {
                font_size: self.text_style.font_size,
                color: text_color,
                ..self.text_style.clone()
            },
        );

        // Draw cursor if focused and not selected
        if self.is_focused && !self.is_selected {
            // Calculate cursor position (simple monospace approximation)
            let base_width = self.value.len() as f32 * 8.0;
            let cursor_x = text_pos.x + base_width;

            // Render IME pre-edit text if active
            if !self.ime_preedit.is_empty() {
                renderer.draw_text(
                    &self.ime_preedit,
                    Vec2::new(cursor_x, text_pos.y),
                    TextStyle {
                        font_size: self.text_style.font_size,
                        color: Color::new(1.0, 1.0, 0.5, 1.0), // Yellow for pre-edit
                        ..self.text_style.clone()
                    },
                );

                // Draw underline for pre-edit
                let preedit_width = self.ime_preedit.len() as f32 * 8.0;
                renderer.fill_rect(
                    Rect::new(
                        cursor_x,
                        text_pos.y + self.text_style.font_size,
                        preedit_width,
                        1.0,
                    ),
                    Color::new(1.0, 1.0, 0.5, 1.0),
                );
            }

            // Draw I-beam cursor
            // If pre-edit is active, cursor is usually at end of pre-edit or specific pos.
            // For simplicity, we put it at the end of pre-edit + value.
            let total_width = base_width + (self.ime_preedit.len() as f32 * 8.0);
            let final_cursor_x = text_pos.x + total_width;

            let cursor_rect = Rect::new(final_cursor_x, text_pos.y, 2.0, self.text_style.font_size);
            renderer.fill_rect(cursor_rect, current_style.text_color);

            //
            // We can calculate cursor position in `on_event`?
            // `text_pos` depends on `bounds`.
            // `bounds` is updated in `layout`.
            // So in `on_event`, we have valid `bounds`.
            // We can recalculate `cursor_x` there and call `set_ime_position`.
        }
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
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
                // Change cursor to text beam
                ctx.set_cursor_icon(CursorIcon::Text);
                true
            }
            OxidXEvent::MouseLeave => {
                self.is_hovered = false;
                // Reset cursor to default
                ctx.set_cursor_icon(CursorIcon::Default);
                true
            }
            OxidXEvent::FocusGained => {
                self.is_focused = true;
                true
            }
            OxidXEvent::FocusLost => {
                self.is_focused = false;
                self.is_selected = false;
                true
            }
            OxidXEvent::MouseDown { .. }
            | OxidXEvent::MouseUp { .. }
            | OxidXEvent::Click { .. } => {
                // Clear selection on click
                self.is_selected = false;
                true
            }
            OxidXEvent::ImePreedit { text, .. } => {
                self.ime_preedit = text.clone();
                // We should update IME position here too if it changes
                self.update_ime_position(ctx);
                true
            }
            OxidXEvent::ImeCommit(text) => {
                // Commit text
                if self.is_selected {
                    self.value.clear();
                    self.is_selected = false;
                }
                self.value.push_str(text);
                self.ime_preedit.clear(); // Clear preedit
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
            OxidXEvent::CharInput { character } => {
                if !character.is_control() {
                    // If text is selected, replace it
                    if self.is_selected {
                        self.value.clear();
                        self.is_selected = false;
                    }
                    self.value.push(*character);
                }
            }
            OxidXEvent::KeyDown { key, modifiers } => {
                // Handle Ctrl+A (Select All)
                if modifiers.ctrl && *key == KeyCode::KEY_A {
                    self.is_selected = !self.value.is_empty();
                    return;
                }

                // Handle Ctrl+C (Copy)
                if modifiers.ctrl && *key == KeyCode::KEY_C {
                    if self.is_selected || !self.value.is_empty() {
                        ctx.copy_to_clipboard(&self.value);
                    }
                    return;
                }

                // Handle Ctrl+V (Paste)
                if modifiers.ctrl && *key == KeyCode::KEY_V {
                    if let Some(text) = ctx.paste_from_clipboard() {
                        if self.is_selected {
                            self.value.clear();
                            self.is_selected = false;
                        }
                        self.value.push_str(&text);
                    }
                    return;
                }

                // Handle Backspace
                if *key == KeyCode::BACKSPACE {
                    if self.is_selected {
                        self.value.clear();
                        self.is_selected = false;
                    } else {
                        self.value.pop();
                    }
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
