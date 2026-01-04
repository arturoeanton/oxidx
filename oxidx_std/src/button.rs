//! Button Component
//!
//! A professional button with:
//! - Hover, press, and disabled states
//! - Interactive styling with theme support
//! - Click callbacks
//! - Keyboard activation (Enter/Space when focused)
//! - Icon support (optional)
//! - Loading state with animation
//!
//!

use glam::Vec2;
use oxidx_core::component::OxidXComponent;
use oxidx_core::events::{KeyCode, MouseButton, OxidXEvent};
use oxidx_core::primitives::{Color, Rect, TextStyle};
use oxidx_core::renderer::Renderer;
use oxidx_core::style::{ComponentState, InteractiveStyle};
use oxidx_core::theme::Theme;
use oxidx_core::OxidXContext;
use std::cell::Cell;

/// Button variant for quick styling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Danger,
    Ghost,
}

/// A clickable button component with full interaction support.
pub struct Button {
    // === Layout ===
    bounds: Rect,
    preferred_size: Vec2,

    // === Content ===
    label: Option<String>,
    icon: Option<String>, // Icon character or emoji

    // === Styling ===
    style: InteractiveStyle,
    text_style: TextStyle,
    variant: ButtonVariant,

    // === State ===
    id: String,
    is_hovered: bool,
    is_pressed: bool,
    is_disabled: bool,
    is_focused: bool,

    // === Loading State ===
    is_loading: bool,
    loading_rotation: f32,

    // === Callbacks ===
    on_click: Option<Box<dyn Fn(&mut OxidXContext) + Send>>,

    // === Animation ===
    /// Press animation progress (0.0 to 1.0)
    press_animation: f32,
    /// Cached center for ripple effect
    press_origin: Cell<Vec2>,

    // === Focus Order ===
    /// Tab navigation order (lower values are focused first)
    focus_order: usize,
}

impl Button {
    /// Creates a new button with default styling.
    pub fn new() -> Self {
        Self {
            bounds: Rect::default(),
            preferred_size: Vec2::new(120.0, 48.0),
            label: None,
            icon: None,
            style: Theme::dark().primary_button,
            text_style: TextStyle::new(16.0).with_color(Color::WHITE),
            variant: ButtonVariant::Primary,
            id: String::new(),
            is_hovered: false,
            is_pressed: false,
            is_disabled: false,
            is_focused: false,
            is_loading: false,
            loading_rotation: 0.0,
            on_click: None,
            press_animation: 0.0,
            press_origin: Cell::new(Vec2::ZERO),
            focus_order: usize::MAX, // Default to very high (last in tab order)
        }
    }

    // === Builder Methods ===

    /// Creates a button with specific bounds (legacy API).
    pub fn with_bounds(x: f32, y: f32, width: f32, height: f32) -> Self {
        let mut btn = Self::new();
        btn.bounds = Rect::new(x, y, width, height);
        btn.preferred_size = Vec2::new(width, height);
        btn
    }

    /// Creates a button with label at specific bounds (legacy API).
    pub fn with_label(x: f32, y: f32, width: f32, height: f32, label: impl Into<String>) -> Self {
        Self::with_bounds(x, y, width, height).label(label)
    }

    /// Sets the button label text.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets an icon (emoji or icon font character).
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Sets the preferred size.
    pub fn preferred_size(mut self, size: Vec2) -> Self {
        self.preferred_size = size;
        self
    }

    /// Sets the width (keeps current height).
    pub fn width(mut self, width: f32) -> Self {
        self.preferred_size.x = width;
        self
    }

    /// Sets the height (keeps current width).
    pub fn height(mut self, height: f32) -> Self {
        self.preferred_size.y = height;
        self
    }

    /// Sets the component ID.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Sets the interactive style.
    pub fn style(mut self, style: InteractiveStyle) -> Self {
        self.style = style;
        self
    }

    /// Sets the button variant (applies predefined styling).
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self.style = Self::style_for_variant(variant);
        self
    }

    /// Sets the disabled state.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.is_disabled = disabled;
        self
    }

    /// Sets the loading state.
    pub fn loading(mut self, loading: bool) -> Self {
        self.is_loading = loading;
        self
    }

    /// Sets the click callback.
    pub fn on_click(mut self, callback: impl Fn(&mut OxidXContext) + Send + 'static) -> Self {
        self.on_click = Some(Box::new(callback));
        self
    }

    /// Sets focus order for Tab navigation (lower values are focused first).
    pub fn with_focus_order(mut self, order: usize) -> Self {
        self.focus_order = order;
        self
    }

    // === Mutable Setters ===

    /// Sets the label (mutable reference).
    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = Some(label.into());
    }

    /// Sets the icon (mutable reference).
    pub fn set_icon(&mut self, icon: impl Into<String>) {
        self.icon = Some(icon.into());
    }

    /// Sets disabled state (mutable reference).
    pub fn set_disabled(&mut self, disabled: bool) {
        self.is_disabled = disabled;
    }

    /// Sets loading state (mutable reference).
    pub fn set_loading(&mut self, loading: bool) {
        self.is_loading = loading;
    }

    /// Sets the style (mutable reference).
    pub fn set_style(&mut self, style: InteractiveStyle) {
        self.style = style;
    }

    // === Helper Methods ===

    /// Returns the style for a given variant.
    fn style_for_variant(variant: ButtonVariant) -> InteractiveStyle {
        let theme = Theme::dark();
        match variant {
            ButtonVariant::Primary => theme.primary_button,
            ButtonVariant::Secondary => theme.secondary_button,
            ButtonVariant::Danger => Self::danger_style(),
            ButtonVariant::Ghost => Self::ghost_style(),
        }
    }

    /// Creates a danger button style.
    fn danger_style() -> InteractiveStyle {
        use oxidx_core::style::Style;

        let danger_color = Color::new(0.9, 0.2, 0.2, 1.0);
        let danger_hover = Color::new(1.0, 0.3, 0.3, 1.0);
        let danger_pressed = Color::new(0.7, 0.15, 0.15, 1.0);

        InteractiveStyle {
            idle: Style::new()
                .bg_solid(danger_color)
                .rounded(4.0)
                .text_color(Color::WHITE),
            hover: Style::new()
                .bg_solid(danger_hover)
                .rounded(4.0)
                .text_color(Color::WHITE),
            pressed: Style::new()
                .bg_solid(danger_pressed)
                .rounded(4.0)
                .text_color(Color::WHITE),
            disabled: Style::new()
                .bg_solid(Color::new(0.3, 0.15, 0.15, 1.0))
                .rounded(4.0)
                .text_color(Color::new(0.6, 0.4, 0.4, 1.0)),
        }
    }

    /// Creates a ghost button style (transparent background).
    fn ghost_style() -> InteractiveStyle {
        use oxidx_core::style::Style;

        InteractiveStyle {
            idle: Style::new()
                .bg_solid(Color::TRANSPARENT)
                .rounded(4.0)
                .text_color(Color::WHITE),
            hover: Style::new()
                .bg_solid(Color::new(1.0, 1.0, 1.0, 0.1))
                .rounded(4.0)
                .text_color(Color::WHITE),
            pressed: Style::new()
                .bg_solid(Color::new(1.0, 1.0, 1.0, 0.2))
                .rounded(4.0)
                .text_color(Color::WHITE),
            disabled: Style::new()
                .bg_solid(Color::TRANSPARENT)
                .rounded(4.0)
                .text_color(Color::new(0.5, 0.5, 0.5, 1.0)),
        }
    }

    /// Returns the current component state.
    fn current_state(&self) -> ComponentState {
        if self.is_disabled {
            ComponentState::Disabled
        } else if self.is_pressed {
            ComponentState::Pressed
        } else if self.is_hovered || self.is_focused {
            ComponentState::Hover
        } else {
            ComponentState::Idle
        }
    }

    /// Triggers the click action.
    fn trigger_click(&self, ctx: &mut OxidXContext) {
        if !self.is_disabled && !self.is_loading {
            if let Some(ref callback) = self.on_click {
                callback(ctx);
            }
        }
    }

    /// Draws a loading spinner.
    fn draw_loading_spinner(&self, renderer: &mut Renderer, center: Vec2, radius: f32) {
        let segments = 8;
        let segment_angle = std::f32::consts::TAU / segments as f32;

        for i in 0..segments {
            let angle = self.loading_rotation + (i as f32 * segment_angle);
            let alpha = (i as f32 / segments as f32) * 0.8 + 0.2;

            let x = center.x + angle.cos() * radius;
            let y = center.y + angle.sin() * radius;

            let dot_size = 3.0;
            renderer.fill_rect(
                Rect::new(x - dot_size / 2.0, y - dot_size / 2.0, dot_size, dot_size),
                Color::new(1.0, 1.0, 1.0, alpha),
            );
        }
    }
}

impl Default for Button {
    fn default() -> Self {
        Self::new()
    }
}

impl OxidXComponent for Button {
    fn update(&mut self, dt: f32) {
        // Loading spinner animation
        if self.is_loading {
            self.loading_rotation += dt * 5.0; // ~5 rad/s
            if self.loading_rotation > std::f32::consts::TAU {
                self.loading_rotation -= std::f32::consts::TAU;
            }
        }

        // Press animation (subtle scale effect could be added here)
        if self.is_pressed {
            self.press_animation = (self.press_animation + dt * 8.0).min(1.0);
        } else {
            self.press_animation = (self.press_animation - dt * 4.0).max(0.0);
        }
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = Rect::new(
            available.x,
            available.y,
            self.preferred_size.x.min(available.width),
            self.preferred_size.y.min(available.height),
        );
        Vec2::new(self.bounds.width, self.bounds.height)
    }

    fn render(&self, renderer: &mut Renderer) {
        let state = self.current_state();
        let current_style = self.style.resolve(state);

        // 1. Draw background/border/shadow
        renderer.draw_style_rect(self.bounds, current_style);

        // 2. Draw focus ring if focused
        if self.is_focused && !self.is_disabled {
            let focus_rect = Rect::new(
                self.bounds.x - 2.0,
                self.bounds.y - 2.0,
                self.bounds.width + 4.0,
                self.bounds.height + 4.0,
            );
            renderer.stroke_rect(focus_rect, Color::new(0.4, 0.6, 1.0, 0.8), 2.0);
        }

        let center = self.bounds.center();

        // 3. Draw loading spinner OR content
        if self.is_loading {
            self.draw_loading_spinner(renderer, center, 10.0);
        } else {
            // Calculate content layout
            let has_icon = self.icon.is_some();
            let has_label = self.label.is_some();
            let icon_size = self.text_style.font_size;
            let gap = 8.0;

            // Measure total content width
            let icon_width = if has_icon { icon_size } else { 0.0 };
            let label_width = if let Some(ref label) = self.label {
                renderer.measure_text(label, self.text_style.font_size)
            } else {
                0.0
            };
            let gap_width = if has_icon && has_label { gap } else { 0.0 };
            let total_width = icon_width + gap_width + label_width;

            // Start position (centered)
            let mut x = center.x - total_width / 2.0;
            let y = center.y - self.text_style.font_size / 2.0;

            // Draw icon
            if let Some(ref icon) = self.icon {
                renderer.draw_text(
                    icon,
                    Vec2::new(x, y),
                    TextStyle {
                        font_size: icon_size,
                        color: current_style.text_color,
                        ..self.text_style.clone()
                    },
                );
                x += icon_size + gap;
            }

            // Draw label
            if let Some(ref label) = self.label {
                renderer.draw_text(
                    label,
                    Vec2::new(x, y),
                    TextStyle {
                        font_size: self.text_style.font_size,
                        color: current_style.text_color,
                        ..self.text_style.clone()
                    },
                );
            }
        }
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        // Register as focusable for Tab navigation
        if !self.id.is_empty() {
            ctx.register_focusable(&self.id, self.focus_order);
            // Sync focus state from the singleton - engine is the source of truth
            self.is_focused = ctx.is_focused(&self.id);
        }

        if self.is_disabled {
            return false;
        }

        // Bounds check for mouse events
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
                true
            }
            OxidXEvent::MouseLeave => {
                self.is_hovered = false;
                self.is_pressed = false;
                true
            }
            OxidXEvent::MouseDown { position, .. } => {
                self.is_pressed = true;
                self.press_origin.set(*position);

                // Request focus - engine will update FocusManager
                if !self.id.is_empty() {
                    ctx.request_focus(&self.id);
                }
                true
            }
            OxidXEvent::MouseUp { .. } => {
                self.is_pressed = false;
                true
            }
            OxidXEvent::Click { button, .. } => {
                if matches!(button, MouseButton::Left) {
                    self.trigger_click(ctx);
                    true
                } else {
                    false
                }
            }
            // FocusGained/FocusLost: reset pressed state when losing focus
            OxidXEvent::FocusLost { id } if id == &self.id => {
                self.is_pressed = false;
                true
            }
            _ => false,
        }
    }

    fn on_keyboard_input(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) {
        if !ctx.is_focused(&self.id) || self.is_disabled {
            return;
        }

        match event {
            OxidXEvent::KeyDown { key, .. } => {
                // Enter or Space activates button
                if *key == KeyCode::ENTER || *key == KeyCode::SPACE {
                    self.is_pressed = true;
                }
            }
            OxidXEvent::KeyUp { key, .. } => {
                if *key == KeyCode::ENTER || *key == KeyCode::SPACE {
                    if self.is_pressed {
                        self.is_pressed = false;
                        self.trigger_click(ctx);
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
        self.bounds.x = x;
        self.bounds.y = y;
    }

    fn set_size(&mut self, width: f32, height: f32) {
        self.bounds.width = width;
        self.bounds.height = height;
        self.preferred_size = Vec2::new(width, height);
    }

    fn is_focusable(&self) -> bool {
        !self.is_disabled
    }
}
