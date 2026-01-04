//! SplitView Component
//!
//! A resizable container that divides space into two panes:
//! - Horizontal: Left | Right
//! - Vertical: Top | Bottom
//!
//! Features:
//! - Draggable gutter to resize panes
//! - Configurable split ratio
//! - Min/max ratio constraints
//! - Cursor change on hover

use glam::Vec2;
use oxidx_core::component::OxidXComponent;
use oxidx_core::events::OxidXEvent;
use oxidx_core::layout::LayoutProps;
use oxidx_core::primitives::{Color, Rect};
use oxidx_core::renderer::Renderer;
use oxidx_core::{CursorIcon, OxidXContext};

/// Split direction for the SplitView.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SplitDirection {
    /// Left | Right split
    #[default]
    Horizontal,
    /// Top | Bottom split
    Vertical,
}

/// Style configuration for the gutter.
#[derive(Debug, Clone)]
pub struct GutterStyle {
    /// Width/height of the gutter in pixels
    pub size: f32,
    /// Normal color
    pub color: Color,
    /// Color when hovered
    pub hover_color: Color,
    /// Color when being dragged
    pub drag_color: Color,
}

impl Default for GutterStyle {
    fn default() -> Self {
        // Subtle zinc colors for professional divider
        Self {
            size: 6.0,
            // border_subtle: #3f3f46
            color: Color::from_hex("3f3f46").unwrap_or(Color::new(0.25, 0.25, 0.27, 1.0)),
            // surface_hover
            hover_color: Color::from_hex("52525b").unwrap_or(Color::new(0.32, 0.32, 0.36, 1.0)),
            // primary on drag
            drag_color: Color::from_hex("6366f1").unwrap_or(Color::new(0.39, 0.4, 0.95, 1.0)),
        }
    }
}

/// A resizable split container with two panes.
///
/// # Example
/// ```ignore
/// let split = SplitView::horizontal(
///     Label::new("Left Panel"),
///     Label::new("Right Panel"),
/// ).with_ratio(0.3);
/// ```
pub struct SplitView {
    /// First panel (left or top)
    first: Box<dyn OxidXComponent>,

    /// Second panel (right or bottom)
    second: Box<dyn OxidXComponent>,

    /// Split direction
    direction: SplitDirection,

    /// Split ratio (0.0 to 1.0) - position of the split
    split_ratio: f32,

    /// Minimum split ratio
    min_ratio: f32,

    /// Maximum split ratio
    max_ratio: f32,

    /// Whether the gutter is being dragged
    is_dragging: bool,

    /// Whether the gutter is hovered
    is_hovered: bool,

    /// Gutter styling
    gutter_style: GutterStyle,

    /// Layout properties
    layout: LayoutProps,

    /// Component bounds
    bounds: Rect,

    /// Cached gutter rect
    gutter_rect: Rect,

    /// Component ID
    id: String,
}

impl SplitView {
    /// Creates a new horizontal split (Left | Right).
    pub fn horizontal(
        first: impl OxidXComponent + 'static,
        second: impl OxidXComponent + 'static,
    ) -> Self {
        Self::new(first, second, SplitDirection::Horizontal)
    }

    /// Creates a new vertical split (Top | Bottom).
    pub fn vertical(
        first: impl OxidXComponent + 'static,
        second: impl OxidXComponent + 'static,
    ) -> Self {
        Self::new(first, second, SplitDirection::Vertical)
    }

    /// Creates a new split view with specified direction.
    pub fn new(
        first: impl OxidXComponent + 'static,
        second: impl OxidXComponent + 'static,
        direction: SplitDirection,
    ) -> Self {
        Self {
            first: Box::new(first),
            second: Box::new(second),
            direction,
            split_ratio: 0.3,
            min_ratio: 0.1,
            max_ratio: 0.9,
            is_dragging: false,
            is_hovered: false,
            gutter_style: GutterStyle::default(),
            layout: LayoutProps::default(),
            bounds: Rect::default(),
            gutter_rect: Rect::default(),
            id: String::new(),
        }
    }

    // === Builder Methods ===

    /// Sets the split ratio (0.0 to 1.0).
    pub fn with_ratio(mut self, ratio: f32) -> Self {
        self.split_ratio = ratio.clamp(self.min_ratio, self.max_ratio);
        self
    }

    /// Sets the minimum ratio constraint.
    pub fn with_min_ratio(mut self, min: f32) -> Self {
        self.min_ratio = min.clamp(0.05, 0.95);
        self
    }

    /// Sets the maximum ratio constraint.
    pub fn with_max_ratio(mut self, max: f32) -> Self {
        self.max_ratio = max.clamp(0.05, 0.95);
        self
    }

    /// Sets the gutter style.
    pub fn with_gutter_style(mut self, style: GutterStyle) -> Self {
        self.gutter_style = style;
        self
    }

    /// Sets the gutter size.
    pub fn with_gutter_size(mut self, size: f32) -> Self {
        self.gutter_style.size = size;
        self
    }

    /// Sets layout properties.
    pub fn with_layout(mut self, layout: LayoutProps) -> Self {
        self.layout = layout;
        self
    }

    /// Sets the component ID.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    // === Internal Methods ===

    /// Calculates the first panel bounds.
    fn first_bounds(&self) -> Rect {
        let gutter = self.gutter_style.size;

        match self.direction {
            SplitDirection::Horizontal => {
                let split_x = self.bounds.x + self.bounds.width * self.split_ratio;
                Rect::new(
                    self.bounds.x,
                    self.bounds.y,
                    split_x - self.bounds.x - gutter / 2.0,
                    self.bounds.height,
                )
            }
            SplitDirection::Vertical => {
                let split_y = self.bounds.y + self.bounds.height * self.split_ratio;
                Rect::new(
                    self.bounds.x,
                    self.bounds.y,
                    self.bounds.width,
                    split_y - self.bounds.y - gutter / 2.0,
                )
            }
        }
    }

    /// Calculates the second panel bounds.
    fn second_bounds(&self) -> Rect {
        let gutter = self.gutter_style.size;

        match self.direction {
            SplitDirection::Horizontal => {
                let split_x = self.bounds.x + self.bounds.width * self.split_ratio;
                let start_x = split_x + gutter / 2.0;
                Rect::new(
                    start_x,
                    self.bounds.y,
                    self.bounds.x + self.bounds.width - start_x,
                    self.bounds.height,
                )
            }
            SplitDirection::Vertical => {
                let split_y = self.bounds.y + self.bounds.height * self.split_ratio;
                let start_y = split_y + gutter / 2.0;
                Rect::new(
                    self.bounds.x,
                    start_y,
                    self.bounds.width,
                    self.bounds.y + self.bounds.height - start_y,
                )
            }
        }
    }

    /// Calculates the gutter bounds.
    fn calc_gutter_rect(&self) -> Rect {
        let gutter = self.gutter_style.size;

        match self.direction {
            SplitDirection::Horizontal => {
                let split_x = self.bounds.x + self.bounds.width * self.split_ratio;
                Rect::new(
                    split_x - gutter / 2.0,
                    self.bounds.y,
                    gutter,
                    self.bounds.height,
                )
            }
            SplitDirection::Vertical => {
                let split_y = self.bounds.y + self.bounds.height * self.split_ratio;
                Rect::new(
                    self.bounds.x,
                    split_y - gutter / 2.0,
                    self.bounds.width,
                    gutter,
                )
            }
        }
    }

    /// Updates the split ratio from mouse position.
    fn update_ratio_from_position(&mut self, pos: Vec2) {
        let new_ratio = match self.direction {
            SplitDirection::Horizontal => (pos.x - self.bounds.x) / self.bounds.width,
            SplitDirection::Vertical => (pos.y - self.bounds.y) / self.bounds.height,
        };

        self.split_ratio = new_ratio.clamp(self.min_ratio, self.max_ratio);
    }
}

impl OxidXComponent for SplitView {
    fn update(&mut self, dt: f32) {
        self.first.update(dt);
        self.second.update(dt);
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        let margin = self.layout.margin;

        self.bounds = Rect::new(
            available.x + margin,
            available.y + margin,
            available.width - margin * 2.0,
            available.height - margin * 2.0,
        );

        // Cache gutter rect
        self.gutter_rect = self.calc_gutter_rect();

        // Layout children
        let first_rect = self.first_bounds();
        let second_rect = self.second_bounds();

        self.first.layout(first_rect);
        self.second.layout(second_rect);

        Vec2::new(
            self.bounds.width + margin * 2.0,
            self.bounds.height + margin * 2.0,
        )
    }

    fn render(&self, renderer: &mut Renderer) {
        // Render first panel
        self.first.render(renderer);

        // Render second panel
        self.second.render(renderer);

        // Render gutter as clean 1px divider line
        let gutter_color = if self.is_dragging {
            self.gutter_style.drag_color
        } else if self.is_hovered {
            self.gutter_style.hover_color
        } else {
            self.gutter_style.color
        };

        // Draw a thin 1px line centered in the gutter
        let line_rect = match self.direction {
            SplitDirection::Horizontal => Rect::new(
                self.gutter_rect.x + self.gutter_rect.width / 2.0 - 0.5,
                self.gutter_rect.y,
                1.0,
                self.gutter_rect.height,
            ),
            SplitDirection::Vertical => Rect::new(
                self.gutter_rect.x,
                self.gutter_rect.y + self.gutter_rect.height / 2.0 - 0.5,
                self.gutter_rect.width,
                1.0,
            ),
        };

        renderer.fill_rect(line_rect, gutter_color);
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        match event {
            OxidXEvent::MouseMove { position, .. } => {
                // Update hover state
                let was_hovered = self.is_hovered;
                self.is_hovered = self.gutter_rect.contains(*position);

                // Set cursor when over gutter
                if self.is_hovered {
                    match self.direction {
                        SplitDirection::Horizontal => ctx.set_cursor_icon(CursorIcon::ColResize),
                        SplitDirection::Vertical => ctx.set_cursor_icon(CursorIcon::RowResize),
                    }
                } else if was_hovered && !self.is_dragging {
                    ctx.set_cursor_icon(CursorIcon::Default);
                }

                // Handle dragging
                if self.is_dragging {
                    self.update_ratio_from_position(*position);
                    // Keep resize cursor while dragging
                    match self.direction {
                        SplitDirection::Horizontal => ctx.set_cursor_icon(CursorIcon::ColResize),
                        SplitDirection::Vertical => ctx.set_cursor_icon(CursorIcon::RowResize),
                    }
                    return true;
                }

                // Forward to children if not dragging
                let first_bounds = self.first_bounds();
                let second_bounds = self.second_bounds();

                if first_bounds.contains(*position) {
                    self.first.on_event(event, ctx);
                }
                if second_bounds.contains(*position) {
                    self.second.on_event(event, ctx);
                }

                was_hovered != self.is_hovered
            }

            OxidXEvent::MouseDown { position, .. } => {
                // Start dragging if clicking on gutter
                if self.gutter_rect.contains(*position) {
                    self.is_dragging = true;
                    return true;
                }

                // Forward to children
                let first_bounds = self.first_bounds();
                let second_bounds = self.second_bounds();

                if first_bounds.contains(*position) {
                    return self.first.on_event(event, ctx);
                }
                if second_bounds.contains(*position) {
                    return self.second.on_event(event, ctx);
                }

                false
            }

            OxidXEvent::MouseUp { position, .. } => {
                if self.is_dragging {
                    self.is_dragging = false;
                    ctx.set_cursor_icon(CursorIcon::Default);
                    return true;
                }

                // Forward to children
                let first_bounds = self.first_bounds();
                let second_bounds = self.second_bounds();

                if first_bounds.contains(*position) {
                    return self.first.on_event(event, ctx);
                }
                if second_bounds.contains(*position) {
                    return self.second.on_event(event, ctx);
                }

                false
            }

            OxidXEvent::Click { position, .. } => {
                // Forward to children
                let first_bounds = self.first_bounds();
                let second_bounds = self.second_bounds();

                if first_bounds.contains(*position) {
                    return self.first.on_event(event, ctx);
                }
                if second_bounds.contains(*position) {
                    return self.second.on_event(event, ctx);
                }

                false
            }

            OxidXEvent::MouseWheel { position, .. } => {
                // Forward to children
                let first_bounds = self.first_bounds();
                let second_bounds = self.second_bounds();

                if first_bounds.contains(*position) {
                    return self.first.on_event(event, ctx);
                }
                if second_bounds.contains(*position) {
                    return self.second.on_event(event, ctx);
                }

                false
            }

            // Non-positional events go to both children
            _ => {
                let handled1 = self.first.on_event(event, ctx);
                let handled2 = self.second.on_event(event, ctx);
                handled1 || handled2
            }
        }
    }

    fn on_keyboard_input(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) {
        self.first.on_keyboard_input(event, ctx);
        self.second.on_keyboard_input(event, ctx);
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

    fn child_count(&self) -> usize {
        2
    }
}
