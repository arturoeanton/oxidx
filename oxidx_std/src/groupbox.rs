//! GroupBox component for OxidX
//!
//! A container with optional title/header that groups related controls.
//!
//! Features:
//! - Optional title with customizable style
//! - Collapsible (expand/collapse)
//! - Custom border and background
//! - Padding control
//! - Child component management
//! - Animated collapse transition

use oxidx_core::{
    component::OxidXComponent,
    context::OxidXContext,
    events::{Event, MouseButton},
    layout::{Alignment, Rect, Spacing},
    primitives::{Color, TextStyle},
    renderer::RenderCommand,
    style::Style,
    theme::Theme,
};

// ============================================================================
// Types
// ============================================================================

/// Border style for GroupBox
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GroupBoxBorder {
    #[default]
    Solid,
    Dashed,
    None,
}

/// Title position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TitlePosition {
    #[default]
    TopLeft,
    TopCenter,
    TopRight,
}

// ============================================================================
// Animation State
// ============================================================================

#[derive(Debug, Clone)]
struct CollapseAnimation {
    /// Current height multiplier (1.0 = expanded, 0.0 = collapsed)
    progress: f32,
    /// Target progress
    target: f32,
    /// Animation speed
    speed: f32,
    /// Content height when fully expanded
    expanded_height: f32,
}

impl Default for CollapseAnimation {
    fn default() -> Self {
        Self {
            progress: 1.0,
            target: 1.0,
            speed: 6.0,
            expanded_height: 0.0,
        }
    }
}

impl CollapseAnimation {
    fn update(&mut self, dt: f32) -> bool {
        if (self.progress - self.target).abs() < 0.001 {
            self.progress = self.target;
            return false;
        }

        let direction = if self.target > self.progress {
            1.0
        } else {
            -1.0
        };

        self.progress += direction * self.speed * dt;
        self.progress = self.progress.clamp(0.0, 1.0);
        true
    }

    fn set_collapsed(&mut self, collapsed: bool) {
        self.target = if collapsed { 0.0 } else { 1.0 };
    }

    fn current_height(&self) -> f32 {
        self.expanded_height * self.progress
    }

    fn is_fully_collapsed(&self) -> bool {
        self.progress < 0.001
    }
}

// ============================================================================
// GroupBox Component
// ============================================================================

/// A container that groups related controls with an optional title
///
/// # Example
/// ```rust
/// let group = GroupBox::new("user_info")
///     .title("User Information")
///     .collapsible(true)
///     .padding(Spacing::uniform(12.0))
///     .children(vec![
///         Box::new(Input::new("name").placeholder("Name")),
///         Box::new(Input::new("email").placeholder("Email")),
///     ]);
/// ```
pub struct GroupBox {
    // Identity
    id: String,

    // Content
    title: Option<String>,
    title_position: TitlePosition,

    // Visual
    border_style: GroupBoxBorder,
    border_color: Option<Color>,
    border_width: f32,
    corner_radius: f32,
    background_color: Option<Color>,
    title_font_size: f32,
    title_color: Option<Color>,
    title_bold: bool,

    // Layout
    padding: Spacing,
    content_spacing: f32,

    // Collapse
    collapsible: bool,
    collapsed: bool,
    collapse_animation: CollapseAnimation,

    // Interaction
    header_hovered: bool,
    header_pressed: bool,

    // Children
    children: Vec<Box<dyn OxidXComponent>>,

    // Layout cache
    bounds: Rect,
    header_rect: Rect,
    content_rect: Rect,
    collapse_icon_rect: Rect,
}

impl GroupBox {
    /// Create a new group box with the given ID
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: None,
            title_position: TitlePosition::TopLeft,
            border_style: GroupBoxBorder::Solid,
            border_color: None,
            border_width: 1.0,
            corner_radius: 6.0,
            background_color: None,
            title_font_size: 14.0,
            title_color: None,
            title_bold: true,
            padding: Spacing::uniform(12.0),
            content_spacing: 8.0,
            collapsible: false,
            collapsed: false,
            collapse_animation: CollapseAnimation::default(),
            header_hovered: false,
            header_pressed: false,
            children: Vec::new(),
            bounds: Rect::ZERO,
            header_rect: Rect::ZERO,
            content_rect: Rect::ZERO,
            collapse_icon_rect: Rect::ZERO,
        }
    }

    // ========================================================================
    // Builder Methods
    // ========================================================================

    /// Set the title text
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set title position
    pub fn title_position(mut self, position: TitlePosition) -> Self {
        self.title_position = position;
        self
    }

    /// Set title font size
    pub fn title_font_size(mut self, size: f32) -> Self {
        self.title_font_size = size;
        self
    }

    /// Set title bold
    pub fn title_bold(mut self, bold: bool) -> Self {
        self.title_bold = bold;
        self
    }

    /// Set custom title color
    pub fn title_color(mut self, color: Color) -> Self {
        self.title_color = Some(color);
        self
    }

    /// Set border style
    pub fn border_style(mut self, style: GroupBoxBorder) -> Self {
        self.border_style = style;
        self
    }

    /// Set custom border color
    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = Some(color);
        self
    }

    /// Set border width
    pub fn border_width(mut self, width: f32) -> Self {
        self.border_width = width;
        self
    }

    /// Set corner radius
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    /// Set background color
    pub fn background(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }

    /// Set padding
    pub fn padding(mut self, padding: Spacing) -> Self {
        self.padding = padding;
        self
    }

    /// Set content spacing between children
    pub fn content_spacing(mut self, spacing: f32) -> Self {
        self.content_spacing = spacing;
        self
    }

    /// Enable/disable collapsibility
    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.collapsible = collapsible;
        self
    }

    /// Set initial collapsed state
    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self.collapse_animation.target = if collapsed { 0.0 } else { 1.0 };
        self.collapse_animation.progress = self.collapse_animation.target;
        self
    }

    /// Add children components
    pub fn children(mut self, children: Vec<Box<dyn OxidXComponent>>) -> Self {
        self.children = children;
        self
    }

    /// Add a single child
    pub fn add_child(mut self, child: Box<dyn OxidXComponent>) -> Self {
        self.children.push(child);
        self
    }

    // ========================================================================
    // State Methods
    // ========================================================================

    /// Check if collapsed
    pub fn is_collapsed(&self) -> bool {
        self.collapsed
    }

    /// Toggle collapse state
    pub fn toggle_collapse(&mut self) {
        if self.collapsible {
            self.collapsed = !self.collapsed;
            self.collapse_animation.set_collapsed(self.collapsed);
        }
    }

    /// Set collapse state
    pub fn set_collapsed(&mut self, collapsed: bool) {
        if self.collapsible && self.collapsed != collapsed {
            self.collapsed = collapsed;
            self.collapse_animation.set_collapsed(collapsed);
        }
    }

    /// Add a child at runtime
    pub fn push_child(&mut self, child: Box<dyn OxidXComponent>) {
        self.children.push(child);
    }

    /// Remove a child by ID
    pub fn remove_child(&mut self, id: &str) -> Option<Box<dyn OxidXComponent>> {
        if let Some(pos) = self.children.iter().position(|c| c.id() == id) {
            Some(self.children.remove(pos))
        } else {
            None
        }
    }

    /// Get child by ID
    pub fn get_child(&self, id: &str) -> Option<&dyn OxidXComponent> {
        self.children
            .iter()
            .find(|c| c.id() == id)
            .map(|c| c.as_ref())
    }

    /// Get mutable child by ID
    pub fn get_child_mut(&mut self, id: &str) -> Option<&mut dyn OxidXComponent> {
        self.children
            .iter_mut()
            .find(|c| c.id() == id)
            .map(|c| c.as_mut())
    }

    // ========================================================================
    // Layout
    // ========================================================================

    fn header_height(&self) -> f32 {
        if self.title.is_some() {
            self.title_font_size * 1.8 + 8.0
        } else {
            0.0
        }
    }

    fn compute_layout(&mut self, ctx: &OxidXContext) {
        let header_h = self.header_height();

        // Header rect
        self.header_rect = Rect {
            x: self.bounds.x,
            y: self.bounds.y,
            width: self.bounds.width,
            height: header_h,
        };

        // Collapse icon rect (positioned at right of header)
        if self.collapsible && self.title.is_some() {
            let icon_size = self.title_font_size;
            self.collapse_icon_rect = Rect {
                x: self.bounds.x + self.bounds.width - self.padding.right - icon_size,
                y: self.bounds.y + (header_h - icon_size) / 2.0,
                width: icon_size,
                height: icon_size,
            };
        }

        // Calculate expanded content height
        let mut total_children_height: f32 = 0.0;
        for child in &self.children {
            let (_, h) = child.preferred_size(ctx);
            total_children_height += h + self.content_spacing;
        }
        if !self.children.is_empty() {
            total_children_height -= self.content_spacing;
        }

        self.collapse_animation.expanded_height =
            total_children_height + self.padding.top + self.padding.bottom;

        // Content rect (animated height)
        let content_height = self.collapse_animation.current_height();
        self.content_rect = Rect {
            x: self.bounds.x + self.padding.left,
            y: self.bounds.y + header_h + self.padding.top,
            width: self.bounds.width - self.padding.left - self.padding.right,
            height: content_height - self.padding.top - self.padding.bottom,
        };

        // Layout children vertically
        if !self.collapse_animation.is_fully_collapsed() {
            let mut y_offset = self.content_rect.y;
            for child in &mut self.children {
                let (_, pref_h) = child.preferred_size(ctx);
                child.set_bounds(Rect {
                    x: self.content_rect.x,
                    y: y_offset,
                    width: self.content_rect.width,
                    height: pref_h,
                });
                y_offset += pref_h + self.content_spacing;
            }
        }
    }

    fn hit_test_header(&self, x: f32, y: f32) -> bool {
        self.header_rect.contains(x, y)
    }

    // ========================================================================
    // Rendering Helpers
    // ========================================================================

    fn draw_collapse_icon(&self, commands: &mut Vec<RenderCommand>, theme: &Theme) {
        let icon_color = self.title_color.unwrap_or(theme.text);
        let cx = self.collapse_icon_rect.x + self.collapse_icon_rect.width / 2.0;
        let cy = self.collapse_icon_rect.y + self.collapse_icon_rect.height / 2.0;
        let size = self.collapse_icon_rect.width * 0.4;

        // Draw chevron (rotated based on collapse state)
        let progress = self.collapse_animation.progress;

        if progress > 0.5 {
            // Pointing down (expanded)
            commands.push(RenderCommand::Line {
                x1: cx - size,
                y1: cy - size * 0.3,
                x2: cx,
                y2: cy + size * 0.3,
                color: icon_color,
                width: 2.0,
            });
            commands.push(RenderCommand::Line {
                x1: cx,
                y1: cy + size * 0.3,
                x2: cx + size,
                y2: cy - size * 0.3,
                color: icon_color,
                width: 2.0,
            });
        } else {
            // Pointing right (collapsed)
            commands.push(RenderCommand::Line {
                x1: cx - size * 0.3,
                y1: cy - size,
                x2: cx + size * 0.3,
                y2: cy,
                color: icon_color,
                width: 2.0,
            });
            commands.push(RenderCommand::Line {
                x1: cx + size * 0.3,
                y1: cy,
                x2: cx - size * 0.3,
                y2: cy + size,
                color: icon_color,
                width: 2.0,
            });
        }
    }
}

// ============================================================================
// OxidXComponent Implementation
// ============================================================================

impl OxidXComponent for GroupBox {
    fn id(&self) -> &str {
        &self.id
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn bounds(&self) -> Rect {
        // Return actual bounds including animated content
        let header_h = self.header_height();
        let content_h = self.collapse_animation.current_height();

        Rect {
            x: self.bounds.x,
            y: self.bounds.y,
            width: self.bounds.width,
            height: header_h + content_h,
        }
    }

    fn preferred_size(&self, ctx: &OxidXContext) -> (f32, f32) {
        let header_h = self.header_height();

        // Calculate children size
        let mut max_width: f32 = 200.0; // Minimum width
        let mut total_height: f32 = 0.0;

        for child in &self.children {
            let (w, h) = child.preferred_size(ctx);
            max_width = max_width.max(w);
            total_height += h + self.content_spacing;
        }

        if !self.children.is_empty() {
            total_height -= self.content_spacing;
        }

        let content_h = if self.collapsed {
            0.0
        } else {
            total_height + self.padding.top + self.padding.bottom
        };

        (
            max_width + self.padding.left + self.padding.right,
            header_h + content_h,
        )
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut OxidXContext) -> bool {
        // Handle header interaction for collapse
        match event {
            Event::MouseMove { x, y } => {
                let was_hovered = self.header_hovered;
                self.header_hovered = self.collapsible && self.hit_test_header(*x, *y);

                if self.header_hovered {
                    ctx.set_cursor_icon(winit::window::CursorIcon::Pointer);
                }

                // Propagate to children if not collapsed
                if !self.collapse_animation.is_fully_collapsed() {
                    for child in &mut self.children {
                        child.handle_event(event, ctx);
                    }
                }

                was_hovered != self.header_hovered
            }

            Event::MouseButton {
                button: MouseButton::Left,
                pressed,
                x,
                y,
            } => {
                if self.collapsible && self.hit_test_header(*x, *y) {
                    if *pressed {
                        self.header_pressed = true;
                        return true;
                    } else if self.header_pressed {
                        self.header_pressed = false;
                        self.toggle_collapse();
                        return true;
                    }
                }

                if !pressed {
                    self.header_pressed = false;
                }

                // Propagate to children
                if !self.collapse_animation.is_fully_collapsed() {
                    for child in &mut self.children {
                        if child.handle_event(event, ctx) {
                            return true;
                        }
                    }
                }

                false
            }

            _ => {
                // Propagate other events to children
                if !self.collapse_animation.is_fully_collapsed() {
                    for child in &mut self.children {
                        if child.handle_event(event, ctx) {
                            return true;
                        }
                    }
                }
                false
            }
        }
    }

    fn update(&mut self, dt: f32, ctx: &mut OxidXContext) -> bool {
        self.compute_layout(ctx);

        let mut needs_redraw = self.collapse_animation.update(dt);

        // Update children
        if !self.collapse_animation.is_fully_collapsed() {
            for child in &mut self.children {
                if child.update(dt, ctx) {
                    needs_redraw = true;
                }
            }
        }

        needs_redraw
    }

    fn render(&self, ctx: &OxidXContext, commands: &mut Vec<RenderCommand>) {
        let theme = ctx.theme();
        let actual_bounds = self.bounds();

        // Draw background
        if let Some(bg_color) = self.background_color {
            commands.push(RenderCommand::RoundedRect {
                rect: actual_bounds,
                color: bg_color,
                corner_radius: self.corner_radius,
            });
        }

        // Draw border
        if self.border_style != GroupBoxBorder::None {
            let border_color = self.border_color.unwrap_or(theme.border);

            match self.border_style {
                GroupBoxBorder::Solid => {
                    commands.push(RenderCommand::RoundedRectStroke {
                        rect: actual_bounds,
                        color: border_color,
                        corner_radius: self.corner_radius,
                        stroke_width: self.border_width,
                    });
                }
                GroupBoxBorder::Dashed => {
                    // For dashed, we draw individual segments
                    // This is a simplified version - draw as solid for now
                    commands.push(RenderCommand::RoundedRectStroke {
                        rect: actual_bounds,
                        color: border_color,
                        corner_radius: self.corner_radius,
                        stroke_width: self.border_width,
                    });
                }
                GroupBoxBorder::None => {}
            }
        }

        // Draw title
        if let Some(ref title) = self.title {
            let text_style = TextStyle {
                font_size: self.title_font_size,
                color: self.title_color.unwrap_or(theme.text),
                bold: self.title_bold,
                ..Default::default()
            };

            // Title background to "cut" the border
            let title_width = ctx.measure_text(title, self.title_font_size).0;
            let title_padding = 8.0;

            let title_x = match self.title_position {
                TitlePosition::TopLeft => self.bounds.x + self.padding.left,
                TitlePosition::TopCenter => self.bounds.x + (self.bounds.width - title_width) / 2.0,
                TitlePosition::TopRight => {
                    self.bounds.x + self.bounds.width
                        - self.padding.right
                        - title_width
                        - if self.collapsible { 24.0 } else { 0.0 }
                }
            };

            let title_y = self.bounds.y + (self.header_height() - self.title_font_size) / 2.0;

            // Draw title background (to mask border)
            if self.border_style != GroupBoxBorder::None {
                let bg = self.background_color.unwrap_or(theme.background);
                commands.push(RenderCommand::Rect {
                    rect: Rect {
                        x: title_x - title_padding / 2.0,
                        y: self.bounds.y - 1.0,
                        width: title_width + title_padding,
                        height: self.border_width + 2.0,
                    },
                    color: bg,
                });
            }

            commands.push(RenderCommand::Text {
                text: title.clone(),
                x: title_x,
                y: title_y,
                style: text_style,
                max_width: None,
            });

            // Draw collapse icon
            if self.collapsible {
                self.draw_collapse_icon(commands, theme);
            }
        }

        // Draw children (with clipping)
        if !self.collapse_animation.is_fully_collapsed() {
            // Push clip rect for content area
            commands.push(RenderCommand::PushClip {
                rect: Rect {
                    x: self.bounds.x,
                    y: self.bounds.y + self.header_height(),
                    width: self.bounds.width,
                    height: self.collapse_animation.current_height(),
                },
            });

            for child in &self.children {
                child.render(ctx, commands);
            }

            commands.push(RenderCommand::PopClip);
        }
    }

    fn focusable(&self) -> bool {
        false
    }

    fn children(&self) -> Option<Vec<&dyn OxidXComponent>> {
        Some(
            self.children
                .iter()
                .map(|c| c.as_ref() as &dyn OxidXComponent)
                .collect(),
        )
    }

    fn children_mut(&mut self) -> Option<Vec<&mut dyn OxidXComponent>> {
        Some(
            self.children
                .iter_mut()
                .map(|c| c.as_mut() as &mut dyn OxidXComponent)
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
    fn test_groupbox_creation() {
        let group = GroupBox::new("test");
        assert_eq!(group.id(), "test");
        assert!(!group.is_collapsed());
    }

    #[test]
    fn test_groupbox_builder() {
        let group = GroupBox::new("test")
            .title("Test Group")
            .collapsible(true)
            .collapsed(true)
            .corner_radius(8.0)
            .padding(Spacing::uniform(16.0));

        assert!(group.is_collapsed());
        assert_eq!(group.title, Some("Test Group".to_string()));
    }

    #[test]
    fn test_collapse_toggle() {
        let mut group = GroupBox::new("test").collapsible(true);

        assert!(!group.is_collapsed());
        group.toggle_collapse();
        assert!(group.is_collapsed());
        group.toggle_collapse();
        assert!(!group.is_collapsed());
    }

    #[test]
    fn test_non_collapsible() {
        let mut group = GroupBox::new("test").collapsible(false);

        group.toggle_collapse();
        assert!(!group.is_collapsed()); // Should not collapse
    }

    #[test]
    fn test_header_height() {
        let group_with_title = GroupBox::new("test").title("Title");
        let group_without_title = GroupBox::new("test");

        assert!(group_with_title.header_height() > 0.0);
        assert_eq!(group_without_title.header_height(), 0.0);
    }
}
