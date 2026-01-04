//! Checkbox component for OxidX
//!
//! Features:
//! - Checked/unchecked/indeterminate states
//! - Animated check mark
//! - Custom label with click-to-toggle
//! - Keyboard support (Space to toggle)
//! - Focus ring
//! - Disabled state
//! - Size variants
//! - Custom colors

use oxidx_core::{
    component::OxidXComponent,
    context::OxidXContext,
    events::{Event, KeyCode, MouseButton, SPACE},
    layout::{Alignment, Rect},
    primitives::{Color, TextStyle},
    renderer::RenderCommand,
    style::Style,
    theme::Theme,
};

// ============================================================================
// Types
// ============================================================================

/// Checkbox state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CheckState {
    #[default]
    Unchecked,
    Checked,
    Indeterminate,
}

impl CheckState {
    pub fn is_checked(&self) -> bool {
        matches!(self, CheckState::Checked)
    }

    pub fn toggle(&self) -> Self {
        match self {
            CheckState::Unchecked => CheckState::Checked,
            CheckState::Checked => CheckState::Unchecked,
            CheckState::Indeterminate => CheckState::Checked,
        }
    }
}

/// Size variant for checkbox
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CheckboxSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl CheckboxSize {
    fn box_size(&self) -> f32 {
        match self {
            CheckboxSize::Small => 14.0,
            CheckboxSize::Medium => 18.0,
            CheckboxSize::Large => 24.0,
        }
    }

    fn font_size(&self) -> f32 {
        match self {
            CheckboxSize::Small => 12.0,
            CheckboxSize::Medium => 14.0,
            CheckboxSize::Large => 16.0,
        }
    }

    fn check_stroke(&self) -> f32 {
        match self {
            CheckboxSize::Small => 1.5,
            CheckboxSize::Medium => 2.0,
            CheckboxSize::Large => 2.5,
        }
    }

    fn spacing(&self) -> f32 {
        match self {
            CheckboxSize::Small => 6.0,
            CheckboxSize::Medium => 8.0,
            CheckboxSize::Large => 10.0,
        }
    }
}

// ============================================================================
// Animation State
// ============================================================================

#[derive(Debug, Clone)]
struct AnimationState {
    /// Check mark animation progress (0.0 to 1.0)
    check_progress: f32,
    /// Target progress
    target_progress: f32,
    /// Animation speed (progress per second)
    animation_speed: f32,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            check_progress: 0.0,
            target_progress: 0.0,
            animation_speed: 8.0, // Fast, snappy animation
        }
    }
}

impl AnimationState {
    fn update(&mut self, dt: f32) -> bool {
        if (self.check_progress - self.target_progress).abs() < 0.001 {
            self.check_progress = self.target_progress;
            return false;
        }

        let direction = if self.target_progress > self.check_progress {
            1.0
        } else {
            -1.0
        };

        self.check_progress += direction * self.animation_speed * dt;
        self.check_progress = self.check_progress.clamp(0.0, 1.0);
        true
    }

    fn set_checked(&mut self, checked: bool) {
        self.target_progress = if checked { 1.0 } else { 0.0 };
    }
}

// ============================================================================
// Checkbox Component
// ============================================================================

/// A checkbox component with optional label
///
/// # Example
/// ```rust
/// let checkbox = Checkbox::new("accept_terms")
///     .label("I accept the terms and conditions")
///     .checked(false)
///     .on_change(|checked| {
///         println!("Terms accepted: {}", checked);
///     });
/// ```
pub struct Checkbox {
    // Identity
    id: String,

    // State
    state: CheckState,
    disabled: bool,

    // Visual
    label: Option<String>,
    size: CheckboxSize,

    // Colors (None = use theme)
    check_color: Option<Color>,
    box_color: Option<Color>,
    label_color: Option<Color>,

    // Interaction state
    hovered: bool,
    pressed: bool,
    focused: bool,

    // Animation
    animation: AnimationState,

    // Callback
    on_change: Option<Box<dyn Fn(bool) + Send + Sync>>,

    // Layout cache
    bounds: Rect,
    box_rect: Rect,
    label_rect: Rect,
}

impl Checkbox {
    /// Create a new checkbox with the given ID
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            state: CheckState::Unchecked,
            disabled: false,
            label: None,
            size: CheckboxSize::Medium,
            check_color: None,
            box_color: None,
            label_color: None,
            hovered: false,
            pressed: false,
            focused: false,
            animation: AnimationState::default(),
            on_change: None,
            bounds: Rect::ZERO,
            box_rect: Rect::ZERO,
            label_rect: Rect::ZERO,
        }
    }

    // ========================================================================
    // Builder Methods
    // ========================================================================

    /// Set the label text
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set initial checked state
    pub fn checked(mut self, checked: bool) -> Self {
        self.state = if checked {
            CheckState::Checked
        } else {
            CheckState::Unchecked
        };
        self.animation.check_progress = if checked { 1.0 } else { 0.0 };
        self.animation.target_progress = self.animation.check_progress;
        self
    }

    /// Set indeterminate state
    pub fn indeterminate(mut self) -> Self {
        self.state = CheckState::Indeterminate;
        self.animation.check_progress = 1.0;
        self.animation.target_progress = 1.0;
        self
    }

    /// Set disabled state
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set size variant
    pub fn size(mut self, size: CheckboxSize) -> Self {
        self.size = size;
        self
    }

    /// Set custom check color
    pub fn check_color(mut self, color: Color) -> Self {
        self.check_color = Some(color);
        self
    }

    /// Set custom box color
    pub fn box_color(mut self, color: Color) -> Self {
        self.box_color = Some(color);
        self
    }

    /// Set custom label color
    pub fn label_color(mut self, color: Color) -> Self {
        self.label_color = Some(color);
        self
    }

    /// Set the change callback
    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        self.on_change = Some(Box::new(callback));
        self
    }

    // ========================================================================
    // State Methods
    // ========================================================================

    /// Get current state
    pub fn get_state(&self) -> CheckState {
        self.state
    }

    /// Check if checkbox is checked
    pub fn is_checked(&self) -> bool {
        self.state.is_checked()
    }

    /// Set checked state programmatically
    pub fn set_checked(&mut self, checked: bool) {
        let new_state = if checked {
            CheckState::Checked
        } else {
            CheckState::Unchecked
        };

        if self.state != new_state {
            self.state = new_state;
            self.animation.set_checked(checked);
        }
    }

    /// Toggle the checkbox
    pub fn toggle(&mut self) {
        if self.disabled {
            return;
        }

        self.state = self.state.toggle();
        self.animation.set_checked(self.state.is_checked());

        if let Some(ref callback) = self.on_change {
            callback(self.state.is_checked());
        }
    }

    // ========================================================================
    // Layout
    // ========================================================================

    fn compute_layout(&mut self, ctx: &OxidXContext) {
        let box_size = self.size.box_size();

        // Checkbox box positioned at left, vertically centered
        self.box_rect = Rect {
            x: self.bounds.x,
            y: self.bounds.y + (self.bounds.height - box_size) / 2.0,
            width: box_size,
            height: box_size,
        };

        // Label positioned to the right of box
        if self.label.is_some() {
            let spacing = self.size.spacing();
            self.label_rect = Rect {
                x: self.box_rect.x + box_size + spacing,
                y: self.bounds.y,
                width: self.bounds.width - box_size - spacing,
                height: self.bounds.height,
            };
        }
    }

    fn hit_test(&self, x: f32, y: f32) -> bool {
        // Check if point is inside bounds (box + label)
        self.bounds.contains(x, y)
    }

    // ========================================================================
    // Rendering
    // ========================================================================

    fn get_colors(&self, theme: &Theme) -> (Color, Color, Color, Color) {
        let (bg, border, check, label) = if self.disabled {
            (
                theme.disabled_background,
                theme.disabled_border,
                theme.disabled_text,
                theme.disabled_text,
            )
        } else if self.state.is_checked() || self.state == CheckState::Indeterminate {
            (
                self.box_color.unwrap_or(theme.primary),
                self.box_color.unwrap_or(theme.primary),
                self.check_color.unwrap_or(theme.on_primary),
                self.label_color.unwrap_or(theme.text),
            )
        } else if self.hovered {
            (
                theme.surface_hover,
                theme.border_hover,
                self.check_color.unwrap_or(theme.on_primary),
                self.label_color.unwrap_or(theme.text),
            )
        } else {
            (
                theme.surface,
                theme.border,
                self.check_color.unwrap_or(theme.on_primary),
                self.label_color.unwrap_or(theme.text),
            )
        };

        (bg, border, check, label)
    }
}

// ============================================================================
// OxidXComponent Implementation
// ============================================================================

impl OxidXComponent for Checkbox {
    fn id(&self) -> &str {
        &self.id
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn preferred_size(&self, ctx: &OxidXContext) -> (f32, f32) {
        let box_size = self.size.box_size();

        if let Some(ref label) = self.label {
            let font_size = self.size.font_size();
            let text_width = ctx.measure_text(label, font_size).0;
            let spacing = self.size.spacing();
            (
                box_size + spacing + text_width,
                box_size.max(font_size * 1.4),
            )
        } else {
            (box_size, box_size)
        }
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
                        self.toggle();
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
                if ctx.is_focused(&self.id) && *key == SPACE {
                    self.pressed = true;
                    true
                } else {
                    false
                }
            }

            Event::KeyUp { key, .. } => {
                if ctx.is_focused(&self.id) && *key == SPACE && self.pressed {
                    self.pressed = false;
                    self.toggle();
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
        self.animation.update(dt)
    }

    fn render(&self, ctx: &OxidXContext, commands: &mut Vec<RenderCommand>) {
        let theme = ctx.theme();
        let (bg_color, border_color, check_color, label_color) = self.get_colors(theme);

        let box_size = self.size.box_size();
        let corner_radius = 3.0;
        let border_width = if self.pressed { 2.0 } else { 1.5 };

        // Draw box background
        commands.push(RenderCommand::RoundedRect {
            rect: self.box_rect,
            color: bg_color,
            corner_radius,
        });

        // Draw box border
        commands.push(RenderCommand::RoundedRectStroke {
            rect: self.box_rect,
            color: border_color,
            corner_radius,
            stroke_width: border_width,
        });

        // Draw check mark or indeterminate line
        if self.animation.check_progress > 0.0 {
            let progress = self.animation.check_progress;
            let stroke = self.size.check_stroke();
            let padding = box_size * 0.2;

            match self.state {
                CheckState::Checked => {
                    // Draw animated checkmark
                    // Checkmark is two lines: short diagonal down-right, then long diagonal up-right
                    let start_x = self.box_rect.x + padding;
                    let start_y = self.box_rect.y + box_size * 0.5;

                    let mid_x = self.box_rect.x + box_size * 0.4;
                    let mid_y = self.box_rect.y + box_size - padding;

                    let end_x = self.box_rect.x + box_size - padding;
                    let end_y = self.box_rect.y + padding;

                    // First segment (0.0 to 0.4 progress)
                    if progress > 0.0 {
                        let seg1_progress = (progress / 0.4).min(1.0);
                        let seg1_end_x = start_x + (mid_x - start_x) * seg1_progress;
                        let seg1_end_y = start_y + (mid_y - start_y) * seg1_progress;

                        commands.push(RenderCommand::Line {
                            x1: start_x,
                            y1: start_y,
                            x2: seg1_end_x,
                            y2: seg1_end_y,
                            color: check_color,
                            width: stroke,
                        });
                    }

                    // Second segment (0.4 to 1.0 progress)
                    if progress > 0.4 {
                        let seg2_progress = ((progress - 0.4) / 0.6).min(1.0);
                        let seg2_end_x = mid_x + (end_x - mid_x) * seg2_progress;
                        let seg2_end_y = mid_y + (end_y - mid_y) * seg2_progress;

                        commands.push(RenderCommand::Line {
                            x1: mid_x,
                            y1: mid_y,
                            x2: seg2_end_x,
                            y2: seg2_end_y,
                            color: check_color,
                            width: stroke,
                        });
                    }
                }

                CheckState::Indeterminate => {
                    // Draw horizontal line
                    let line_y = self.box_rect.y + box_size / 2.0;
                    let line_start = self.box_rect.x + padding;
                    let line_end = self.box_rect.x + box_size - padding;
                    let animated_end = line_start + (line_end - line_start) * progress;

                    commands.push(RenderCommand::Line {
                        x1: line_start,
                        y1: line_y,
                        x2: animated_end,
                        y2: line_y,
                        color: check_color,
                        width: stroke,
                    });
                }

                CheckState::Unchecked => {}
            }
        }

        // Draw focus ring
        if self.focused {
            let focus_rect = Rect {
                x: self.box_rect.x - 2.0,
                y: self.box_rect.y - 2.0,
                width: self.box_rect.width + 4.0,
                height: self.box_rect.height + 4.0,
            };
            commands.push(RenderCommand::RoundedRectStroke {
                rect: focus_rect,
                color: theme.focus_ring,
                corner_radius: corner_radius + 2.0,
                stroke_width: 2.0,
            });
        }

        // Draw label
        if let Some(ref label) = self.label {
            let font_size = self.size.font_size();
            let text_style = TextStyle {
                font_size,
                color: label_color,
                ..Default::default()
            };

            // Vertically center label
            let text_height = font_size;
            let text_y = self.label_rect.y + (self.label_rect.height - text_height) / 2.0;

            commands.push(RenderCommand::Text {
                text: label.clone(),
                x: self.label_rect.x,
                y: text_y,
                style: text_style,
                max_width: Some(self.label_rect.width),
            });
        }
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
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkbox_creation() {
        let checkbox = Checkbox::new("test");
        assert_eq!(checkbox.id(), "test");
        assert!(!checkbox.is_checked());
    }

    #[test]
    fn test_checkbox_builder() {
        let checkbox = Checkbox::new("test")
            .label("Test Label")
            .checked(true)
            .size(CheckboxSize::Large)
            .disabled(false);

        assert!(checkbox.is_checked());
        assert_eq!(checkbox.label, Some("Test Label".to_string()));
    }

    #[test]
    fn test_checkbox_toggle() {
        let mut checkbox = Checkbox::new("test");
        assert!(!checkbox.is_checked());

        checkbox.toggle();
        assert!(checkbox.is_checked());

        checkbox.toggle();
        assert!(!checkbox.is_checked());
    }

    #[test]
    fn test_check_state_toggle() {
        assert_eq!(CheckState::Unchecked.toggle(), CheckState::Checked);
        assert_eq!(CheckState::Checked.toggle(), CheckState::Unchecked);
        assert_eq!(CheckState::Indeterminate.toggle(), CheckState::Checked);
    }

    #[test]
    fn test_disabled_toggle() {
        let mut checkbox = Checkbox::new("test").disabled(true);
        checkbox.toggle();
        assert!(!checkbox.is_checked()); // Should not toggle when disabled
    }
}
