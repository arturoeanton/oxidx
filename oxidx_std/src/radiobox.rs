//! RadioButton and RadioGroup components for OxidX
//!
//! Features:
//! - Single selection within groups
//! - Animated selection indicator
//! - Custom labels
//! - Keyboard navigation (arrow keys)
//! - Focus ring
//! - Disabled state (individual + group)
//! - Size variants
//! - Horizontal/Vertical layouts

use oxidx_core::{
    component::OxidXComponent,
    context::OxidXContext,
    events::{Event, KeyCode, MouseButton, ARROW_DOWN, ARROW_LEFT, ARROW_RIGHT, ARROW_UP, SPACE},
    layout::{Alignment, Rect},
    primitives::{Color, TextStyle},
    renderer::RenderCommand,
    style::Style,
    theme::Theme,
};
use std::sync::{Arc, RwLock};

// ============================================================================
// Types
// ============================================================================

/// Size variant for radio buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RadioSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl RadioSize {
    fn circle_size(&self) -> f32 {
        match self {
            RadioSize::Small => 14.0,
            RadioSize::Medium => 18.0,
            RadioSize::Large => 24.0,
        }
    }

    fn dot_size(&self) -> f32 {
        match self {
            RadioSize::Small => 6.0,
            RadioSize::Medium => 8.0,
            RadioSize::Large => 10.0,
        }
    }

    fn font_size(&self) -> f32 {
        match self {
            RadioSize::Small => 12.0,
            RadioSize::Medium => 14.0,
            RadioSize::Large => 16.0,
        }
    }

    fn spacing(&self) -> f32 {
        match self {
            RadioSize::Small => 6.0,
            RadioSize::Medium => 8.0,
            RadioSize::Large => 10.0,
        }
    }
}

/// Layout direction for radio group
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RadioLayout {
    #[default]
    Vertical,
    Horizontal,
}

// ============================================================================
// Shared Group State
// ============================================================================

/// Shared state for a radio group
#[derive(Debug)]
pub struct RadioGroupState {
    /// Currently selected value
    selected: Option<String>,
    /// All option values in order
    options: Vec<String>,
    /// Group name/id
    name: String,
}

impl RadioGroupState {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            selected: None,
            options: Vec::new(),
            name: name.into(),
        }
    }

    pub fn select(&mut self, value: &str) {
        if self.options.contains(&value.to_string()) {
            self.selected = Some(value.to_string());
        }
    }

    pub fn selected(&self) -> Option<&str> {
        self.selected.as_deref()
    }

    pub fn is_selected(&self, value: &str) -> bool {
        self.selected.as_deref() == Some(value)
    }

    fn register(&mut self, value: String) {
        if !self.options.contains(&value) {
            self.options.push(value);
        }
    }

    fn next(&self, current: &str) -> Option<&str> {
        let idx = self.options.iter().position(|v| v == current)?;
        let next_idx = (idx + 1) % self.options.len();
        Some(&self.options[next_idx])
    }

    fn prev(&self, current: &str) -> Option<&str> {
        let idx = self.options.iter().position(|v| v == current)?;
        let prev_idx = if idx == 0 {
            self.options.len() - 1
        } else {
            idx - 1
        };
        Some(&self.options[prev_idx])
    }
}

pub type SharedGroupState = Arc<RwLock<RadioGroupState>>;

// ============================================================================
// Animation State
// ============================================================================

#[derive(Debug, Clone)]
struct AnimationState {
    /// Dot scale (0.0 to 1.0)
    dot_scale: f32,
    /// Target scale
    target_scale: f32,
    /// Animation speed
    animation_speed: f32,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            dot_scale: 0.0,
            target_scale: 0.0,
            animation_speed: 10.0,
        }
    }
}

impl AnimationState {
    fn update(&mut self, dt: f32) -> bool {
        if (self.dot_scale - self.target_scale).abs() < 0.001 {
            self.dot_scale = self.target_scale;
            return false;
        }

        let direction = if self.target_scale > self.dot_scale {
            1.0
        } else {
            -1.0
        };

        self.dot_scale += direction * self.animation_speed * dt;
        self.dot_scale = self.dot_scale.clamp(0.0, 1.0);
        true
    }

    fn set_selected(&mut self, selected: bool) {
        self.target_scale = if selected { 1.0 } else { 0.0 };
    }
}

// ============================================================================
// RadioButton Component
// ============================================================================

/// A single radio button (usually used within a RadioGroup)
///
/// # Example
/// ```rust
/// let group_state = Arc::new(RwLock::new(RadioGroupState::new("gender")));
///
/// let radio = RadioButton::new("male", "Male")
///     .group(group_state.clone())
///     .size(RadioSize::Medium);
/// ```
pub struct RadioButton {
    // Identity
    id: String,
    value: String,
    label: String,

    // Group
    group: Option<SharedGroupState>,

    // Visual
    size: RadioSize,

    // Colors (None = use theme)
    dot_color: Option<Color>,
    ring_color: Option<Color>,
    label_color: Option<Color>,

    // State
    disabled: bool,
    hovered: bool,
    pressed: bool,
    focused: bool,

    // Animation
    animation: AnimationState,

    // Callback (for standalone use)
    on_select: Option<Box<dyn Fn(&str) + Send + Sync>>,

    // Layout
    bounds: Rect,
    circle_rect: Rect,
    label_rect: Rect,
}

impl RadioButton {
    /// Create a new radio button with value and label
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        let value = value.into();
        let id = format!("radio_{}", value);

        Self {
            id,
            value,
            label: label.into(),
            group: None,
            size: RadioSize::Medium,
            dot_color: None,
            ring_color: None,
            label_color: None,
            disabled: false,
            hovered: false,
            pressed: false,
            focused: false,
            animation: AnimationState::default(),
            on_select: None,
            bounds: Rect::ZERO,
            circle_rect: Rect::ZERO,
            label_rect: Rect::ZERO,
        }
    }

    // ========================================================================
    // Builder Methods
    // ========================================================================

    /// Associate with a radio group
    pub fn group(mut self, group: SharedGroupState) -> Self {
        // Register this option with the group
        if let Ok(mut state) = group.write() {
            state.register(self.value.clone());
        }
        self.group = Some(group);
        self
    }

    /// Set size variant
    pub fn size(mut self, size: RadioSize) -> Self {
        self.size = size;
        self
    }

    /// Set disabled state
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set custom dot color
    pub fn dot_color(mut self, color: Color) -> Self {
        self.dot_color = Some(color);
        self
    }

    /// Set custom ring color
    pub fn ring_color(mut self, color: Color) -> Self {
        self.ring_color = Some(color);
        self
    }

    /// Set custom label color
    pub fn label_color(mut self, color: Color) -> Self {
        self.label_color = Some(color);
        self
    }

    /// Set callback for standalone use
    pub fn on_select<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_select = Some(Box::new(callback));
        self
    }

    // ========================================================================
    // State Methods
    // ========================================================================

    /// Check if this radio is selected
    pub fn is_selected(&self) -> bool {
        if let Some(ref group) = self.group {
            if let Ok(state) = group.read() {
                return state.is_selected(&self.value);
            }
        }
        false
    }

    /// Select this radio button
    pub fn select(&mut self) {
        if self.disabled {
            return;
        }

        if let Some(ref group) = self.group {
            if let Ok(mut state) = group.write() {
                state.select(&self.value);
            }
        }

        self.animation.set_selected(true);

        if let Some(ref callback) = self.on_select {
            callback(&self.value);
        }
    }

    /// Navigate to next option in group
    fn select_next(&mut self) {
        if let Some(ref group) = self.group {
            if let Ok(state) = group.read() {
                if let Some(next) = state.next(&self.value) {
                    let next = next.to_string();
                    drop(state);
                    if let Ok(mut state) = group.write() {
                        state.select(&next);
                    }
                }
            }
        }
    }

    /// Navigate to previous option in group
    fn select_prev(&mut self) {
        if let Some(ref group) = self.group {
            if let Ok(state) = group.read() {
                if let Some(prev) = state.prev(&self.value) {
                    let prev = prev.to_string();
                    drop(state);
                    if let Ok(mut state) = group.write() {
                        state.select(&prev);
                    }
                }
            }
        }
    }

    // ========================================================================
    // Layout
    // ========================================================================

    fn compute_layout(&mut self) {
        let circle_size = self.size.circle_size();

        self.circle_rect = Rect {
            x: self.bounds.x,
            y: self.bounds.y + (self.bounds.height - circle_size) / 2.0,
            width: circle_size,
            height: circle_size,
        };

        let spacing = self.size.spacing();
        self.label_rect = Rect {
            x: self.circle_rect.x + circle_size + spacing,
            y: self.bounds.y,
            width: self.bounds.width - circle_size - spacing,
            height: self.bounds.height,
        };
    }

    fn hit_test(&self, x: f32, y: f32) -> bool {
        self.bounds.contains(x, y)
    }

    // ========================================================================
    // Rendering
    // ========================================================================

    fn get_colors(&self, theme: &Theme) -> (Color, Color, Color) {
        let is_selected = self.is_selected();

        if self.disabled {
            (
                theme.disabled_border,
                theme.disabled_text,
                theme.disabled_text,
            )
        } else if is_selected {
            (
                self.ring_color.unwrap_or(theme.primary),
                self.dot_color.unwrap_or(theme.primary),
                self.label_color.unwrap_or(theme.text),
            )
        } else if self.hovered {
            (
                theme.border_hover,
                self.dot_color.unwrap_or(theme.primary),
                self.label_color.unwrap_or(theme.text),
            )
        } else {
            (
                theme.border,
                self.dot_color.unwrap_or(theme.primary),
                self.label_color.unwrap_or(theme.text),
            )
        }
    }
}

// ============================================================================
// OxidXComponent Implementation for RadioButton
// ============================================================================

impl OxidXComponent for RadioButton {
    fn id(&self) -> &str {
        &self.id
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
        self.compute_layout();
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn preferred_size(&self, ctx: &OxidXContext) -> (f32, f32) {
        let circle_size = self.size.circle_size();
        let font_size = self.size.font_size();
        let text_width = ctx.measure_text(&self.label, font_size).0;
        let spacing = self.size.spacing();

        (
            circle_size + spacing + text_width,
            circle_size.max(font_size * 1.4),
        )
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut OxidXContext) -> bool {
        if self.disabled {
            return false;
        }

        match event {
            Event::MouseMove { x, y } => {
                let was_hovered = self.hovered;
                self.hovered = self.hit_test(*x, *y);

                if self.hovered {
                    ctx.set_cursor_icon(winit::window::CursorIcon::Pointer);
                }

                was_hovered != self.hovered
            }

            Event::MouseButton {
                button: MouseButton::Left,
                pressed,
                x,
                y,
            } => {
                if self.hit_test(*x, *y) {
                    if *pressed {
                        self.pressed = true;
                        ctx.request_focus(&self.id);
                        true
                    } else if self.pressed {
                        self.pressed = false;
                        self.select();
                        true
                    } else {
                        false
                    }
                } else {
                    if !pressed && self.pressed {
                        self.pressed = false;
                        true
                    } else {
                        false
                    }
                }
            }

            Event::KeyDown { key, .. } => {
                if !ctx.is_focused(&self.id) {
                    return false;
                }

                match *key {
                    k if k == SPACE => {
                        self.pressed = true;
                        true
                    }
                    k if k == ARROW_DOWN || k == ARROW_RIGHT => {
                        self.select_next();
                        true
                    }
                    k if k == ARROW_UP || k == ARROW_LEFT => {
                        self.select_prev();
                        true
                    }
                    _ => false,
                }
            }

            Event::KeyUp { key, .. } => {
                if ctx.is_focused(&self.id) && *key == SPACE && self.pressed {
                    self.pressed = false;
                    self.select();
                    true
                } else {
                    false
                }
            }

            Event::FocusChanged { focused } => {
                self.focused = *focused && ctx.is_focused(&self.id);
                true
            }

            _ => false,
        }
    }

    fn update(&mut self, dt: f32, _ctx: &mut OxidXContext) -> bool {
        // Sync animation with selection state
        let is_selected = self.is_selected();
        if (is_selected && self.animation.target_scale < 1.0)
            || (!is_selected && self.animation.target_scale > 0.0)
        {
            self.animation.set_selected(is_selected);
        }

        self.animation.update(dt)
    }

    fn render(&self, ctx: &OxidXContext, commands: &mut Vec<RenderCommand>) {
        let theme = ctx.theme();
        let (ring_color, dot_color, label_color) = self.get_colors(theme);

        let circle_size = self.size.circle_size();
        let center_x = self.circle_rect.x + circle_size / 2.0;
        let center_y = self.circle_rect.y + circle_size / 2.0;
        let outer_radius = circle_size / 2.0;

        // Draw outer ring
        let stroke_width = if self.pressed { 2.0 } else { 1.5 };
        commands.push(RenderCommand::CircleStroke {
            cx: center_x,
            cy: center_y,
            radius: outer_radius - stroke_width / 2.0,
            color: ring_color,
            stroke_width,
        });

        // Draw inner dot with animation
        if self.animation.dot_scale > 0.0 {
            let dot_radius = self.size.dot_size() / 2.0 * self.animation.dot_scale;
            commands.push(RenderCommand::Circle {
                cx: center_x,
                cy: center_y,
                radius: dot_radius,
                color: dot_color,
            });
        }

        // Draw focus ring
        if self.focused {
            commands.push(RenderCommand::CircleStroke {
                cx: center_x,
                cy: center_y,
                radius: outer_radius + 2.0,
                color: theme.focus_ring,
                stroke_width: 2.0,
            });
        }

        // Draw label
        let font_size = self.size.font_size();
        let text_style = TextStyle {
            font_size,
            color: label_color,
            ..Default::default()
        };

        let text_y = self.label_rect.y + (self.label_rect.height - font_size) / 2.0;

        commands.push(RenderCommand::Text {
            text: self.label.clone(),
            x: self.label_rect.x,
            y: text_y,
            style: text_style,
            max_width: Some(self.label_rect.width),
        });
    }

    fn focusable(&self) -> bool {
        !self.disabled
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// ============================================================================
// RadioGroup Component
// ============================================================================

/// A group of radio buttons with shared state
///
/// # Example
/// ```rust
/// let group = RadioGroup::new("payment_method")
///     .label("Payment Method")
///     .options(vec![
///         ("credit", "Credit Card"),
///         ("debit", "Debit Card"),
///         ("paypal", "PayPal"),
///     ])
///     .selected("credit")
///     .layout(RadioLayout::Vertical)
///     .on_change(|value| {
///         println!("Selected: {}", value);
///     });
/// ```
pub struct RadioGroup {
    // Identity
    id: String,

    // State
    state: SharedGroupState,

    // Options
    options: Vec<(String, String)>, // (value, label)

    // Visual
    group_label: Option<String>,
    size: RadioSize,
    layout: RadioLayout,
    spacing: f32,

    // State
    disabled: bool,

    // Rendered radio buttons
    radios: Vec<RadioButton>,

    // Callback
    on_change: Option<Box<dyn Fn(&str) + Send + Sync>>,

    // Layout
    bounds: Rect,
}

impl RadioGroup {
    /// Create a new radio group
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let id = format!("radiogroup_{}", name);
        let state = Arc::new(RwLock::new(RadioGroupState::new(&name)));

        Self {
            id,
            state,
            options: Vec::new(),
            group_label: None,
            size: RadioSize::Medium,
            layout: RadioLayout::Vertical,
            spacing: 8.0,
            disabled: false,
            radios: Vec::new(),
            on_change: None,
            bounds: Rect::ZERO,
        }
    }

    // ========================================================================
    // Builder Methods
    // ========================================================================

    /// Set group label
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.group_label = Some(label.into());
        self
    }

    /// Add options as (value, label) pairs
    pub fn options(mut self, options: Vec<(&str, &str)>) -> Self {
        self.options = options
            .into_iter()
            .map(|(v, l)| (v.to_string(), l.to_string()))
            .collect();
        self.rebuild_radios();
        self
    }

    /// Set initially selected value
    pub fn selected(mut self, value: &str) -> Self {
        if let Ok(mut state) = self.state.write() {
            state.select(value);
        }
        self
    }

    /// Set size for all radio buttons
    pub fn size(mut self, size: RadioSize) -> Self {
        self.size = size;
        self.rebuild_radios();
        self
    }

    /// Set layout direction
    pub fn layout(mut self, layout: RadioLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Set spacing between options
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Set disabled state for all options
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self.rebuild_radios();
        self
    }

    /// Set change callback
    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_change = Some(Box::new(callback));
        self
    }

    // ========================================================================
    // State Methods
    // ========================================================================

    /// Get currently selected value
    pub fn selected_value(&self) -> Option<String> {
        if let Ok(state) = self.state.read() {
            state.selected().map(|s| s.to_string())
        } else {
            None
        }
    }

    /// Select a value programmatically
    pub fn select_value(&mut self, value: &str) {
        if let Ok(mut state) = self.state.write() {
            state.select(value);
        }
    }

    // ========================================================================
    // Internal
    // ========================================================================

    fn rebuild_radios(&mut self) {
        self.radios = self
            .options
            .iter()
            .map(|(value, label)| {
                RadioButton::new(value, label)
                    .group(self.state.clone())
                    .size(self.size)
                    .disabled(self.disabled)
            })
            .collect();
    }

    fn compute_layout(&mut self, ctx: &OxidXContext) {
        let mut y_offset = self.bounds.y;
        let mut x_offset = self.bounds.x;

        // Account for group label
        if let Some(ref label) = self.group_label {
            let label_height = self.size.font_size() * 1.5;
            y_offset += label_height;
        }

        for radio in &mut self.radios {
            let (pref_w, pref_h) = radio.preferred_size(ctx);

            match self.layout {
                RadioLayout::Vertical => {
                    radio.set_bounds(Rect {
                        x: x_offset,
                        y: y_offset,
                        width: self.bounds.width,
                        height: pref_h,
                    });
                    y_offset += pref_h + self.spacing;
                }
                RadioLayout::Horizontal => {
                    radio.set_bounds(Rect {
                        x: x_offset,
                        y: y_offset,
                        width: pref_w,
                        height: pref_h,
                    });
                    x_offset += pref_w + self.spacing;
                }
            }
        }
    }
}

// ============================================================================
// OxidXComponent Implementation for RadioGroup
// ============================================================================

impl OxidXComponent for RadioGroup {
    fn id(&self) -> &str {
        &self.id
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
        // Layout will be computed in render when we have context
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn preferred_size(&self, ctx: &OxidXContext) -> (f32, f32) {
        let mut total_w: f32 = 0.0;
        let mut total_h: f32 = 0.0;

        // Group label
        if let Some(ref _label) = self.group_label {
            total_h += self.size.font_size() * 1.5;
        }

        match self.layout {
            RadioLayout::Vertical => {
                for radio in &self.radios {
                    let (w, h) = radio.preferred_size(ctx);
                    total_w = total_w.max(w);
                    total_h += h + self.spacing;
                }
                if !self.radios.is_empty() {
                    total_h -= self.spacing; // Remove last spacing
                }
            }
            RadioLayout::Horizontal => {
                for radio in &self.radios {
                    let (w, h) = radio.preferred_size(ctx);
                    total_w += w + self.spacing;
                    total_h = total_h.max(h);
                }
                if !self.radios.is_empty() {
                    total_w -= self.spacing;
                }
            }
        }

        (total_w, total_h)
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut OxidXContext) -> bool {
        let prev_selected = self.selected_value();

        // Dispatch to child radios
        let mut handled = false;
        for radio in &mut self.radios {
            if radio.handle_event(event, ctx) {
                handled = true;
            }
        }

        // Check if selection changed
        let new_selected = self.selected_value();
        if prev_selected != new_selected {
            if let (Some(ref callback), Some(ref value)) = (&self.on_change, &new_selected) {
                callback(value);
            }
        }

        handled
    }

    fn update(&mut self, dt: f32, ctx: &mut OxidXContext) -> bool {
        self.compute_layout(ctx);

        let mut needs_redraw = false;
        for radio in &mut self.radios {
            if radio.update(dt, ctx) {
                needs_redraw = true;
            }
        }
        needs_redraw
    }

    fn render(&self, ctx: &OxidXContext, commands: &mut Vec<RenderCommand>) {
        let theme = ctx.theme();

        // Draw group label
        if let Some(ref label) = self.group_label {
            let font_size = self.size.font_size();
            let text_style = TextStyle {
                font_size,
                color: theme.text,
                bold: true,
                ..Default::default()
            };

            commands.push(RenderCommand::Text {
                text: label.clone(),
                x: self.bounds.x,
                y: self.bounds.y,
                style: text_style,
                max_width: Some(self.bounds.width),
            });
        }

        // Render child radios
        for radio in &self.radios {
            radio.render(ctx, commands);
        }
    }

    fn focusable(&self) -> bool {
        false // Individual radios are focusable
    }

    fn children(&self) -> Option<Vec<&dyn OxidXComponent>> {
        Some(
            self.radios
                .iter()
                .map(|r| r as &dyn OxidXComponent)
                .collect(),
        )
    }

    fn children_mut(&mut self) -> Option<Vec<&mut dyn OxidXComponent>> {
        Some(
            self.radios
                .iter_mut()
                .map(|r| r as &mut dyn OxidXComponent)
                .collect(),
        )
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_radio_group_state() {
        let mut state = RadioGroupState::new("test");
        state.register("a".to_string());
        state.register("b".to_string());
        state.register("c".to_string());

        assert_eq!(state.selected(), None);

        state.select("b");
        assert_eq!(state.selected(), Some("b"));
        assert!(state.is_selected("b"));
        assert!(!state.is_selected("a"));
    }

    #[test]
    fn test_radio_group_navigation() {
        let mut state = RadioGroupState::new("test");
        state.register("a".to_string());
        state.register("b".to_string());
        state.register("c".to_string());

        assert_eq!(state.next("a"), Some("b"));
        assert_eq!(state.next("c"), Some("a")); // Wraps
        assert_eq!(state.prev("b"), Some("a"));
        assert_eq!(state.prev("a"), Some("c")); // Wraps
    }

    #[test]
    fn test_radio_button_creation() {
        let radio = RadioButton::new("option1", "Option 1");
        assert_eq!(radio.id(), "radio_option1");
        assert!(!radio.is_selected());
    }

    #[test]
    fn test_radio_group_creation() {
        let group = RadioGroup::new("test")
            .options(vec![("a", "Option A"), ("b", "Option B")])
            .selected("a");

        assert_eq!(group.selected_value(), Some("a".to_string()));
    }

    #[test]
    fn test_shared_state() {
        let state = Arc::new(RwLock::new(RadioGroupState::new("test")));

        let _radio1 = RadioButton::new("opt1", "Option 1").group(state.clone());
        let _radio2 = RadioButton::new("opt2", "Option 2").group(state.clone());

        // Both should be registered
        if let Ok(s) = state.read() {
            assert!(s.options.contains(&"opt1".to_string()));
            assert!(s.options.contains(&"opt2".to_string()));
        }
    }
}
